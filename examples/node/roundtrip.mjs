import path from 'node:path';
import { fileURLToPath } from 'node:url';
import zk from '@zkprov/node';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const airPath = path.resolve(__dirname, '..', 'air', 'toy.air');

const cfg = {
  backendId: 'native@0.0',
  field: 'Prime254',
  hashId: 'blake3',
  friArity: 2,
  profileId: 'balanced',
  airPath,
  publicInputsJson: JSON.stringify({ demo: true, n: 7 }),
};

const backs = await zk.listBackends();
const backendIds = backs.map((backend) => backend?.id).filter((id) => Boolean(id));
console.log('backends:', backendIds);

const { proof, meta } = await zk.prove(cfg);
console.log('D=', meta.digest, 'len=', meta.proof_len);

const vr = await zk.verify(cfg, proof);
console.log('verified=', vr.verified, 'D2=', vr.meta.digest);

if (!vr.verified) {
  process.exit(1);
}
