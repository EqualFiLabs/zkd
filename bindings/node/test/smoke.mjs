import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import zkprov from '../index.js';

const { listBackends, prove, verify } = zkprov;

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function createTestRunner() {
  const tests = [];

  function test(name, fn) {
    tests.push({ name, fn });
  }

  async function run() {
    let failed = false;

    for (const { name, fn } of tests) {
      try {
        await fn();
        console.log(`ok - ${name}`);
      } catch (err) {
        failed = true;
        console.error(`not ok - ${name}`);
        console.error(err);
      }
    }

    if (failed) {
      process.exitCode = 1;
    }
  }

  return { test, run };
}

const { test, run } = createTestRunner();

function resolveToyAirPath() {
  return path.resolve(__dirname, '../../../examples/air/toy.air');
}

function assertDigestShape(value) {
  assert.equal(typeof value, 'string', 'digest must be a string');
  assert.equal(value.length, 66, 'digest must be 66 characters (0x + 64 hex)');
  assert.match(value, /^0x[0-9a-f]{64}$/);
}

test('listBackends exposes the native backend', async () => {
  const backends = await listBackends();
  assert.ok(Array.isArray(backends), 'listBackends should return an array');
  const nativeBackend = backends.find((backend) => backend && backend.id === 'native@0.0');
  assert.ok(nativeBackend, 'expected native@0.0 backend');
});

test('prove and verify toy AIR', async () => {
  const airPath = resolveToyAirPath();
  const config = {
    backendId: 'native@0.0',
    field: 'Prime254',
    hashId: 'blake3',
    friArity: 2,
    profileId: 'balanced',
    airPath,
    publicInputsJson: '{"demo":true,"n":7}',
  };

  const proveResult = await prove(config);
  assert.ok(Buffer.isBuffer(proveResult.proof), 'prove() should return a Buffer proof');
  assert.ok(proveResult.proof.length > 0, 'proof should not be empty');
  assert.ok(proveResult.meta && typeof proveResult.meta === 'object', 'meta should be an object');
  assertDigestShape(proveResult.meta.digest);
  assert.equal(
    proveResult.meta.proof_len,
    proveResult.proof.length,
    'meta.proof_len should match proof length',
  );

  const verifyResult = await verify(config, proveResult.proof);
  assert.equal(verifyResult.verified, true, 'verify() should resolve to verified=true');
  assert.ok(verifyResult.meta && typeof verifyResult.meta === 'object', 'verify meta should be object');
  assertDigestShape(verifyResult.meta.digest);
  assert.equal(
    verifyResult.meta.digest,
    proveResult.meta.digest,
    'verify digest should match prove digest',
  );
});

await run();
