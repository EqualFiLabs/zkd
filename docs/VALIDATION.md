---

# **Validation — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.2
**Purpose:** Define how correctness is validated at every layer — configuration, AIR, constraint satisfaction, proof integrity, and backend conformance.
**Status:** Stable post Phase 0 CI green.

---

## 1. Overview

Validation occurs in four domains:

1. **Configuration Validation** — ensures chosen `field`, `hash`, `profile`, `backend` are mutually compatible.
2. **AIR Validation** — checks the algebraic program’s internal consistency.
3. **Prover Runtime Validation** — enforces constraint satisfaction and proof soundness.
4. **Verifier Validation** — reconstructs commitments and rejects any mismatch.

Every run produces a deterministic `ValidationReport` object (see § 7).

---

## 2. Configuration Validation

| Check         | Condition                                        | Error                    | Recovery               |
| ------------- | ------------------------------------------------ | ------------------------ | ---------------------- |
| Field Support | `field ∈ backend.capabilities.fields`            | `CapabilityMismatch`     | Abort                  |
| Hash Support  | `hash ∈ backend.capabilities.hashes`             | `CapabilityMismatch`     | Abort                  |
| Profile Fit   | λ(profile) ≥ λ_min                               | `WeakSecurityProfile`    | Suggest higher profile |
| FRI Arity     | profile.arity ∈ backend.capabilities.fri_arities | `InvalidFRIConfig`       | Abort                  |
| Const Cols    | count ≤ CONST_COL_LIMIT                          | `TraceShapeError`        | Abort                  |
| Bundle Degree | Σ(degree) ≤ MAX_DEGREE                           | `DegreeOverflow`         | Abort                  |
| Curve Support | `curve ∈ backend.capabilities.curves`            | `CapabilityMismatch`     | Abort                  |
| Pedersen Enabled | `backend.capabilities.pedersen`               | `PedersenConfigMismatch` | Abort                  |
| Keccak Enabled | `backend.capabilities.keccak`                   | `KeccakUnavailable`      | Abort                  |

Config validation always precedes compilation; failures never defer to runtime.

---

## 3. AIR Validation

Performed by the **AIR-IR compiler** before any backend code runs.

### 3.1 Structural Checks

* All column indices unique.
* Boundary constraints reference valid row positions.
* No circular dependency between bundles.
* Periodic columns’ length divides trace length.
* Public inputs tagged either `Absorb` or `LiftAsConstCols` (not both).

### 3.2 Type Consistency

* Constraint expressions typed over field elements only.
* Public inputs lifted to const cols must match declared type.
* Bytes→Field conversions explicitly through hash-to-field bundles.

### 3.3 Degree Accounting

For every constraint `f(x₁,…,xₙ) = 0`:

```
deg(f) ≤ MAX_DEGREE
```

If violated → `DegreeOverflow`.

---

## 4. Prover Runtime Validation

Executed during `prove()` after trace build but before FRI.

| Check                 | Description                                                                | Failure                 |
| --------------------- | -------------------------------------------------------------------------- | ----------------------- |
| Constraint Evaluation | All transition + boundary polynomials evaluate to 0 mod p within tolerance | `ConstraintUnsatisfied` |
| Trace Length          | Power-of-two required for FFT                                              | `InvalidTraceLength`    |
| FRI Parameter Bounds  | `blowup ≥ 2`, `queries ≥ log₂(trace)`                                      | `InvalidFRIConfig`      |
| Merkle Commitments    | Root non-zero and unique per column set                                    | `CommitmentError`       |
| Transcript Seed       | Non-zero hash after absorbing public inputs                                | `TranscriptError`       |
| Proof Integrity       | Final object matches declared header length                                | `SerializationError`    |
| Point Validity        | Pedersen point lies on selected curve                                      | `InvalidCurvePoint`     |
| Blinding Reuse        | Same blinding `r` reused across commitments                                | `BlindingReuse`         |
| Range Enforcement     | RangeCheck bundle fails                                                    | `RangeCheckOverflow`    |

Runtime errors abort the session and emit a structured JSON log (`severity=ERROR`).

---

## 5. Verifier Validation

Verifier re-executes the cryptographic pipeline deterministically.

### 5.1 Transcript Reconstruction

```
seed = H(PROG || BACKEND || PROFILE || PublicInputs)
```

If hash diverges → `TranscriptMismatch`.

### 5.2 Merkle Verification

* Each opened path checked depth = log₂(arityᵈ).
* Hash function agrees with program metadata.
* Root matches proof header.

Failures: `InvalidMerkleProof` or `HashMismatch`.

### 5.3 FRI Verification

* Number of rounds ≤ max_depth.
* Query indices distinct.
* Polynomial reconstruction degree ≤ declared bound.
  Failure: `FRIConsistencyError`.

### 5.4 Constraint Re-check

Verifier samples challenges from transcript and evaluates constraints on queried positions.
Mismatch → `ConstraintViolation`.

---

## 6. Backend Conformance Validation

Each adapter runs its own sanity tests at registration time.

| Test                 | Requirement                                                                |
| -------------------- | -------------------------------------------------------------------------- |
| Capability Integrity | Advertised field/hash pairs actually link to implemented curves/functions. |
| Proof Roundtrip      | Self-prove and verify toy AIR internally.                                  |
| Profile Mapping      | λ(profile) ≥ λ_min under adapter’s params.                                 |
| Serialization Parity | Proof bytes parse to identical commitments across versions.                |

Backends failing self-tests are refused registration.

---

## 7. ValidationReport Object

```rust
pub struct ValidationReport {
    pub config_passed: bool,
    pub air_passed: bool,
    pub runtime_passed: bool,
    pub verifier_passed: bool,
    pub commit_passed: bool, // new
    pub backend_id: String,
    pub profile_id: String,
    pub issues: Vec<ValidationIssue>,
}

pub struct ValidationIssue {
    pub stage: String,        // e.g. "runtime"
    pub code: String,         // e.g. "ConstraintUnsatisfied"
    pub message: String,      // human-readable
    pub fatal: bool,
}
```

Emitted as JSON under `reports/validation-{program_id}.json`.

---

## 8. Determinism Checks

1. Hash domain separation tags (`PROG`,`BUND`,`PUBI`) hard-coded.
2. Public input JSON keys sorted lexicographically before hashing.
3. Field serialization canonical LE representation.
4. Random oracles seeded exclusively from transcript hash.
5. Floating-point ops forbidden — integer field math only.

If two runs on different machines produce distinct proof hashes → `NonDeterministicExecution`.

---

## 9. Constraint Failure Semantics

### 9.1 Soft vs Hard Failures

| Type | Meaning                                        | Action            |
| ---- | ---------------------------------------------- | ----------------- |
| Soft | Recoverable — can retry with different profile | Warn + suggest    |
| Hard | Mathematical inconsistency                     | Abort immediately |

### 9.2 Examples

* Soft: `WeakSecurityProfile` → recommend “secure” profile.
* Hard: `ConstraintUnsatisfied` → abort and dump offending rows to `/tmp/trace_err.csv`.
* PedersenConfigMismatch → backend does not advertise pedersen=true
* InvalidCurvePoint → curve point not on curve
* BlindingReuse → blinding scalar reused
* RangeCheckOverflow → value exceeds declared bit bound
* KeccakUnavailable → backend lacks Keccak support

---

## 10. Logging and Observability

* All validation results stream to structured JSONL:

  ```json
  {"stage":"air","event":"DegreeOverflow","row":42,"timestamp":"2025-10-07T00:00:00Z"}
  ```
* `--stats` flag prints summary table:

| Stage    | Passed | Time (ms) |
| -------- | ------ | --------- |
| Config   | ✅      | 12        |
| AIR      | ✅      | 47        |
| Runtime  | ✅      | 1053      |
| Verifier | ✅      | 210       |

---

## 11. Cross-Backend Equivalence Validation

To ensure mathematical equivalence across adapters:

1. Generate proof under each backend with identical AIR + inputs.
2. Collect commitment roots and final public outputs.
3. Compute digest `D = H(roots || public_outputs)`.
4. All backends must yield same `D`.
   If not → `CrossBackendDrift`.

---

## 12. Performance Guards

* Runtime monitors wall-clock and RSS; if `mem > profile.limit_mem`, warn but continue.
* Excessive runtime (> 3× expected profile baseline) → `PerformanceAnomaly`.
* Reported in ValidationReport non-fatal issues.

---

## 13. Testing Hooks

The unit test framework exposes:

```rust
assert_validation_passes!(program, backend, profile);
assert_air_invalid!(bad_air_ir, "DegreeOverflow");
assert_cross_backend_equivalence!(program, ["native","winterfell"]);
```

All hooks return `ValidationReport` for inspection.

---

## 14. Rationale

Validation enforces determinism and honesty through layered checks:
first for configuration sanity, then for algebraic soundness, finally for cryptographic integrity.
By recording all outcomes in a machine-readable report, the system becomes auditable and provably reproducible across machines and backends.

**Design mantra:** *“Every bit accounted for.”*

---
