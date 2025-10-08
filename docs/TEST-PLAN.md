# **Test Plan — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.2
**Related Docs:** `architecture.md`, `interfaces.md`, `validation.md`
**Status:** Draft → Stable once Phase 0 passes CI

---

## 1) Scope & Goals

This plan defines the **unit**, **integration**, **cross-backend**, **negative**, **fuzz**, **performance**, and **determinism** tests required to ship the MVP described in RFC-ZK01. It also specifies fixtures, CLI/SDK commands, golden vectors, and CI gates. Testing is part of the PPP-required verification step before merge. 

**Primary goals**

* Prove/verify correctness across supported backends (native, Winterfell MVP; Plonky2/3 optional in Phase 2).
* Guarantee deterministic outputs for identical inputs/profiles/backends.
* Enforce configuration guardrails and fail loudly on incompatible parameter sets.
* Provide reproducible benches with expected ranges for time/memory.

---

## 2) Environments

* **OS:** Ubuntu 22.04 (CI), macOS 14 (dev), Windows 11 (ad-hoc)
* **Toolchain:** Rust 1.81+, cargo, clang (for potential SIMD), jq
* **CPU Features:** AVX2 preferred (falls back if unavailable)
* **Profiles under test:** `dev-fast`, `balanced`, `secure`
* **Backends under test (Phase 0):** `native`, `winterfell@0.6`
* **Language runtimes:** Node.js LTS (18/20), Python 3.10+, Go 1.20+, .NET 6+, Swift 5.8+, WASI runtime (wasmtime ≥ 15)
* **Shared libraries:** Prebuilt `libzkprov` artifacts for Linux (x86_64, aarch64), macOS (universal), Windows (MSVC) plus `.wasm` for WASI

---

## 3) Directory Layout & Conventions

```
/tests
  /unit
    algebra_*.rs
    crypto_merkle.rs
    crypto_transcript.rs
    crypto_pedersen.rs
    crypto_keccak.rs
    fri_*.rs
    air_ir_{parser,degree}.rs
    public_io_{encode,bind}.rs
    gadget_range.rs
  /integration
    e2e_toy_air.rs
    e2e_merkle.rs
    e2e_running_sum.rs
    e2e_commitment.rs
  /cross_backend
    parity_roots.rs
  /negative
    bad_config.rs
    tamper_proof.rs
    wrong_inputs.rs
    invalid_curve.rs
    blinding_reuse.rs
    range_overflow.rs
  /fuzz
    transcript_roundtrip.rs
    fri_params.rs
  /golden_vectors
    program.hash
    inputs.json
    roots.json
  /fixtures
    pedersen.air
    pedersen_inputs.json
    rangecheck.air
    keccak.air
/scripts
  run_bench.sh
```

---

## 4) Fixtures & Golden Vectors

* **Toy AIR** (`tests/fixtures/toy.air`): 8 columns, simple transition `y' = y + x`.
* **Merkle AIR** (`tests/fixtures/merkle.air`): absorbs a 16-field root, no in-constraint use.
* **Running-sum AIR** (`tests/fixtures/runsum.air`): uses lifted public vector as CONST columns.
* **Public Inputs** (`tests/fixtures/*.json`): canonical JSON with deterministic ordering.
* **Golden Vectors**

  * `program.hash`: BLAKE3 of `(PROG || BACKEND || PROFILE)` domain.
  * `roots.json`: commitment roots and final public outputs for cross-backend parity.

Generation rule: write once under `tests/golden_vectors/`; tests verify equality on CI.

---

## 5) Unit Tests

### 5.1 Algebra (Field, FFT/NTT)

* **`algebra_field_ops.rs`**

  * Add/mul/inv properties (incl. zero/one edge cases).
  * Canonical LE serialization round-trip.
  * `cargo test --package core --test algebra_field_ops`
* **`algebra_fft.rs`**

  * Radix-2 planner; forward/backward NTT recovers input.
  * Coset sampling correctness.

### 5.2 Cryptographic Primitives

* **`crypto_merkle.rs`**

  * Poseidon2 node hashing; arity-2 vs arity-4 root equivalence under same leaves.
  * Path verification for random indices.
* **`crypto_transcript.rs`**

  * Domain separation tags (`PROG`,`BUND`,`PUBI`).
  * Public input absorption order and deterministic seed.
* **`crypto_pedersen.rs`**

  * Tests group operations, on-curve checks, and Pedersen commitment determinism.
* **`crypto_keccak.rs`**

  * Compares Rust implementation outputs to official Keccak test vectors.

### 5.3 FRI / LDT

* **`fri_layers.rs`**

  * Round construction caps depth; query schedule distinctness.
  * Recombination degree ≤ bound; negative test for bad degree.

### 5.4 AIR-IR & Degree Accounting

* **`air_ir_parser.rs`**

  * Parse minimal program; reject duplicate column IDs.
* **`air_ir_degree.rs`**

  * `deg(f) ≤ MAX_DEGREE` enforced; overflow → `DegreeOverflow`.

### 5.5 Public I/O

* **`public_io_bind.rs`**

  * `LiftAsConstCols`: injects CONST cols (length, typing).
  * `Absorb`: transcript seed changes with any byte change.
* **`public_io_encode.rs`**

  * Scalar/Vector/Bytes canonical encodings; JSON stability.

### 5.6 Range Gadgets

* **`gadget_range.rs`** — asserts proper enforcement of value bounds.

---

## 6) Integration Tests (End-to-End)

### 6.1 Toy AIR Round-Trip

* **Command (CLI):**

  ```bash
  zkd prove  -p tests/fixtures/toy.air -i tests/fixtures/toy.json -b native --profile balanced -o /tmp/toy.proof
  zkd verify -p tests/fixtures/toy.air -i tests/fixtures/toy.json -b native --profile balanced -P /tmp/toy.proof
  ```
* **Expected:**

  ```
  ✅ Proof verified successfully.
  Stats: elapsed_ms≈100..800, mem_bytes≤64MB
  ```

### 6.2 Merkle AIR (Absorb-only)

* Prove/verify with both `native` and `winterfell@0.6`.
* Confirm that changing one byte in `root` fails with `TranscriptMismatch`.

### 6.3 Running-Sum AIR (Lifted Vector)

* Prove/verify; enforce auto-injected range checks for `U32` when enabled.
* Memory ceiling check: `CONST_COL_LIMIT` respected.

---

## 7) Cross-Backend Parity Matrix

| Program    | Inputs      | Profile  | Backends            | Check                          |   |                  |
| ---------- | ----------- | -------- | ------------------- | ------------------------------ | - | ---------------- |
| toy.air    | toy.json    | balanced | native ↔ winterfell | same public outputs digest `D` |   |                  |
| merkle.air | merkle.json | balanced | native ↔ winterfell | same `D = H(roots              |   | public_outputs)` |
| runsum.air | runsum.json | balanced | native ↔ winterfell | same `D`                       |   |                  |

**Command (SDK example):**

```rust
let d_native = digest_outputs("native", "balanced")?;
let d_wf     = digest_outputs("winterfell@0.6", "balanced")?;
assert_eq!(d_native, d_wf);
```

Failure raises `CrossBackendDrift` (see `validation.md` §11).

---

## 8) Negative Tests

* **Bad Config**

  * `backend=plonky2` + `hash=keccak` → `CapabilityMismatch`.
  * `fri_blowup=1` → `InvalidFRIConfig`.
* **Tampered Proof**

  * Flip one byte in `proof.bytes` → `ProofDecodeError` or `ConstraintViolation`.
* **Wrong Public Inputs**

  * Provide mismatched `root` → `TranscriptMismatch`.
* **Degree Overflow**

  * Construct constraint with `deg > MAX_DEGREE` → `DegreeOverflow`.
* **Trace Shape**

  * Non-power-of-two length → `InvalidTraceLength`.

| Test                     | Expected Error           |
| ------------------------ | ------------------------ |
| Invalid curve point      | `InvalidCurvePoint`      |
| Blinding reuse           | `BlindingReuse`          |
| Range overflow           | `RangeCheckOverflow`     |
| Backend missing pedersen | `PedersenConfigMismatch` |
| Backend missing keccak   | `KeccakUnavailable`      |

All negative tests must **abort** and log structured JSON errors per `interfaces.md`.

---

## 9) Fuzz & Property Tests

* **Transcript Round-Trip Fuzz**

  * Random public input permutations → same lexicographic JSON → identical seeds.
* **FRI Parameter Fuzz**

  * Randomize `(blowup, queries)` within bounds; reject outside bounds.
* **AIR Expression Fuzz**

  * Random expression trees bounded by `MAX_DEGREE`; parser never panics.

Run with:

```bash
cargo test --features fuzzing -- --ignored --nocapture
```

---

## 10) Performance & Resource Tests

Script: `/scripts/run_bench.sh`

### 10.1 Targets (balanced profile)

| Program | Rows | Expected time (native) | Expected mem |
| ------- | ---- | ---------------------- | ------------ |
| toy     | 2¹⁴  | 0.2–1.0 s              | ≤ 128 MB     |
| merkle  | 2¹⁶  | 0.6–2.5 s              | ≤ 256 MB     |
| runsum  | 2¹⁶  | 0.8–3.0 s              | ≤ 320 MB     |

| Program  | Expected time (balanced) | Notes                                                  |
| -------- | ------------------------ | ------------------------------------------------------ |
| pedersen | 1–3 s                    | Includes group ops; validate mobile profile throttling |

Outliers > 3× baseline → `PerformanceAnomaly` (non-fatal) recorded in `ValidationReport`.

---

## 11) Determinism Tests

* Two consecutive runs (same host) produce **identical** proof headers and transcript seeds.
* Cross-machine (CI runner vs local) equality on:

  * Program hash
  * Public-input encoding digest
  * Final public outputs digest `D`

Command:

```bash
zkd prove  -p tests/fixtures/merkle.air -i tests/fixtures/merkle.json -b native --profile balanced -o /tmp/m.proof
jq -r '.seed' /tmp/m.proof.meta > /tmp/seed1
zkd prove  -p tests/fixtures/merkle.air -i tests/fixtures/merkle.json -b native --profile balanced -o /tmp/m.proof
jq -r '.seed' /tmp/m.proof.meta > /tmp/seed2
diff /tmp/seed1 /tmp/seed2
```

Exit code must be `0`.

---

## 12) CLI Contract Tests

* `zkd backend ls` prints at least `native` and `winterfell@0.6` with capability tables.
* `zkd profile ls` includes `dev-fast`, `balanced`, `secure`.
* `zkd io schema -p tests/fixtures/runsum.air` prints JSON schema with lifted vectors.

---

## 13) SDK Contract Tests

* `prove()` / `verify()` round-trip returns `Ok(true)` on valid inputs.
* Errors serialize to JSON:

  ```json
  { "error": "CapabilityMismatch", "message": "hash=keccak not supported by plonky2" }
  ```
* Event stream emits `ProofGenerated` and `ProofVerified` with stats populated.

---

## 14) Validation Reports

Every test that performs a prove/verify must emit a `ValidationReport` to `reports/validation-*.json` and assert:

* `config_passed && air_passed && runtime_passed && verifier_passed == true`
* `commit_passed == true`
* `issues.is_empty()` for positive tests
* Specific codes set for negative tests (`ConstraintUnsatisfied`, `TranscriptMismatch`, etc.)

See `validation.md` for report schema.

---

## 15) FFI & Multi-Language Tests

* **C harness round-trip:** Build and run `tests/ffi/c_roundtrip.c` against `libzkprov` to call `zkp_init`, `zkp_prove`, and `zkp_verify` on toy and merkle fixtures; assert return codes are `NULL` and free all pointers via `zkp_free`.
* **Node/TypeScript binding test:** Execute the N-API addon’s jest/tap suite to prove and verify the toy AIR asynchronously; compare digests with CLI fixtures.
* **Python binding test:** Use `pytest` with `ctypes`/`cffi` wrapper to call the shared library, deserialize JSON responses, and ensure proof buffers are freed explicitly.
* **Go binding test:** Run `go test ./bindings/go/...` to compile the cgo wrapper and prove/verify toy + merkle programs.
* **.NET binding test:** Execute `dotnet test` for the P/Invoke wrapper, ensuring `SafeHandle` disposes buffers.
* **Swift/iOS test:** Build the Swift Package Example (macOS + iOS simulator) verifying a recursive proof via the Swift binding.
* **WASI/WebAssembly test:** Run `wasmtime` against the wasm binding to confirm `zkp_verify` passes for the toy fixture in a WASI sandbox.
* **Cross-language parity:** Generate a proof through the CLI/SDK and verify it in each binding (and vice versa) asserting identical `D` digests and seeds.
* **Memory leak checks:** Run Valgrind (Linux), Instruments (macOS), and `dotnet-counters` to ensure repeated FFI calls do not leak when `zkp_free` is used.
* **Callback test:** Register an event callback from each binding, run a proof, and assert receipt of progress JSONL messages with monotonically increasing `percent`.

All bindings publish CI jobs that build language packages, link against the shipped `libzkprov` artifacts, and upload logs/artifacts mirroring CLI/SDK tests.

---

## 16) CI Matrix & Gates

**Matrix**

* Backends: `native`, `winterfell@0.6`
* Profiles: `dev-fast` (unit), `balanced` (integration & parity)

**Gates**

* Unit tests: 100% pass
* Integration: 100% pass
* Cross-backend parity: all `D` digests equal
* Lints/format: `cargo fmt -- --check`, `cargo clippy -D warnings`
* Coverage: **≥ 80%** lines, **≥ 90%** critical modules (crypto, FRI, transcript)
* Artifacts: upload `events.jsonl`, `reports/validation-*.json`, and bench logs

This satisfies the PPP execution & iteration flow with passing test suite and clean artifacts. 

---

## 17) Acceptance Criteria (MVP)

* [ ] **Unit**: Algebra, Merkle, Transcript, FRI, AIR-IR, Public I/O all green
* [ ] **Integration**: toy/merkle/runsum prove & verify on `native` and `winterfell`
* [ ] **Cross-backend**: parity digest `D` identical across backends
* [ ] **Negative**: all planned failures emit correct error codes
* [ ] **Fuzz**: no panics; rejects invalid FRI ranges; parser stable
* [ ] **Performance**: within expected ranges; no OOM
* [ ] **Determinism**: identical seeds/headers across runs and hosts
* [ ] **FFI**: C ABI + Node/TS, Python, Go, .NET, Swift, WASI bindings pass round-trip and parity tests across supported OSes
* [ ] **CI Gates**: matrix passes; coverage thresholds met

---

## 18) Expansion (Phase 2+)

* Add Plonky2/3 to matrix; enable recursion tests (stark-in-stark).
* Add GPU kernels to bench matrix.
* Add on-chain verifier golden tests (EVM/Cairo) with gas/word-size budgets.
* Add “large trace” nightly with synthetic 2¹⁹–2²¹ rows.

---

## 19) Rationale

This plan enforces the PPP principle of **tight, implementation-ready specs** with explicit success conditions and reproducible commands. It keeps tests atomic (1–2 hours when run locally in subsets) and ties results to deterministic artifacts and golden vectors. 

**Mantra:** *“Prove it, then prove we proved it.”*

---
