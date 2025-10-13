import { init } from './loader.js';
import { access } from 'node:fs/promises';
import { resolve } from 'node:path';

const wasmPath = resolve('./bindings/wasm/zkprov_wasi.wasm');

// Copy or ensure example AIR is visible from WASI’s /work
const airPathHost = resolve('./examples/air/toy.air');
await access(airPathHost);

const z = await init({ wasmPath, preopens: { '/work': '.' } });
const backends = await z.listBackends();
console.log('backends:', backends);

const cfg = {
  backendId: 'native@0.0',
  field: 'Prime254',
  hashId: 'blake3',
  friArity: 2,
  profileId: 'balanced',
  airPath: '/work/examples/air/toy.air',
  publicInputsJson: '{"demo":true,"n":7}'
};

const { proof, meta } = await z.prove(cfg);
console.log('D=', meta.digest, 'len=', meta.proof_len);

const vr = await z.verify(cfg, proof);
console.log('verified=', vr.verified, 'D=', vr.meta.digest);
if (!vr.verified) process.exit(1);
