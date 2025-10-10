---

# **Architecture — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.3
**Purpose:** Describe the logical, data, and control-flow architecture of the multi-backend STARK proving system.
**Status:** Draft → Stable after Phase 0 implementation.

---

## 1. Overview

The proving engine is structured as a **layered stack**:

```
┌──────────────────────────────────────────────────────────────┐
│                      CLI / SDK / FFI                         │
│ (zkd CLI, Rust SDK, JSON configs, profiles, C ABI + bindings) │
│   • Bindings: Node/TS, Python, Go, .NET, Swift/iOS, WASI      │
├──────────────────────────────────────────────────────────────┤
│                    Coordinator Layer                          │
│   • Loads AIR-IR + Bundles                                   │
│   • Validates Profile + Backend Capabilities                 │
│   • Orchestrates trace build / prove / verify flows          │
├──────────────────────────────────────────────────────────────┤
│             Algebraic Intermediate Representation (AIR-IR)    │
│   • Columns, Constraints, Periodics, Public IO                │
│   • Portable across backends                                 │
│   • Emits Backend-neutral IR                                 │
├──────────────────────────────────────────────────────────────┤
│                  Bundle Composition Engine                    │
│   • Library of reusable gadgets (Poseidon, Range Check, etc.) │
│   • Port wiring & degree bound checks                         │
│   • Produces unified constraint set                           │
├──────────────────────────────────────────────────────────────┤
│                Backend Adapter Layer (new in v0.2)            │
│   • Translates AIR-IR → backend-native program                │
│   • Implements ProverBackend / VerifierBackend                │
│   • Examples: native | Winterfell | Plonky2 | Plonky3         │
├──────────────────────────────────────────────────────────────┤
│               Algebraic & Cryptographic Core                  │
│   • Field FFTs, FRI reductions, Merkle commitments            │
│   • Fiat–Shamir Transcript, Random Oracle                     │
│   • Deterministic serialization                               │
└──────────────────────────────────────────────────────────────┘
```

**Design Pillars**

1. Determinism → identical inputs = identical outputs, validated at the proof layer independent of binary reproducibility.
2. Portability → same AIR across all backends.
3. Extensibility → new backends, new hashes, zero rewrites.
4. Transparency → open validation reports, golden vectors, reproducible benches.

Authoring flows follow a single deterministic path: human-readable [YAML AIR definitions](./air-yaml.md) compile into canonical AIR-IR, adapters lower the IR into backend programs, and proof emission records a manifest plus determinism vector for downstream consumers.

---

## 2. Data-Flow Summary

### Prove Path

1. **Input Stage**
   User provides:

   * AIR-IR (`program.air`)
   * Public inputs (`inputs.json`)
   * Profile (`balanced`)
   * Backend selection (`winterfell`)

2. **Validation**
   The coordinator loads the backend registry, resolves a backend via capability matching, and checks that
   `(field, hash, fri, arity)` ∈ `backend.capabilities`.

3. **Trace & Constraint Build**
   Bundles populate columns → AIR-IR emits canonical form.

4. **Backend Compile**
   Adapter lowers AIR-IR into backend-native structures
   (transition fns, periodic cols, constants).

5. **Proving Phase**
   Backend executes FRI/Merkle/Transcript to produce a proof blob.

6. **Emission**
   Proof and stats written to `proofs/{program_id}-{backend}-{profile}.proof`.

### Verify Path

1. Coordinator re-hydrates AIR-IR, public inputs, backend adapter.
2. Adapter re-computes transcript absorptions and verifies backend-specific proof object.
3. Deterministic verdict: `verified = true | false`.

### Golden Vector CI

The validation surface integrates a [Golden Vector Registry](./golden-vectors.md).
Every CI run regenerates reference proofs for each backend, compares digests, and marks `vector_passed = true` only when all capability-compatible adapters produce identical outputs.
Proof artifacts record the determinism vector manifest, and reports embed the manifest hash so bindings can assert provenance.

### EVM Interop Path

For on-chain consumers, the coordinator exports proof blobs, determinism manifests, and Keccak digests described in [EVM Interop Gadgets](./evm-interop.md).
Off-chain provers submit `proof.json` containing the determinism vector, while Solidity stubs consume the digest and emit parity checks via `VerifierStub`.

---

## 3. Component Boundaries

| Layer               | Responsibility                                     | Key Interfaces                       |
| ------------------- | -------------------------------------------------- | ------------------------------------ |
| **CLI / SDK / FFI** | User entrypoints; exposes C ABI & language bindings | `zkd`, `zkp_*` functions, language SDKs |
| **Coordinator**     | Lifecycle mgmt, config validation, event logging   | `Prover::run()`, `Verifier::run()`   |
| **AIR-IR**          | Algebraic definitions portable across backends     | `AirProgram`, `Constraint`, `Column` |
| **Bundle Engine**   | Reusable gadgets; composition + degree enforcement | `BundleSpec`, `BundleRegistry`       |
| **Backend Adapter** | IR → backend translation, proof orchestration      | `ProverBackend`, `VerifierBackend`   |
| **Crypto Core**     | Field FFTs, FRI, hash, merkle, transcript          | `Field`, `MerkleTree`, `Transcript`  |

---

## 4. Backend Adapter Design

### Adapter Trait

```rust
trait ProverBackend {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;
    fn compile(&self, air: &AirIR, bundles: &BundleSet) -> BackendProgram;
    fn prove(&self, prog: &BackendProgram, pub_io: &PublicIO, profile: &Profile) -> ProofBlob;
}
```

Each adapter registers itself into a global `BackendRegistry`:

```rust
BackendRegistry::register(Box::new(WinterfellBackend::new()));
BackendRegistry::register(Box::new(Plonky2Backend::new()));
```

### Capability Matrix (excerpt)

| Backend            | Fields                | Hashes                      | FRI Arities | Recursion                   | Lookups |
| ------------------ | --------------------- | --------------------------- | ----------- | --------------------------- | ------- |
| **Native**         | 254-bit prime         | Poseidon2 / Rescue / Blake3 | {2,4}       | none                        | false   |
| **Winterfell 0.6** | BabyBear / Goldilocks | Poseidon2 / Rescue          | {2,4,8}     | none                        | false   |
| **Plonky2 0.2.x**  | Goldilocks            | Poseidon2                   | dynamic     | stark-in-stark              | true    |
| **Plonky3 0.1.x**  | Goldilocks            | Poseidon2                   | dynamic     | stark-in-stark / snark-wrap | true    |

---

## 5. Configuration & Profiles

Each proof run references:

```toml
backend = "winterfell@0.6"
profile = "balanced"
field   = "Goldilocks"
hash    = "poseidon2"

[advanced]
fri_blowup  = 16
fri_queries = 30
grind_bits  = 18
```

**Profile registry** supplies defaults meeting the target λ.
Adapters may internally adjust these to satisfy backend constraints.

---

## 6. Public Inputs & Outputs

AIR-IR exposes `PublicDecl`s (scalars, vectors, bytes).
Adapters must:

1. Map lifted constants into backend constant wires.
2. Absorb committed inputs into transcript pre-challenge.
3. Guarantee deterministic encoding.

All public outputs (e.g., Merkle roots) are re-computed and returned for verification equality.

---

## 7. Event Model

| Event               | Layer       | Emitted When         | Payload                                        |
| ------------------- | ----------- | -------------------- | ---------------------------------------------- |
| `BackendRegistered` | Core        | Adapter registration | `{id, capabilities}`                           |
| `ProofGenerated`    | Coordinator | Prover completes     | `{program_id, backend, elapsed_ms, mem_bytes}` |
| `ProofVerified`     | Coordinator | Verifier success     | `{program_id, backend, verified}`              |

All events append to `events.jsonl` for observability.

---

## 8. Security Envelope

* **λ guarantee**: Each backend must reach ≥ 80 bits under its chosen parameters.
* **Deterministic transcript**: includes `(backend_id, profile_id)` to prevent replay.
* **Isolation**: adapters execute pure compute; no I/O side channels.
* **Cross-backend tests**: same AIR + inputs must yield identical final public outputs.
* **Versioned program hashes**: proof reuse across backends impossible.

---

## 9. Build & Packaging Layout

```
/src
  /air          → IR definitions
  /bundles      → reusable gadgets (range, commit, arith)
  /backend
     native/
     winterfell/
     plonky2/
     plonky3/
  /crypto       → field, FRI, hash, merkle, keccak, poseidon, pedersen
  /evm          → ABI + digest helpers
  /cli          → zkd command tool
/include        → generated C header (zkprov.h)
/bindings       → language bindings (node/, python/, go/, dotnet/, swift/, wasm/)
/libs           → compiled shared libraries (libzkprov.so, .dylib, .dll, .wasm)
/docs
  rfc.md
  architecture.md
  interfaces.md
  roadmap.md
/tests
  e2e/
  golden_vectors/
```

Each backend compiles as an independent crate/module, imported via the registry at runtime.

Prebuilt artifacts are optional; reproducibility at the proof layer is mandatory.

---

## 10. Future Extensions

* **GPU Acceleration Layer** — optional FRI kernels.
* **Recursive Proof Aggregator** — via Plonky3 or SNARK wrapper.
* **On-chain Verifier Generator** — EVM / Cairo codegen.
* **AIR Optimizer** — static degree reduction and bundle fusion.
* **Deterministic Bench Suite** — cross-backend reproducibility metrics.

---

## 11. Design Rationale

A hard separation between **AIR authoring** and **backend implementation** maximizes portability and future-proofing.
Winterfell and Plonky families evolve rapidly; by pinning a common IR and schema-verified profiles, proofs remain reproducible even as libraries diverge.
This modular model mirrors the UNIX philosophy: small, composable units — algebraic front-end, interchangeable engines.

The cryptographic and privacy modules unify proof commitments, enabling native interoperability with EVM and deterministic hiding for private inputs without breaking cross-backend determinism.

**Mantra:** *“Change the engine, keep the math.”*

---

## 12. Application Profiles & Use Cases

Although the core prover exposes a fully general AIR and bundling system, most developers will integrate through **pre-baked application profiles**.
Each profile is a complete circuit (AIR + manifest + input schema) bundled with a human-readable identifier such as `zk-auth-pedersen-secret` or `zk-proof-of-solvency-lite`.
Profiles are distributed with the core and surfaced through both the CLI (`zkd profile ls`) and the SDK (`load_profile(id)`).

**Goals**

* Accelerate integration by offering common zero-knowledge workflows as drop-in circuits.
* Maintain full composability: advanced users can fork or extend profiles by editing manifests.
* Guarantee deterministic results across backends and bindings.

**Examples**

| Profile ID | Purpose | Core Gadget Bindings | Typical Public Inputs |
| ----------- | -------- | -------------------- | --------------------- |
| `zk-auth-pedersen-secret` | Knowledge of secret for commitment `C` (auth/login) | PedersenCommit, PoseidonBind | `C`, `nonce`, `origin`, `nullifier` |
| `zk-allowlist-merkle` | Membership in Merkle set with challenge binding | MerklePathVerify, PoseidonBind | `root`, `pk_hash`, `path`, `nonce`, `origin` |
| `zk-attr-range` | Attribute within `[min,max]` without revealing value | RangeCheck, PedersenCommit | `commitment`, `min`, `max` |
| `zk-balance-geq` | Balance ≥ threshold without disclosing balance | RangeCheck, PedersenCommit | `commitment`, `threshold`, `adapter_proof` |
| `zk-uniqueness-nullifier` | One-use nullifier per epoch | PoseidonNullifier, PoseidonBind | `nullifier`, `epoch` |
| `zk-proof-of-solvency-lite` | Assets ≥ liabilities commitment delta | MerklePathVerify, PedersenCommit, RangeCheck | `asset_root`, `liability_root`, `delta_commitment` |
| `zk-vote-private` | Single private ballot from allowlist | MerklePathVerify, PedersenCommit, PoseidonBind | `root`, `vote_commitment`, `nonce`, `tally_binding` |
| `zk-file-hash-inclusion` | Inclusion of file hash in committed set | MerklePathVerify, PoseidonBind | `root`, `file_hash`, `path` |
| `zk-score-threshold` | Score ≥ threshold with epoch binding | PedersenCommit, RangeCheck | `commitment`, `threshold`, `epoch` |
| `zk-age-over` | Age bound proof optimized for mobile | RangeCheckLite, PedersenCommit | `commitment`, `bound` |

**Presets**

Each profile ships multiple presets (`fast`, `balanced`, `tight`, `mobile`) which internally map to standard performance profiles from `/profiles/*.toml`.
Developers can override presets at runtime using the same CLI/SDK flags as custom AIRs.

**Extensibility**

* All profiles follow the same manifest schema as user-defined AIRs.
* Gadgets remain swappable — e.g., replace Pedersen with PoseidonCommit by editing the manifest.
* Deterministic digests (`D`) are preserved across profile revisions.

**Output**

Profiles are versioned independently (e.g., `zk-auth-pedersen-secret@1.0.0`) and validated on CI alongside the core proof suite.

---

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
