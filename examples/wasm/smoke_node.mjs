import { resolve } from 'node:path';
import { init } from '../../bindings/wasm/loader.js';

const wasmPath = resolve('bindings/wasm/zkprov_wasi.wasm');
const repoRoot = resolve('.');
const z = await init({ wasmPath, preopens: { '/work': repoRoot } });

const cfg = {
  backendId: 'native@0.0',
  field: 'Prime254',
  hashId: 'blake3',
  friArity: 2,
  profileId: 'balanced',
  airPath: '/work/examples/air/toy.air',
  publicInputsJson: JSON.stringify({ demo: true, n: 7 })
};

const backs = await z.listBackends(); console.log('backs', backs);
const { proof, meta } = await z.prove(cfg); console.log('D=', meta.digest);
const { verified } = await z.verify(cfg, proof);
console.log('verified=', verified);
if (!verified) process.exit(1);
