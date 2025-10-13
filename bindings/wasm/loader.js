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

const VIRTUAL_ROOT = '/__zkprov';
const CLIENT_ERROR_INVALID_JSON = 'CLIENT_INVALID_JSON';
const CLIENT_ERROR_SHAPE = 'CLIENT_INVALID_SHAPE';
const CLIENT_ERROR_BYTES = 'CLIENT_INVALID_BYTES';
const CLIENT_ERROR_ENVELOPE = 'CLIENT_INVALID_ENVELOPE';

const DEFAULT_POLYFILLS = {
  wasi: 'https://unpkg.com/@wasmer/wasi@1.2.2?module',
  wasmfs: 'https://unpkg.com/@wasmer/wasmfs@1.2.2?module'
};

function isNodeEnvironment() {
  return typeof process !== 'undefined' &&
    process.versions != null &&
    process.versions.node != null;
}

function createError(code, msg, detail) {
  const error = new Error(msg);
  error.name = 'ZkpError';
  if (code !== undefined) {
    error.code = code;
  }
  if (detail !== undefined) {
    error.detail = detail;
  }
  return error;
}

function formatError(code, context) {
  const name = ERROR_CODES[code] ?? `ZKP_ERR_UNKNOWN(${code})`;
  return createError(code, `${context} failed with ${name}`, { context });
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
    throw createError(CLIENT_ERROR_SHAPE, `${name} must be a finite number`);
  }
  return num;
}

function ensureString(value, name) {
  if (typeof value !== 'string') {
    throw createError(CLIENT_ERROR_SHAPE, `${name} must be a string`);
  }
  return value;
}

function normalizeBytes(bytes, name = 'bytes') {
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
    return textEncoder.encode(bytes);
  }
  throw createError(CLIENT_ERROR_BYTES, `${name} must be a Uint8Array, Buffer, ArrayBuffer, or string`);
}

function bytesToString(bytes, name) {
  const u8 = normalizeBytes(bytes, name);
  return textDecoder.decode(u8);
}

function toNodeBuffer(bytes) {
  if (typeof Buffer !== 'undefined') {
    return Buffer.from(bytes);
  }
  return new Uint8Array(bytes);
}

function randomHex(bytes = 8) {
  let out = '';
  for (let i = 0; i < bytes; i++) {
    out += Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
  }
  return out;
}

function joinPath(base, name) {
  if (base.endsWith('/')) {
    return base + name;
  }
  return `${base}/${name}`;
}

function dirname(path) {
  if (path === '/') {
    return '/';
  }
  const idx = path.lastIndexOf('/');
  if (idx <= 0) {
    return '/';
  }
  return path.slice(0, idx);
}

function parseJson(json, context) {
  try {
    return JSON.parse(json);
  } catch (err) {
    throw createError(CLIENT_ERROR_INVALID_JSON, `${context} returned invalid JSON`, { cause: err?.message, json });
  }
}

function ensureArray(value, context) {
  if (!Array.isArray(value)) {
    throw createError(CLIENT_ERROR_SHAPE, `${context} must be an array`, { value });
  }
  return value;
}

function parseEnvelope(json, context) {
  const parsed = parseJson(json, context);
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw createError(CLIENT_ERROR_ENVELOPE, `${context} envelope must be an object`, { value: parsed });
  }
  const { ok, code, msg, ...detail } = parsed;
  if (typeof ok !== 'boolean') {
    throw createError(CLIENT_ERROR_ENVELOPE, `${context} envelope missing boolean ok`, { value: parsed });
  }
  if (!Number.isInteger(code)) {
    throw createError(CLIENT_ERROR_ENVELOPE, `${context} envelope missing integer code`, { value: parsed });
  }
  if (typeof msg !== 'string') {
    throw createError(CLIENT_ERROR_ENVELOPE, `${context} envelope missing string msg`, { value: parsed });
  }
  return { ok, code, msg, detail };
}

function normalizeEnvelope(envelope, context) {
  if (!envelope.ok) {
    throw createError(envelope.code, envelope.msg || `${context} failed`, envelope.detail);
  }
  const meta = { ok: true, code: envelope.code, msg: envelope.msg, detail: envelope.detail };
  Object.assign(meta, envelope.detail);
  return meta;
}

function normalizeJsonInput(value, name) {
  if (value == null) {
    throw createError(CLIENT_ERROR_INVALID_JSON, `${name} must not be null or undefined`);
  }
  if (typeof value === 'string') {
    return normalizeJsonString(value, name);
  }
  if (typeof value === 'object' && !ArrayBuffer.isView(value) && !(value instanceof ArrayBuffer)) {
    try {
      return JSON.stringify(value);
    } catch (err) {
      throw createError(CLIENT_ERROR_INVALID_JSON, `${name} could not be serialized to JSON`, { cause: err?.message });
    }
  }
  const text = bytesToString(value, name);
  return normalizeJsonString(text, name);
}

function normalizeJsonString(text, name) {
  try {
    const parsed = JSON.parse(text);
    return JSON.stringify(parsed);
  } catch (err) {
    throw createError(CLIENT_ERROR_INVALID_JSON, `${name} must be valid JSON`, { cause: err?.message });
  }
}

function createNodeFsHelper({ rootGuest, rootHost, fsPromises, path }) {
  const normalizedRootGuest = rootGuest.endsWith('/') ? rootGuest : `${rootGuest}`;

  function toHostPath(guestPath) {
    if (!guestPath.startsWith(normalizedRootGuest)) {
      throw createError(CLIENT_ERROR_BYTES, `virtual path '${guestPath}' must be within ${normalizedRootGuest}`);
    }
    const suffix = guestPath.slice(normalizedRootGuest.length);
    const relative = suffix.startsWith('/') ? suffix.slice(1) : suffix;
    return path.join(rootHost, relative);
  }

  async function ensureDir(hostPath) {
    const dir = path.dirname(hostPath);
    await fsPromises.mkdir(dir, { recursive: true });
  }

  async function writeFile(guestPath, data) {
    const hostPath = toHostPath(guestPath);
    await ensureDir(hostPath);
    await fsPromises.writeFile(hostPath, data);
  }

  async function removeFile(guestPath) {
    const hostPath = toHostPath(guestPath);
    try {
      if (typeof fsPromises.rm === 'function') {
        await fsPromises.rm(hostPath, { force: true });
      } else {
        await fsPromises.unlink(hostPath);
      }
    } catch (err) {
      if (err && (err.code === 'ENOENT' || err.code === 'ENOTDIR')) {
        return;
      }
      throw err;
    }
  }

  async function withVirtualFile(guestPath, data, fn) {
    await writeFile(guestPath, data);
    try {
      return await fn();
    } finally {
      await removeFile(guestPath);
    }
  }

  return {
    rootGuest,
    withVirtualFile
  };
}

function createMemFsHelper({ wasmFs, rootGuest }) {
  const fs = wasmFs.fs;
  const promises = fs.promises ?? fs;

  async function writeFile(guestPath, data) {
    const dir = dirname(guestPath);
    if (promises.mkdir) {
      await promises.mkdir(dir, { recursive: true });
    } else if (fs.mkdirSync) {
      fs.mkdirSync(dir, { recursive: true });
    }
    if (promises.writeFile) {
      await promises.writeFile(guestPath, data);
    } else if (fs.writeFileSync) {
      fs.writeFileSync(guestPath, data);
    } else {
      throw createError(CLIENT_ERROR_BYTES, 'memfs does not support writeFile');
    }
  }

  async function removeFile(guestPath) {
    try {
      if (promises.unlink) {
        await promises.unlink(guestPath);
      } else if (fs.unlinkSync) {
        fs.unlinkSync(guestPath);
      }
    } catch (err) {
      if (err && (err.code === 'ENOENT' || err.code === 'ENOTDIR')) {
        return;
      }
      throw err;
    }
  }

  async function withVirtualFile(guestPath, data, fn) {
    await writeFile(guestPath, data);
    try {
      return await fn();
    } finally {
      await removeFile(guestPath);
    }
  }

  return {
    rootGuest,
    withVirtualFile
  };
}

function normalizeProofOutput(proofView) {
  const bytes = new Uint8Array(proofView);
  return toNodeBuffer(bytes);
}

function createApi(exports, fsHelper) {
  const {
    memory,
    zkp_alloc: wasmAlloc,
    zkp_free: wasmFree,
    zkp_init,
    zkp_list_backends,
    zkp_list_profiles,
    zkp_prove,
    zkp_verify
  } = exports;

  const views = createViewManager(memory);

  function allocBytes(size) {
    const ptr = Number(wasmAlloc(BigInt(size)));
    if (ptr === 0) {
      throw createError(CLIENT_ERROR_BYTES, `zkp_alloc failed for ${size} bytes`);
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
        const parsed = parseJson(json, 'zkp_list_backends');
        return ensureArray(parsed, 'zkp_list_backends');
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
        const parsed = parseJson(json, 'zkp_list_profiles');
        return ensureArray(parsed, 'zkp_list_profiles');
      } finally {
        free(strPtr);
      }
    } finally {
      free(outPtr);
    }
  }

  function parseMeta(json, context) {
    const envelope = parseEnvelope(json, context);
    return normalizeEnvelope(envelope, context);
  }

  function prove(config) {
    if (!config || typeof config !== 'object') {
      throw createError(CLIENT_ERROR_SHAPE, 'config must be an object');
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
        throw createError(CLIENT_ERROR_BYTES, 'zkp_prove returned invalid outputs');
      }
      const proofView = new Uint8Array(views.u8().buffer, proofPtr, Number(proofLen));
      const proof = normalizeProofOutput(proofView);
      const metaJson = readCString(metaPtr);
      let meta;
      try {
        meta = parseMeta(metaJson, 'zkp_prove');
      } finally {
        free(metaPtr);
      }
      free(proofPtr);
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
      throw createError(CLIENT_ERROR_SHAPE, 'config must be an object');
    }
    const backend = allocCString(ensureString(config.backendId, 'config.backendId'));
    const field = allocCString(ensureString(config.field, 'config.field'));
    const hash = allocCString(ensureString(config.hashId, 'config.hashId'));
    const profile = allocCString(ensureString(config.profileId, 'config.profileId'));
    const air = allocCString(ensureString(config.airPath, 'config.airPath'));
    const publicInputs = allocCString(ensureString(config.publicInputsJson, 'config.publicInputsJson'));
    const outMetaPtr = allocPtr32();

    const proofArray = normalizeBytes(proofBytes, 'proof');
    if (proofArray.byteLength === 0) {
      throw createError(CLIENT_ERROR_BYTES, 'proof must not be empty');
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
        throw createError(CLIENT_ERROR_BYTES, 'zkp_verify returned invalid meta pointer');
      }
      const metaJson = readCString(metaPtr);
      let meta;
      try {
        meta = parseMeta(metaJson, 'zkp_verify');
      } finally {
        free(metaPtr);
      }
      const verified = Boolean(meta.detail?.verified ?? meta.verified);
      return { verified, meta };
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

  async function proveFromBuffers(config, airBytes, inputsBytes) {
    if (!fsHelper) {
      throw createError(CLIENT_ERROR_BYTES, 'proveFromBuffers is unavailable without a virtual filesystem');
    }
    const airData = normalizeBytes(airBytes, 'airBytes');
    if (airData.byteLength === 0) {
      throw createError(CLIENT_ERROR_BYTES, 'airBytes must not be empty');
    }
    const inputsJson = normalizeJsonInput(inputsBytes, 'inputsBytes');
    const virtualAirPath = joinPath(fsHelper.rootGuest, `air-${randomHex(6)}.air`);
    return fsHelper.withVirtualFile(virtualAirPath, airData, () => {
      const proveConfig = {
        ...config,
        airPath: virtualAirPath,
        publicInputsJson: inputsJson
      };
      return prove(proveConfig);
    });
  }

  async function verifyFromBuffers(config, airBytes, inputsBytes, proofBytes) {
    if (!fsHelper) {
      throw createError(CLIENT_ERROR_BYTES, 'verifyFromBuffers is unavailable without a virtual filesystem');
    }
    const airData = normalizeBytes(airBytes, 'airBytes');
    if (airData.byteLength === 0) {
      throw createError(CLIENT_ERROR_BYTES, 'airBytes must not be empty');
    }
    const inputsJson = normalizeJsonInput(inputsBytes, 'inputsBytes');
    const virtualAirPath = joinPath(fsHelper.rootGuest, `air-${randomHex(6)}.air`);
    return fsHelper.withVirtualFile(virtualAirPath, airData, () => {
      const verifyConfig = {
        ...config,
        airPath: virtualAirPath,
        publicInputsJson: inputsJson
      };
      return verify(verifyConfig, proofBytes);
    });
  }

  return {
    listBackends,
    listProfiles,
    prove,
    proveFromBuffers,
    verifyFromBuffers,
    verify: (config, proof) => {
      const result = verify(config, proof);
      return { verified: Boolean(result.verified), meta: result.meta };
    }
  };
}

async function loadModule(specifiers) {
  const list = Array.isArray(specifiers) ? specifiers : [specifiers];
  let lastError;
  for (const spec of list) {
    try {
      return await import(spec);
    } catch (err) {
      lastError = err;
    }
  }
  throw lastError;
}

async function fetchWasmBytes(wasmPath) {
  if (typeof fetch !== 'function') {
    throw createError(CLIENT_ERROR_BYTES, 'fetch API is not available to load wasm');
  }
  const response = await fetch(wasmPath);
  if (!response.ok) {
    throw createError(CLIENT_ERROR_BYTES, `failed to fetch wasm from ${wasmPath}: ${response.status} ${response.statusText}`);
  }
  const buffer = await response.arrayBuffer();
  return new Uint8Array(buffer);
}

export async function init({ wasmPath, wasmBytes, preopens = {}, args = [], env = {} } = {}) {
  const resolvedPreopens = { ...preopens };
  let wasmBinary = wasmBytes ? normalizeBytes(wasmBytes, 'wasmBytes') : null;

  if (!wasmPath && !wasmBinary) {
    throw createError(CLIENT_ERROR_SHAPE, 'wasmPath or wasmBytes is required');
  }

  if (isNodeEnvironment()) {
    const [wasiMod, fsPromises, pathMod, osMod] = await Promise.all([
      import('node:wasi'),
      import('node:fs/promises'),
      import('node:path'),
      import('node:os')
    ]);
    const { WASI } = wasiMod;
    const path = pathMod.default ?? pathMod;
    const os = osMod.default ?? osMod;
    if (!wasmBinary) {
      const readFile = fsPromises.readFile ?? (async (file) => {
        throw createError(CLIENT_ERROR_BYTES, 'fs.promises.readFile is unavailable');
      });
      wasmBinary = await readFile(wasmPath);
    }
    const hostVirtualRoot = await fsPromises.mkdtemp(path.join(os.tmpdir(), 'zkprov-'));
    if (!resolvedPreopens[VIRTUAL_ROOT]) {
      resolvedPreopens[VIRTUAL_ROOT] = hostVirtualRoot;
    }
    const wasi = new WASI({ args, env, preopens: resolvedPreopens, version: 'preview1' });
    const module = await WebAssembly.compile(wasmBinary);
    const instance = await WebAssembly.instantiate(module, {
      wasi_snapshot_preview1: wasi.wasiImport
    });
    wasi.initialize(instance);
    const fsHelper = createNodeFsHelper({
      rootGuest: VIRTUAL_ROOT,
      rootHost: resolvedPreopens[VIRTUAL_ROOT],
      fsPromises,
      path
    });
    return createApi(instance.exports, fsHelper);
  }

  const [{ WASI }] = await Promise.all([
    loadModule(['@wasmer/wasi', DEFAULT_POLYFILLS.wasi])
  ]);
  const [{ WasmFs }] = await Promise.all([
    loadModule(['@wasmer/wasmfs', DEFAULT_POLYFILLS.wasmfs])
  ]);
  const wasmFs = new WasmFs();
  if (!resolvedPreopens[VIRTUAL_ROOT]) {
    resolvedPreopens[VIRTUAL_ROOT] = VIRTUAL_ROOT;
  }
  const wasi = new WASI({
    args,
    env,
    preopens: resolvedPreopens,
    bindings: {
      ...WASI.defaultBindings,
      fs: wasmFs.fs
    }
  });
  if (!wasmBinary) {
    wasmBinary = await fetchWasmBytes(wasmPath);
  }
  const module = await WebAssembly.compile(wasmBinary);
  const imports = wasi.getImports(module);
  const instance = await WebAssembly.instantiate(module, imports);
  wasi.start(instance);
  const fsHelper = createMemFsHelper({ wasmFs, rootGuest: VIRTUAL_ROOT });
  return createApi(instance.exports, fsHelper);
}
