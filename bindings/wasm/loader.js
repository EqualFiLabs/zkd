import { WASI } from 'node:wasi';
import { readFile } from 'node:fs/promises';

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

const ERROR_CODES = {
  0: 'ZKP_OK',
  1: 'ZKP_ERR_INVALID_ARG',
  2: 'ZKP_ERR_BACKEND',
  3: 'ZKP_ERR_PROFILE',
  4: 'ZKP_ERR_PROOF_CORRUPT',
  5: 'ZKP_ERR_VERIFY_FAIL',
  6: 'ZKP_ERR_INTERNAL'
};

function formatError(code, context) {
  const name = ERROR_CODES[code] ?? `ZKP_ERR_UNKNOWN(${code})`;
  return new Error(`${context} failed with ${name}`);
}

function createViewManager(memory) {
  let cachedBuffer = memory.buffer;
  let cachedU8 = new Uint8Array(cachedBuffer);
  let cachedDataView = new DataView(cachedBuffer);

  function refresh() {
    if (cachedBuffer !== memory.buffer) {
      cachedBuffer = memory.buffer;
      cachedU8 = new Uint8Array(cachedBuffer);
      cachedDataView = new DataView(cachedBuffer);
    }
  }

  return {
    u8() {
      refresh();
      return cachedU8;
    },
    data() {
      refresh();
      return cachedDataView;
    }
  };
}

function ensureNumber(value, name) {
  const num = Number(value);
  if (!Number.isFinite(num)) {
    throw new Error(`${name} must be a finite number`);
  }
  return num;
}

function ensureString(value, name) {
  if (typeof value !== 'string') {
    throw new Error(`${name} must be a string`);
  }
  return value;
}

function normalizeBytes(bytes) {
  if (bytes instanceof Uint8Array) {
    return bytes;
  }
  if (ArrayBuffer.isView(bytes)) {
    return new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  }
  if (bytes instanceof ArrayBuffer) {
    return new Uint8Array(bytes);
  }
  if (typeof bytes === 'string') {
    return new TextEncoder().encode(bytes);
  }
  throw new Error('proof must be a Uint8Array, Buffer, ArrayBuffer, or string');
}

export async function init({ wasmPath, preopens = {}, args = [], env = {} } = {}) {
  if (!wasmPath) {
    throw new Error('wasmPath is required');
  }

  const wasmBytes = await readFile(wasmPath);
  const wasi = new WASI({ args, env, preopens, version: 'preview1' });
  const module = await WebAssembly.compile(wasmBytes);
  const instance = await WebAssembly.instantiate(module, {
    wasi_snapshot_preview1: wasi.wasiImport
  });
  wasi.initialize(instance);

  const {
    memory,
    zkp_alloc: wasmAlloc,
    zkp_free: wasmFree,
    zkp_init,
    zkp_list_backends,
    zkp_list_profiles,
    zkp_prove,
    zkp_verify
  } = instance.exports;

  const views = createViewManager(memory);

  function allocBytes(size) {
    const ptr = Number(wasmAlloc(BigInt(size)));
    if (ptr === 0) {
      throw new Error(`zkp_alloc failed for ${size} bytes`);
    }
    return ptr;
  }

  function allocCString(str) {
    const bytes = textEncoder.encode(str);
    const ptr = allocBytes(bytes.length + 1);
    const u8 = views.u8();
    u8.set(bytes, ptr);
    u8[ptr + bytes.length] = 0;
    return ptr;
  }

  function readCString(ptr) {
    if (!ptr) {
      return '';
    }
    const u8 = views.u8();
    let end = ptr;
    while (u8[end] !== 0) {
      end++;
    }
    return textDecoder.decode(u8.subarray(ptr, end));
  }

  function allocPtr32() {
    const ptr = allocBytes(4);
    views.data().setUint32(ptr, 0, true);
    return ptr;
  }

  function allocPtr64() {
    const ptr = allocBytes(8);
    views.data().setBigUint64(ptr, 0n, true);
    return ptr;
  }

  function loadPtr(ptr) {
    return views.data().getUint32(ptr, true);
  }

  function loadU64(ptr) {
    return views.data().getBigUint64(ptr, true);
  }

  function free(ptr) {
    if (ptr) {
      wasmFree(ptr);
    }
  }

  const rcInit = zkp_init();
  if (rcInit !== 0) {
    throw formatError(rcInit, 'zkp_init');
  }

  async function listBackends() {
    const outPtr = allocPtr32();
    try {
      const rc = zkp_list_backends(outPtr);
      if (rc !== 0) {
        throw formatError(rc, 'zkp_list_backends');
      }
      const strPtr = loadPtr(outPtr);
      if (!strPtr) {
        return null;
      }
      try {
        const json = readCString(strPtr);
        return JSON.parse(json);
      } finally {
        free(strPtr);
      }
    } finally {
      free(outPtr);
    }
  }

  async function listProfiles() {
    const outPtr = allocPtr32();
    try {
      const rc = zkp_list_profiles(outPtr);
      if (rc !== 0) {
        throw formatError(rc, 'zkp_list_profiles');
      }
      const strPtr = loadPtr(outPtr);
      if (!strPtr) {
        return null;
      }
      try {
        const json = readCString(strPtr);
        return JSON.parse(json);
      } finally {
        free(strPtr);
      }
    } finally {
      free(outPtr);
    }
  }

  function prove(config) {
    if (!config || typeof config !== 'object') {
      throw new Error('config must be an object');
    }
    const backend = allocCString(ensureString(config.backendId, 'config.backendId'));
    const field = allocCString(ensureString(config.field, 'config.field'));
    const hash = allocCString(ensureString(config.hashId, 'config.hashId'));
    const profile = allocCString(ensureString(config.profileId, 'config.profileId'));
    const air = allocCString(ensureString(config.airPath, 'config.airPath'));
    const publicInputs = allocCString(ensureString(config.publicInputsJson, 'config.publicInputsJson'));
    const outProofPtr = allocPtr32();
    const outProofLen = allocPtr64();
    const outMetaPtr = allocPtr32();

    try {
      const friArity = ensureNumber(config.friArity, 'config.friArity');
      const rc = zkp_prove(
        backend,
        field,
        hash,
        friArity >>> 0,
        profile,
        air,
        publicInputs,
        outProofPtr,
        outProofLen,
        outMetaPtr
      );
      if (rc !== 0) {
        throw formatError(rc, 'zkp_prove');
      }
      const proofPtr = loadPtr(outProofPtr);
      const proofLen = loadU64(outProofLen);
      const metaPtr = loadPtr(outMetaPtr);
      if (!proofPtr || proofLen === 0n || !metaPtr) {
        throw new Error('zkp_prove returned invalid outputs');
      }
      const proofView = new Uint8Array(views.u8().buffer, proofPtr, Number(proofLen));
      const proof = Buffer.from(proofView);
      const metaJson = readCString(metaPtr);
      const meta = JSON.parse(metaJson);
      free(proofPtr);
      free(metaPtr);
      return { proof, meta };
    } finally {
      free(outProofPtr);
      free(outProofLen);
      free(outMetaPtr);
      free(backend);
      free(field);
      free(hash);
      free(profile);
      free(air);
      free(publicInputs);
    }
  }

  function verify(config, proofBytes) {
    if (!config || typeof config !== 'object') {
      throw new Error('config must be an object');
    }
    const backend = allocCString(ensureString(config.backendId, 'config.backendId'));
    const field = allocCString(ensureString(config.field, 'config.field'));
    const hash = allocCString(ensureString(config.hashId, 'config.hashId'));
    const profile = allocCString(ensureString(config.profileId, 'config.profileId'));
    const air = allocCString(ensureString(config.airPath, 'config.airPath'));
    const publicInputs = allocCString(ensureString(config.publicInputsJson, 'config.publicInputsJson'));
    const outMetaPtr = allocPtr32();

    const proofArray = normalizeBytes(proofBytes);
    if (proofArray.byteLength === 0) {
      throw new Error('proof must not be empty');
    }
    const proofPtr = allocBytes(proofArray.byteLength);
    views.u8().set(proofArray, proofPtr);

    try {
      const friArity = ensureNumber(config.friArity, 'config.friArity');
      const rc = zkp_verify(
        backend,
        field,
        hash,
        friArity >>> 0,
        profile,
        air,
        publicInputs,
        proofPtr,
        BigInt(proofArray.byteLength),
        outMetaPtr
      );
      if (rc !== 0) {
        throw formatError(rc, 'zkp_verify');
      }
      const metaPtr = loadPtr(outMetaPtr);
      if (!metaPtr) {
        throw new Error('zkp_verify returned invalid meta pointer');
      }
      const metaJson = readCString(metaPtr);
      const meta = JSON.parse(metaJson);
      free(metaPtr);
      return meta;
    } finally {
      free(outMetaPtr);
      free(backend);
      free(field);
      free(hash);
      free(profile);
      free(air);
      free(publicInputs);
      free(proofPtr);
    }
  }

  return {
    listBackends,
    listProfiles,
    prove,
    verify: (config, proof) => {
      const meta = verify(config, proof);
      return { verified: Boolean(meta.verified), meta };
    }
  };
}
