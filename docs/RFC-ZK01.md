# **RFC-ZK01: General-Purpose STARK Prover — v0.3**

---

## 1. Abstract

This RFC defines a **modular, backend-agnostic STARK proving engine** that allows developers to construct, prove, and verify algebraic constraint systems over configurable cryptographic backends.
It verifies algebraic relations only; **this engine does not execute programs or VM bytecode.**
It separates the high-level **AIR-IR** (Algebraic Intermediate Representation) from the low-level proving backends (Winterfell, Plonky2, Plonky3, native).
Developers can author custom bundles, public-input schemas, and parameter profiles while targeting any supported backend through a consistent API.
Backends are resolved by capability matching rather than global defaults, ensuring the adapter layer selects an implementation compatible with the requested field, hash, and FRI arity.
Version 0.3 introduces the **Backend Adapter Layer**, YAML-authored AIR definitions, determinism manifests, Golden Vector registry enforcement, and EVM interoperability gadgets while preserving backward-compatible semantics.

---

## 2. Roles

| Role                 | Responsibility                                                             |
| -------------------- | -------------------------------------------------------------------------- |
| **User / Developer** | Defines AIR, public inputs, and bundles; selects backend and profile.      |
| **Prover Engine**    | Coordinates trace build, constraint evaluation, and proof generation.      |
| **Verifier Engine**  | Reconstructs transcript and verifies proof deterministically.              |
| **Bundle Author**    | Creates reusable sub-AIR gadgets with typed ports and degree bounds.       |
| **Backend Adapter**  | Translates AIR-IR into backend-specific API (Winterfell, Plonky2/3, etc.). |
| **Binding Maintainer** | Keeps language bindings in sync with the C ABI; ships Node/TS, Python, Flutter/Dart, WASI packages and curates the DIY cookbook for Go, .NET, Java/Kotlin, and Swift. |
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
| `BACKEND`         | STARK implementation (`native`,`winterfell`,`plonky2`,`plonky3`) resolved via capability matching | capability-matched |
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

### 4.2 YAML AIR Definition

Programs MAY be authored in a human-readable `.yaml` schema that mirrors the `AirProgram` structs above.
YAML documents compile deterministically into canonical `.air` binaries using the CLI:

```bash
$ zkd compile balance.yml
```

Example minimal YAML:

```yaml
meta:
  name: balance_check
  field: Prime254
  hash: poseidon2
columns:
  trace_cols: 4
  const_cols: 1
constraints:
  transition_count: 2
  boundary_count: 1
```

The compiler lifts YAML into AIR-IR, performs validation, and emits bytecode-identical `.air` binaries for every platform.
No auxiliary state or randomness participates in this transformation, enabling reproducible downstream proofs.

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
| **C ABI & FFI** | Language ↔ Engine | `zkp_init()`, `zkp_prove(cfg)`, `zkp_verify(...)` etc.; wrapped by Node/TS, Python, Flutter/Dart, WASI bindings (Go/.NET/Java/Kotlin/Swift via DIY cookbook) |
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

AIR defines how user data binds into the algebraic trace.

Each `PublicDecl` can now specify a binding type:

| Binding | Description | Visibility |
| -------- | ------------ | -------- |
| `LiftAsConstCols` | Injects a constant vector into the trace | Public |
| `Absorb` | Hashed directly into the transcript | Public |
| `Pedersen(curve)` | Commits to hidden value `v` with blinding `r`; AIR enforces `C = v·H + r·G` | Public (Cx,Cy) |
| `PoseidonCommit` | Hash-based commitment for cheap hiding | Public scalar |
| `KeccakCommit` | Keccak256-based commitment for EVM interop | Public scalar |

Witness values that correspond to commitments (`v`, `r`) remain private.
All commitments are encoded deterministically, curve-checked, and bound to the proof transcript.

---

### 9.1 Pre-Baked Application Profiles

To accelerate developer adoption, the prover includes a library of **pre-baked application profiles**.
Each profile combines an AIR circuit with a declarative manifest of public inputs, gadgets, and limits, enabling developers to integrate common ZK workflows without writing AIR code.

Profiles are loaded via `zkd profile ls` or through the SDK:

```ts
import { prove } from "@zkd/sdk";
prove({ profile_id: "zk-auth-pedersen-secret", public_inputs: { C, nonce, origin } });
```

Each profile is:

* **Deterministic:** identical digest `D` across backends and bindings.
* **Composable:** gadgets and parameters editable for advanced users.
* **Documented:** manifests published in `profile-catalog.md`.

Initial profiles:

1. `zk-auth-pedersen-secret` — passwordless secret authentication
2. `zk-allowlist-merkle` — allowlist membership with replay protection
3. `zk-attr-range` — attribute range proof
4. `zk-balance-geq` — balance ≥ threshold attest
5. `zk-uniqueness-nullifier` — one-use nullifier per epoch
6. `zk-proof-of-solvency-lite` — assets vs liabilities delta commitment
7. `zk-vote-private` — private ballot from allowlist
8. `zk-file-hash-inclusion` — document inclusion proof
9. `zk-score-threshold` — reputation/score ≥ threshold
10. `zk-age-over` — mobile-optimized age gate

These adhere to the same validation, commitment, and digest-binding rules defined elsewhere in this RFC.

### 9.2 EVM Interop Gadgets

| Gadget | Description | Output Binding |
| ------ | ----------- | -------------- |
| `KeccakCommit` | Hashes transcript inputs with Keccak256 for Solidity parity | `bytes32` digest exposed to on-chain verifiers |
| `EvmLogProof` | Produces inclusion proof for canonical Ethereum log topics | ABI-encoded proof blob plus Merkle branch |
| `VerifierStub` | Minimal Solidity verifier that checks digest equality only | `bool` flag per proof manifest |

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
* **Determinism:** Proof determinism is defined at the algorithmic level. Given identical AIR, backend, profile, and public inputs, every honest build—regardless of compiler, platform, or binary—must emit the same transcript and final digest `D`. Binaries need not be bit-identical; reproducibility of outputs, not artifacts, is the invariant. All randomness is derived deterministically from transcript seeds, and floating-point operations are forbidden.
* **Integrity:** Merkle and FRI commitments binding.
* **Cross-backend parity:** Equivalent AIR + public inputs → proof verifies on all backends supporting same field/hash.
* **Isolation:** Adapters sandboxed; no network I/O during prove/verify. FFI bindings must propagate validation results without mutating proof bytes.
* **Side Channels:** Constant-time field ops in secure profile.
* **Versioning:** Program hash includes backend id to prevent mismatched proof reuse.

FFI bindings are required to forward `ValidationReport` objects and structured errors without alteration so that determinism and security guarantees hold independent of the host language.

### 12.1 Privacy & Commitment Security

* **Hiding:** Pedersen and Poseidon commitments hide the numeric witness; `r` must be sampled fresh per commitment.
* **Binding:** Each commitment is enforced by AIR constraints; reusing a blinding scalar triggers `BlindingReuse`.
* **Range Checking:** Range bundles ensure committed values fall within declared bit-widths.
* **EVM Compatibility:** `KeccakCommit` uses canonical Keccak256 encoding for digest parity with Solidity.
* **Isolation:** All commitment gadgets execute deterministically and offline; no randomness or external I/O.

### 12.2 Build Provenance

* **Build Provenance (Transparency Rule)**
  Implementations may distribute prebuilt binaries for convenience, but users are encouraged to build from source.
  Proof-level determinism is verifiable via golden vectors and validation reports, ensuring that locally built binaries produce the same digests as reference runs.
  Prebuilt artifacts are considered *non-canonical*.

### 12.3 Determinism Vector

Every proof exports a manifest capturing the provenance of the build inputs.
The manifest is hashed into the transcript and must be preserved verbatim for downstream verification.

```json
{
  "determinism_vector": {
    "compiler_commit": "0a1b2c3d",
    "backend": "native@0.0",
    "system": "linux-x86_64",
    "seed": "0001020304050607"
  }
}
```

Validation pipelines recompute the manifest hash and compare it against the proof header, guaranteeing deterministic provenance across hosts and bindings.

---

## 13. Testing & Verification

**Unit** — field math, FFT, FRI, Merkle, transcript.
**Integration** — end-to-end proof/verify for toy and mid AIRs per backend.
**Cross-Backend** — identical AIR verified under native & Winterfell.
**Negative** — tampered proof fails.
**Bench** — time/memory for (dev, balanced, secure) profiles × (backends).
**DoD:** All tests pass; deterministic transcripts; λ ≥ target; backend validator rejects invalid combos.

### 13.1 Golden Vector Registry

Golden vectors capture canonical transcripts and digests per AIR/back-end pairing.
Entries live under `/tests/golden_vectors/{program}/{backend}.json` and must agree byte-for-byte on the final digest `D`.
CI enforces parity by regenerating vectors for each backend and failing if any digest diverges.
Registry updates require explicit review with manifest hashes attached to prevent accidental drift.

---

## 14. Rationale and Summary

This revision positions the STARK engine as a universal proving platform: a stable IR front-end with swappable cryptographic backends.
Backends implement standard traits and declare capabilities, letting the engine match each proof request to a compatible implementation rather than relying on global defaults.
The proving core verifies algebraic relations exclusively—never full programs or VM bytecode—keeping the focus on deterministic constraint satisfaction.
This architecture preserves determinism and soundness while future-proofing for new FRI variants or hybrid SNARK wrappers.

**Design mantra:** *“One AIR to rule them all — backends as replaceable engines.”*

---

## 15. Roadmap

| Phase | Scope | Acceptance Gate |
| ----- | ----- | --------------- |
| Ph0 | YAML parser + deterministic proofs | `zkd compile` succeeds on reference AIRs and manifests embed determinism vector hashes |
| Ph1 | Golden Vector Registry | Cross-backend digests equal for all registry entries (CI enforced) |
| Ph2 | EVM Interop Gadgets | Solidity `VerifierStub` verifies Keccak digests emitted by off-chain prover |
| Ph3 | Profile Registry (community bundles) | Community bundles publish manifests with passing vector validation |
| Ph4 | zk-VM adapters (deferred) | Design review complete; adapters gated on deterministic manifest semantics |

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
