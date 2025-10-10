# **EVM Interoperability Path**

**Parent RFC:** RFC-ZK01 §9.2

---

## 1. Workflow Overview

```
off-chain prove → proof.json (determinism vector) → digest_D → Solidity VerifierStub
```

1. Run `zkd prove --manifest proofs/balance.proof.json` to produce proof bytes and determinism manifest.
2. Compute `digest_D` via Keccak encoding of header/body (automatically emitted by CLI).
3. Submit digest + manifest to Solidity `VerifierStub` for replay.

---

## 2. Solidity Stub

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract VerifierStub {
    event ProofAccepted(bytes32 digest, string backend);

    function verify(bytes32 digest, bytes32 expected, string calldata backend) external returns (bool) {
        require(digest == expected, "DIGEST_MISMATCH");
        emit ProofAccepted(digest, backend);
        return true;
    }
}
```

The contract compares digests only; determinism manifests remain off-chain for auditors.

---

## 3. Foundry Test Snippet

```solidity
function testDigestParity() public {
    bytes32 digest = 0x1234...; // from proof.json
    bytes32 expected = 0x1234...;
    VerifierStub stub = new VerifierStub();
    bool ok = stub.verify(digest, expected, "native@0.0");
    assertTrue(ok);
}
```

Proof manifests should be loaded from `proofs/*.proof.json` during the test setup.

---

## 4. ABI Packing Details

`digest_D` uses Solidity ABI encoding of:

```text
(uint64 backendIdHash, uint64 profileIdHash, uint64 pubioHash, uint64 bodyLen, bytes body)
```

* Integers encoded as 32-byte big-endian words.
* `body` prefixed with length and padded to 32-byte boundary.
* Final Keccak256 digest matches CLI output (`proofs/*.digest.hex`).

---

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
