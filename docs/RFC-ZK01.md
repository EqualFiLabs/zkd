# **RFC-ZK01: General-Purpose STARK Prover — v0.2**

---

## 1. Abstract

This RFC defines a **modular, backend-agnostic STARK proving engine** that allows developers to construct, prove, and verify algebraic constraint systems over configurable cryptographic backends.
It separates the high-level **AIR-IR** (Algebraic Intermediate Representation) from the low-level proving backends (Winterfell, Plonky2, Plonky3, native).
Developers can author custom bundles, public-input schemas, and parameter profiles while targeting any supported backend through a consistent API.
Version 0.2 introduces the **Backend Adapter Layer**, enabling portable proofs across multiple STARK implementations without rewriting programs.

---

## 2. Roles

| Role                 | Responsibility                                                             |
| -------------------- | -------------------------------------------------------------------------- |
| **User / Developer** | Defines AIR, public inputs, and bundles; selects backend and profile.      |
| **Prover Engine**    | Coordinates trace build, constraint evaluation, and proof generation.      |
| **Verifier Engine**  | Reconstructs transcript and verifies proof deterministically.              |
| **Bundle Author**    | Creates reusable sub-AIR gadgets with typed ports and degree bounds.       |
| **Backend Adapter**  | Translates AIR-IR into backend-specific API (Winterfell, Plonky2/3, etc.). |
| **Profile Registry** | Maintains security parameter presets (dev, balanced, secure).              |

---

## 3. Policy Invariants

| Constant          | Description                                                      | Default        |
| ----------------- | ---------------------------------------------------------------- | -------------- |
| `λ_min`           | Minimum security bits                                            | 80             |
| `FIELD_PRIME`     | Canonical field modulus                                          | 2²⁵⁴ − 2³¹ + 1 |
| `MERKLE_ARITY`    | Supported tree arities                                           | { 2, 4 }       |
| `PROFILE`         | Preset parameter bundle                                          | balanced       |
| `HASH_FN`         | Poseidon2 default; Rescue/Blake3 optional                        | Poseidon2      |
| `BACKEND`         | STARK implementation (`native`,`winterfell`,`plonky2`,`plonky3`) | native         |
| `CONST_COL_LIMIT` | Max lifted const/public columns                                  | 64             |
| `MAX_DEGREE`      | Global polynomial degree bound                                   | 64             |

### Compatibility Gate

Each backend publishes a capability matrix (fields, hashes, FRI arity, recursion).
Configurations failing `BACKEND.capabilities` are rejected before execution.

---

## 4. State Model

```text
Dataset Program {
  id: UUID,
  name: string,
  version: string,
  air_ir: bytes,
  public_schema: PublicDecl[],
  bundles: BundleSpec[],
  backend_id: string,
  created_at: timestamp
}

Dataset Backend {
  id: string,                 // "winterfell@0.6"
  fields: FieldId[],
  hashes: HashId[],
  fri: { arities: int[], max_depth: int, grinding: bool },
  recursion: enum<none, stark-in-stark, snark-wrapper>,
  lookups: bool
}

Dataset Profile {
  id: string,                 // "dev-fast","balanced","secure"
  blowup: int,
  fri_queries: int,
  grind_bits: int,
  merkle_arity: int
}

Dataset Proof {
  program_id: UUID,
  backend_id: string,
  profile_id: string,
  proof_bytes: bytes,
  stats: ProofStats,
  created_at: timestamp
}
```

### 4.1 Structs / Schemas

```text
PublicDecl {
  name: string,
  type: enum<Scalar, Vector, Bytes>,
  len: int?,
  binding: enum<Absorb, LiftAsConstCols>,
  visibility: enum<PublicInput, PublicOutput>
}

BundleSpec {
  name: string,
  ports: { in: Port[], out: Port[] },
  degree_bound: int,
  generator: fn(trace, inputs) -> cols
}

Capabilities {
  fields: FieldId[],
  hashes: HashId[],
  recursion: bool,
  lookups: bool
}
```

---

## 5. Primary Flow A — Proof Generation

### 5.1 `build_trace()`

Inputs: AIR-IR, bundle registry, public inputs.
Preconditions: schema valid, no unresolved columns.
Steps: allocate columns, inject CONST cols, run bundle generators, populate trace.
Output: complete trace matrix.

### 5.2 `prove()`

Inputs: trace, AIR-IR, profile, backend, public inputs.
Steps:

1. Resolve backend adapter via registry.
2. Adapter compiles AIR → BackendProgram.
3. Compute constraints → composition poly.
4. Commit Merkle roots; absorb public inputs into transcript.
5. Run FRI/LDT through backend API.
   Outputs: `ProofBlob`, `ProofStats`.

---

## 6. Primary Flow B — Proof Verification

### 6.1 `verify()`

Inputs: Program, backend, profile, proof, public inputs.
Steps: load adapter → rebuild transcript → verify Merkle + FRI layers → check constraints.
Output: `verified: bool`.

---

## 7. Integration (APIs / Interfaces)

| Interface       | Direction     | Shape                                                                             |
| --------------- | ------------- | --------------------------------------------------------------------------------- |
| CLI             | User ↔ Engine | `zkd prove -p program.air -i inputs.json --backend winterfell --profile balanced` |
| Rust SDK        | Library       | `prove(program, trace, pub_in, profile, backend)` / `verify(...)`                 |
| Backend Trait   | Internal      | `ProverBackend` / `VerifierBackend`                                               |
| Config Schema   | JSON/TOML     | validated params (field, hash, profile, backend)                                  |
| Bundle Registry | File/Dir      | `*.bundle` describing ports + generator                                           |
| Verifier Output | JSON          | version, λ, backend, elapsed_ms, mem_bytes                                        |

### Backend Traits

```rust
pub trait ProverBackend {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;
    fn compile(&self, air: &AirIR, bundles: &BundleSet) -> BackendProgram;
    fn prove(&self, prog: &BackendProgram, pub_io: &PublicIO, profile: &Profile) -> ProofBlob;
}
pub trait VerifierBackend {
    fn verify(&self, prog: &BackendProgram, pub_io: &PublicIO, proof: &ProofBlob, profile: &Profile) -> bool;
}
```

---

## 8. Deterministic Derivations / Addressing

All IDs derived as BLAKE3(`program_name || version || backend_id || profile_id`).
Proof files: `proofs/{program_id}-{backend_id}-{profile_id}.proof`.
Bundle namespaces: `bundle::{name}@{hash(code)}`.

---

## 9. Public Inputs / Data Binding

AIR defines `PublicDecl`s.

* `LiftAsConstCols`: injected as read-only CONST columns.
* `Absorb`: hashed into transcript pre-challenge.
  Canonical encoding (field LE or fixed bytes) shared across backends.
  Verifier must mirror absorb order.
  Public outputs (optional) re-computed for equivalence.

---

## 10. Reward / Fee / Allocation Rules

Not applicable for local execution. Cloud deployments may later define billing per proof-time.

---

## 11. Events

| Event               | Trigger              | Payload                                                  |
| ------------------- | -------------------- | -------------------------------------------------------- |
| `ProofGenerated`    | Proof completed      | `{program_id, backend, profile, size_bytes, elapsed_ms}` |
| `ProofVerified`     | Verification success | `{program_id, backend, verified}`                        |
| `BackendRegistered` | Adapter added        | `{id, capabilities}`                                     |
| `ProfileRegistered` | Profile added        | `{id, params}`                                           |

All events logged JSONL + stdout.

---

## 12. Security & Privacy Considerations

* **Soundness:** Each backend must guarantee λ ≥ declared target.
* **Determinism:** Transcripts stable given same backend + profile.
* **Integrity:** Merkle and FRI commitments binding.
* **Cross-backend parity:** Equivalent AIR + public inputs → proof verifies on all backends supporting same field/hash.
* **Isolation:** Adapters sandboxed; no network I/O during prove/verify.
* **Side Channels:** Constant-time field ops in secure profile.
* **Versioning:** Program hash includes backend id to prevent mismatched proof reuse.

---

## 13. Testing & Verification

**Unit** — field math, FFT, FRI, Merkle, transcript.
**Integration** — end-to-end proof/verify for toy and mid AIRs per backend.
**Cross-Backend** — identical AIR verified under native & Winterfell.
**Negative** — tampered proof fails.
**Bench** — time/memory for (dev, balanced, secure) profiles × (backends).
**DoD:** All tests pass; deterministic transcripts; λ ≥ target; backend validator rejects invalid combos.

---

## 14. Rationale and Summary

This revision positions the STARK engine as a universal proving platform: a stable IR front-end with swappable cryptographic backends.
Backends implement standard traits and declare capabilities, letting users choose Winterfell for simplicity, Plonky2/3 for recursion, or native for experimentation.
This architecture preserves determinism and soundness while future-proofing for new FRI variants or hybrid SNARK wrappers.

**Design mantra:** *“One AIR to rule them all — backends as replaceable engines.”*

---
