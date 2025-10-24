import zk from '@zkprov/node';

const cfg = {
  backendId: 'native@0.0',
  field: 'Prime254',
  hashId: 'blake3',
  friArity: 2,
  profileId: 'balanced',
  airPath: 'examples/air/toy.air',
  publicInputsJson: JSON.stringify({ demo: true, n: 7 }),
};

const backs = await zk.listBackends();
console.log('backends:', Object.keys(backs));

const { proof, meta } = await zk.prove(cfg);
console.log('D=', meta.digest, 'len=', meta.proof_len);

const vr = await zk.verify(cfg, proof);
console.log('verified=', vr.verified, 'D2=', vr.meta.digest);

if (!vr.verified) {
  process.exit(1);
}
