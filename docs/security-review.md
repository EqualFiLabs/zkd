# **Security Review Checklist — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.3
**Purpose:** Consolidate review items before releases or significant architectural changes.

---

## 1. Checklist

- [ ] Configuration validation rejects unsupported field/hash/profile combinations.
- [ ] AIR schema validated and deterministic under YAML compiler.
- [ ] Proof manifest includes Determinism Vector and passes Golden Vector CI.
- [ ] Parity verification across all supported backends recorded and signed off.
- [ ] Commitment gadgets (`Pedersen`, `PoseidonCommit`, `KeccakCommit`) audited for curve/hash correctness.
- [ ] EVM interoperability path validated against Solidity `VerifierStub`.

---

## 2. Review Notes

Capture manifest hashes, backend digests, and vector validation logs for every release candidate.
Escalate any divergence to incident response and block release until parity is restored.

---

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
