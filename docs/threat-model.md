# **Threat Model — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.3
**Purpose:** Enumerate adversarial capabilities, security assumptions, and mitigations across the proving stack.

---

## 1. Assets & Adversaries

* **Assets:** AIR definitions, determinism manifests, proof digests, profile manifests, backend capability registry.
* **Adversaries:** Malicious provers attempting transcript tampering, compromised build pipelines, and verifiers receiving manipulated manifests.
* **Trust Assumptions:** Deterministic transcript, authenticated registry updates, and reproducible build artifacts.

---

## 2. Integrity

Proof integrity relies on deterministic pipelines: any tampering with AIR, manifest, or backend selection is detected via digest comparison and determinism vector hashing.
Golden Vector CI enforces cross-backend equality, providing an anti-tampering tripwire that catches compromised binaries even when they emit syntactically valid proofs.

### 2.1 Commitment Gadgets

All commitment gadgets (Pedersen, PoseidonCommit, KeccakCommit) bind public outputs to trace cells; malformed commitments trigger constraint failures.

### 2.2 Manifest Validation

The Determinism Vector captures `compiler_commit`, `backend`, `system`, and `seed`.
Verifiers recompute the manifest hash and reject any proof whose manifest diverges from the expected digest, preventing replay or provenance spoofing.

---

## 3. Availability

* Backend adapters are sandboxed; DoS via long-running computations is mitigated by capability-aware scheduling.
* CLI exposes `--profile` tuning to trade off execution time versus security.

---

## 4. Privacy

Private inputs remain off-chain; only commitments and deterministic digests surface to verifiers.
Commitment gadgets (`Pedersen`, `Poseidon`, `KeccakCommit`) hide witness data while yielding deterministic outputs for backend parity.
No witness values are serialized beyond required commitments.

---

## 5. Supply Chain

* Determinism manifests make malicious compiler/toolchain modifications observable.
* Golden Vector registry acts as reproducibility oracle across hosts.

---

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
