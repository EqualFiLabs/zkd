---

# **Architecture — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.2
**Purpose:** Describe the logical, data, and control-flow architecture of the multi-backend STARK proving system.
**Status:** Draft → Stable after Phase 0 implementation.

---

## 1. Overview

The proving engine is structured as a **layered stack**:

```
┌──────────────────────────────────────────────────────────────┐
│                         CLI / SDK                            │
│      (zkd, Rust bindings, JSON configs, profiles)             │
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
   The coordinator loads the backend registry → checks that
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

---

## 3. Component Boundaries

| Layer               | Responsibility                                     | Key Interfaces                       |
| ------------------- | -------------------------------------------------- | ------------------------------------ |
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
