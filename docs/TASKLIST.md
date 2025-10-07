Here’s a **fully regenerated `TASKLIST.md`** that bakes in the new crypto primitives, privacy gadgets (Pedersen, range, arithmetic under commitments), and EVM interop (Keccak + ABI + Solidity stub), and carries through all phases to a finished v1.0 with docs, SDKs, GPU, recursion, registry, and security review. It keeps your “micro-task + DoD + files” structure and is consistent with the rest of the spec suite.

---

# Tasklist — General-Purpose STARK Prover

**Parent RFC:** RFC-ZK01 v0.3 (post-commitment update)
**Status:** Living document (update as tasks close)
**Goal:** Deliver a deterministic, multi-backend STARK prover with commitments, privacy gadgets, and EVM interop; ship as CLI + library + service with SDKs and docs.

---

## Phase 0 — Foundation (MVP + Commitments)

> Goal: ship a working deterministic prover with CLI/SDK, baseline backends, **crypto primitives**, **privacy gadgets**, and **EVM interop**.
> Exit: CI green (coverage ≥80%), deterministic proofs, commitment tests pass, Solidity digest parity passes.

### Task 0.1 — Repository Scaffold & Base Tooling

* **Objective:** initialize reproducible repo with Rust workspace, core lib, CLI, docs.
* **Files:** `/Cargo.toml`, `/src/main.rs`, `/src/lib.rs`, `/crates/corelib/`, `/docs/`, `/scripts/`, `/.gitignore`, `/.editorconfig`, `/.gitattributes`, `/.github/workflows/ci.yml`, `/LICENSE`, `/README.md`
* **Steps:** init workspace; add deps (`serde`, `serde_json`, `thiserror`, `clap`, `rayon`, `blake3`); editorconfig; license; README; dummy CLI.
* **DoD:** `cargo build` & `cargo test` green locally and in CI; README Quickstart present.

### Task 0.2 — Backend Registry & Core Prover Interface

* **Objective:** trait system for adapters + registry.
* **Files:** `/crates/corelib/src/{backend.rs,registry.rs,errors.rs}`
* **Steps:** define `ProverBackend`/`VerifierBackend`, `Capabilities`, registry; unit tests.
* **DoD:** `cargo test -p corelib` passes; `list_backends()` returns `native` at minimum.

### Task 0.3 — Proof Profile System (Performance Tuning)

* **Objective:** parse TOML profiles controlling λ/FRI/memory.
* **Files:** `/crates/corelib/src/profile.rs`, `/profiles/{dev-fast.toml,balanced.toml,secure.toml}`
* **Steps:** struct + loader + validation; defaults; tests.
* **DoD:** `zkd profile ls` lists profiles; invalid profile rejected.

### Task 0.4 — Native Reference Backend

* **Objective:** deterministic reference backend for tests.
* **Files:** `/crates/backends/native/src/lib.rs`
* **Steps:** stub AIR compile, deterministic transcript, FRI sim, proof blob; registry register.
* **DoD:** `zkd prove -b native` works; `zkd verify` passes; stats emitted.

### Task 0.5 — CLI Tooling & Proof File Format

* **Objective:** `zkd` subcommands + binary proof format.
* **Files:** `/src/main.rs`, `/crates/corelib/src/{cli.rs,proof.rs}`
* **Steps:** implement `prove/verify/backend ls/profile ls/io schema`; stats flag; proof header parser.
* **DoD:** CLI integration tests pass; corrupted proof yields exit code 4.

### Task 0.6 — **Core Crypto Primitives Library**

* **Objective:** reusable primitives: Poseidon2, Rescue, Merkle(arity 2/4), Keccak256, hash-to-field.
* **Files:** `/crates/corelib/src/crypto/{poseidon.rs,rescue.rs,merkle.rs,keccak.rs,hash_to_field.rs}`, `/tests/crypto_primitives.rs`
* **Steps:** implement permutations; Keccak vectors; deterministic hash-to-field; merkle helpers.
* **DoD:** test vectors pass; `zkd io schema` reflects selected hash; merkle parity tests green.

### Task 0.7 — **Privacy Gadget Bundles (v1)**

* **Objective:** Pedersen commitments + range checks + arithmetic under commitments.
* **Files:** `/crates/bundles/{pedersen.rs,range.rs,arith.rs}`, `/crates/corelib/src/air/bindings.rs`, `/tests/privacy_gadgets.rs`
* **Steps:** extend `Capabilities` with `curves`, `pedersen`; implement `PedersenCommit(Cx,Cy)`, `RangeCheck(v,k)`, `AddUnderCommit`; validators for point validity and r-reuse.
* **DoD:** positive tests pass on `native`; negative tests emit `InvalidCurvePoint`, `BlindingReuse`, `RangeCheckOverflow`.

### Task 0.8 — **EVM Interop & ABI Helpers**

* **Objective:** Keccak commitments and digest parity with Solidity verifier stub.
* **Files:** `/crates/corelib/src/evm/{abi.rs,digest.rs}`, `/tests/evm_interop.rs`, `/examples/evm_verifier/contracts/VerifierStub.sol`, `/examples/evm_verifier/foundry.toml`
* **Steps:** ABI encoders for inputs/proof meta; KeccakCommit binding; Foundry project with `VerifierStub` that recomputes public-output digest `D`; parity tests.
* **DoD:** Solidity stub verifies `D` from a native proof; Keccak vectors pass; ABI round-trip equality.

### Task 0.9.0 — C ABI Bindings (Desktop + Mobile)
- **Objective:** export a stable C API for embedding (desktop + mobile), exposing proof and verification entry points.
- **Files:** `/crates/ffi-c/src/lib.rs`, `/include/zkprov.h`, `/tests/ffi_roundtrip.c`
- **Steps:**
  1. Expose six extern "C" functions: `zkp_init`, `zkp_prove`, `zkp_verify`, `zkp_list_backends`, `zkp_list_profiles`, and `zkp_free`.
  2. Map Rust errors to numeric return codes and structured JSON strings.
  3. Provide allocation helpers (`zkp_alloc`, `zkp_free`) for safe interop.
  4. Compile and link against Rust corelib; produce shared library `libzkprov.so` (Linux/Android) and `libzkprov.dylib` (macOS).
  5. Write a small C harness verifying a toy proof via FFI.
- **DoD:** header compiles with `clang -Wall`; C example verifies proof and prints deterministic digest D; FFI library builds on CI.

### Task 0.9.1 — Dart FFI Plugin (Flutter)
- **Objective:** publish Flutter plugin wrapping the C ABI.
- **Files:** `/bindings/flutter_plugin/lib/zkprov_ffi.dart`, `/android/src/main/jniLibs/arm64-v8a/libzkprov.so`, `/ios/ZkProv.xcframework/`
- **Steps:** implement Dart bindings; add sample UI; automate builds for Android/iOS.
- **DoD:** Flutter demo app proves/verifies locally; CI builds plugin successfully.


### Task 0.9.2 — AIR-IR Parser & Public I/O (commitment aware)

* **Objective:** parse `.air` incl. `Pedersen(curve)`, `PoseidonCommit`, `KeccakCommit` bindings.
* **Files:** `/crates/corelib/src/air/{parser.rs,types.rs}`, `/tests/air_ir_{parser,degree}.rs`
* **Steps:** grammar updates; type/binding checks; degree accounting unchanged.
* **DoD:** `.air` files with commitment bindings parse and validate.

### Task 0.10 — Validation & Report System (commitment-aware)

* **Objective:** structured JSON validation with new errors and `commit_passed` flag.
* **Files:** `/crates/corelib/src/validation.rs`, `/reports/`, `/tests/validation_commitments.rs`
* **Steps:** config gates for curves/pedersen/keccak; runtime checks for point validity, r-reuse, range checks; report flagging.
* **DoD:** ValidationReport includes `commit_passed`; negative tests log precise codes.

### Task 0.11 — Winterfell Adapter (v0.6)

* **Objective:** adapter to real STARK engine (no recursion yet).
* **Files:** `/crates/backends/winterfell/src/lib.rs`
* **Steps:** map AIR to Winterfell; parameter mapping from profiles; capability declaration; minimal proof bridge.
* **DoD:** end-to-end proofs/verify on toy & merkle AIR; cross-backend parity digest `D` with native on demos.

### Task 0.12 — Integration Tests & Golden Vectors

* **Objective:** e2e tests for toy/merkle/range/pedersen + golden vectors.
* **Files:** `/tests/integration/{e2e_toy.rs,e2e_merkle.rs,e2e_range.rs,e2e_pedersen.rs}`, `/tests/golden_vectors/{program.hash,roots.json}`
* **Steps:** produce vectors once; assert equality on CI; parity native↔winterfell.
* **DoD:** CI matrix passes; digests equal; vectors archived.

### Task 0.13 — Threat Model & Security Checks

* **Objective:** document threats; add integrity guardrails.
* **Files:** `/docs/threat-model.md`, `/crates/corelib/src/security.rs`
* **Steps:** λ envelope; transcript domain tags; forbid floating-point; pedersen notes on r-reuse.
* **DoD:** doc complete; tampering tests fail verification deterministically.

### Task 0.14 — Runbook & Bench Script

* **Objective:** reproducible build/bench.
* **Files:** `/docs/runbook.md`, `/scripts/{build.sh,run_bench.sh}`
* **Steps:** release build; bench toy/merkle/pedersen; CSV outputs.
* **DoD:** scripts run unattended; pedersen time in bounds.

### Task 0.15 — Phase 0 Retrospective & Docs Sync

* **Objective:** finalize docs to match implementation.
* **Files:** `/docs/{architecture.md,interfaces.md,validation.md,test-plan.md,roadmap.md}`
* **Steps:** ensure sections for commitments/keccak are complete; link fixtures; update diagrams.
* **DoD:** doc parity achieved; links validated.

---

## Phase 1 — Portability (Plonky2 + Recursion Scaffolding)

> Goal: add Plonky2 backend, extend capability matrix, publish **mobile-recommended profiles**, and enable recursion IR scaffolding (no outer proofs yet).

### Task 1.1 — Capabilities Matrix (extended)

* **Files:** `/crates/corelib/src/capabilities.rs`, `/backends/*.json`, `/crates/corelib/tests/cap_matrix.rs`
* **Steps:** add `curves`, `pedersen`, `keccak`; validate config/backends.
* **DoD:** invalid combos rejected with precise error messages.

### Task 1.2 — AIR-IR → Backend Lowering Hooks

* **Files:** `/crates/corelib/src/air/lowering.rs`, `/crates/corelib/tests/air_lowering.rs`
* **Steps:** define lowering for adapters; property tests; stability across runs.
* **DoD:** lowering deterministic and panic-free.

### Task 1.3 — Plonky2 Adapter: Basic Prove/Verify

* **Files:** `/crates/backends/plonky2/src/{lib.rs,config.rs}`
* **Steps:** map profiles; convert lowered program; proof serialization; declare recursion capability as `stark-in-stark`.
* **DoD:** `zkd prove -b plonky2` on demos; digest `D` parity with native/winterfell.

### Task 1.4 — Recursion IR (Backend-Neutral) + CLI Stubs

* **Files:** `/crates/corelib/src/recursion.rs`, `/tests/recursion_spec.rs`, `/src/main.rs`
* **Steps:** define `RecursionSpec`, header checks, digest rule; CLI `zkd prove --inner` arg parsing (no backend execution yet).
* **DoD:** spec validated; parser & config errors well-formed.

### Task 1.5 — Mobile-Recommended Profiles

* **Files:** `/profiles/{rec-mobile-fast.toml,rec-mobile-balanced.toml}`, `/crates/corelib/src/profile.rs`
* **Steps:** caps for rows/max_inner; enforcement; clear errors.
* **DoD:** large traces rejected with `RecursionLimitExceeded`.

### Task 1.6 — Cross-Backend Parity CI Job (3-way)

* **Files:** `/.github/workflows/ci.yml`, `/tests/cross_backend/parity_3x.rs`
* **Steps:** prove sample AIRs across native/winterfell/plonky2; compare `D`.
* **DoD:** CI fails on drift; otherwise green.

### Task 1.7 — Docs & Examples (Plonky2)

* **Files:** `/docs/recursion.md`, `/examples/plonky2/README.md`
* **Steps:** backend selection, profile notes, caveats.
* **DoD:** examples run locally.

---

## Phase 2 — Acceleration (GPU, Plonky3, Recursion Execute, SNARK Wrapper)

> Goal: speed up proving; land **Plonky3**; turn recursion spec into working aggregated proofs; optional SNARK wrapper.

### Task 2.1 — GPU Feature Gate & Stubs

* **Files:** `/Cargo.toml`, `/crates/corelib/src/gpu/mod.rs`, `/docs/runbook.md`
* **Steps:** feature `gpu`; stub types; device detection logs.
* **DoD:** builds succeed with/without `--features gpu`.

### Task 2.2 — GPU Kernels (FFT/FRI-1)

* **Files:** `/crates/gpu/src/{fft.rs,fri.rs}`, `/tests/gpu/fft_roundtrip.rs`
* **Steps:** radix-2 NTT; first FRI reduction; numeric tests.
* **DoD:** 2–3× speedup on 2¹⁶; unit tests pass.

### Task 2.3 — Plonky3 Adapter

* **Files:** `/crates/backends/plonky3/src/lib.rs`, `/tests/parity_plonky3.rs`
* **Steps:** lowering, profile mapping, recursion gadget hooks.
* **DoD:** digest parity with other backends on demos.

### Task 2.4 — Recursion Execution (Outer Proof)

* **Files:** `/crates/backends/plonky2/src/recursion.rs`, `/crates/backends/plonky3/src/recursion.rs`, `/tests/recursion_e2e.rs`
* **Steps:** implement verification constraints in outer; validate headers; compute `D*`; CLI `zkd prove --inner ...` functional.
* **DoD:** aggregated proof verifies; tampering → `RecursionDigestMismatch`.

### Task 2.5 — Determinism Guard on GPU Path

* **Files:** `/tests/determinism_gpu.rs`
* **Steps:** CPU vs GPU exact equality of headers & `D`.
* **DoD:** equality enforced; deviations error out.

### Task 2.6 — SNARK Wrapper (optional adapter)

* **Files:** `/crates/backends/snarkwrap/src/lib.rs`, `/docs/architecture.md`
* **Steps:** verify STARK transcript inside a succinct SNARK; verify-only API.
* **DoD:** wrapper proof verifies a batch.

### Task 2.7 — Bench Harness & CSV Publisher

* **Files:** `/scripts/bench_matrix.sh`, `/bench/{bench_matrix.rs,results/*.csv}`
* **Steps:** run program×backend×profile×gpu; CSV artifacts; regression gates.
* **DoD:** CI uploads CSV; regression guard active.

---

## Phase 3 — Integration (Service, Docker, SDKs, Observability)

> Goal: expose as REST/gRPC service; ship SDKs; rate-limit, cache, metrics, storage; production containers.

### Task 3.1 — REST API Server Skeleton

* **Files:** `/crates/server/src/main.rs`, `/crates/server/src/{routes.rs,auth.rs,cache.rs,metrics.rs,storage/{fs.rs,s3.rs}}`, `/openapi.yaml`
* **Steps:** `POST /v0/prove`, `POST /v0/verify`, `GET /v0/backends`, `GET /v0/profiles`; job queue; JSONL logs.
* **DoD:** curl round-trip works; OpenAPI served.

### Task 3.2 — Docker & CI Build

* **Files:** `/Dockerfile`, `/docker-compose.yaml`, `/.github/workflows/docker.yml`
* **Steps:** multi-stage build; healthcheck; push artifacts.
* **DoD:** `docker run zkprov:latest` runs server; e2e test passes.

### Task 3.3 — AuthN/Z & Rate Limiting

* **Files:** `/crates/server/src/auth.rs`, `/docs/runbook.md`
* **Steps:** `X-Api-Key`; sliding-window limiter; hashed key logs.
* **DoD:** load test shows limits respected.

### Task 3.4 — Proof Cache & Idempotency Keys

* **Files:** `/crates/server/src/cache.rs`
* **Steps:** hash `(program_id, backend, profile, inputs)`; honor `Idempotency-Key`; return cached results.
* **DoD:** high cache hit rate on repeats.

### Task 3.5 — Observability: Metrics & Tracing

* **Files:** `/crates/server/src/metrics.rs`, dashboards in `/docs/runbook.md`
* **Steps:** Prometheus `/metrics`; record QPS, latency, failures; trace job IDs.
* **DoD:** Grafana dashboard shows backend latency.

### Task 3.6 — TypeScript SDK

* **Files:** `/sdks/ts/src/index.ts`, `/sdks/ts/package.json`, `/sdks/ts/README.md`
* **Steps:** typed client for REST; ESM/CJS; retries; examples.
* **DoD:** `npm pack` works; example script proves & verifies.

### Task 3.7 — Python SDK

* **Files:** `/sdks/py/zkprov/__init__.py`, `setup.py`, `README.md`
* **Steps:** requests layer; retries; wheels build; examples.
* **DoD:** `pip install -e .` works; script verifies.

### Task 3.8 — Pluggable Storage Adapters

* **Files:** `/crates/server/src/storage/{fs.rs,s3.rs}`, `/config/service.toml`
* **Steps:** trait; FS + S3/GCS; presigned URLs.
* **DoD:** large proofs retrievable via presigned links.

---

## Phase 4 — Ecosystem & Tooling (Registry, Examples, Docs Site, Public CI)

> Goal: grow adoption; simplify integration; publish docs & examples; security hardening; v1.0 packaging.

### Task 4.1 — Bundle Registry Format & CLI

* **Files:** `/crates/registry/src/lib.rs`, `/crates/cli/src/registry.rs`
* **Steps:** `bundle.json` schema; `zkd registry publish/install`; lockfile `zkd.lock`.
* **DoD:** install pulls bundle; hashes locked.

### Task 4.2 — Official Examples Repo (submodule)

* **Files:** `/examples/*`, `.gitmodules`
* **Steps:** merkle/range/pedersen/hash-chain/recursion demos; CI executes.
* **DoD:** examples pass on supported backends.

### Task 4.3 — Docs Site (static)

* **Files:** `/docs/site/*`, `/docs/sidebar.json`
* **Steps:** import RFCs/guides/API refs; deploy via Pages; link checker.
* **DoD:** site live; all links green.

### Task 4.4 — Multi-Lang Bindings Index

* **Files:** `/docs/bindings.md`, `/sdks/*/README.md`
* **Steps:** canonical snippets; version matrix; ABI notes for EVM.
* **DoD:** code samples compile.

### Task 4.5 — Public CI Matrix (Backends × Profiles)

* **Files:** `/.github/workflows/matrix.yml`
* **Steps:** nightly bench jobs on beefy runner; attach CSV/flamegraphs; notify regressions.
* **DoD:** badges show trend graphs; auto-alerts on regressions.

### Task 4.6 — Security Review & Hardening

* **Files:** `/docs/security-review.md` + patches
* **Steps:** `cargo audit/deny`; commit fuzz seeds; fix crashers; privacy notes (r-reuse, curve checks).
* **DoD:** zero critical issues; review doc signed off.

### Task 4.7 — v1.0 Release Packaging

* **Files:** `/CHANGELOG.md`, `/.github/workflows/release.yml`
* **Steps:** generate release notes; attach binaries (`zkd`, server image); publish SDK packages; version tags.
* **DoD:** `v1.0.0` tag live; artifacts downloadable.

---

## Phase Completion Gates (Summary)

**Phase 0 (Foundation + Commitments)**

* [ ] Native & Winterfell backends working
* [ ] Deterministic proofs & transcripts
* [ ] **Crypto primitives** (Poseidon/Rescue/Merkle/Keccak) shipped
* [ ] **Privacy gadgets** (Pedersen + Range + Arith) shipped
* [ ] **EVM Interop** (KeccakCommit + ABI + Solidity digest stub) passes parity
* [ ] CI coverage ≥ 80%, validation reports emitted

**Phase 1 (Portability)**

* [ ] Plonky2 adapter functional
* [ ] Mobile recursion profiles enforce caps
* [ ] 3-way parity native↔winterfell↔plonky2

**Phase 2 (Acceleration & Recursion Execute)**

* [ ] GPU kernels yield 2–3× on large traces
* [ ] Plonky3 adapter functional
* [ ] Recursion outer proof works; determinism guard passes

**Phase 3 (Integration)**

* [ ] REST/gRPC service in Docker
* [ ] TS/Python SDKs live
* [ ] Auth, rate limits, cache, metrics, storage adapters

**Phase 4 (Ecosystem & Release)**

* [ ] Bundle registry live
* [ ] Examples & docs site live
* [ ] Security review passed
* [ ] v1.0 published

---

## Acceptance Criteria Map (Cross-Reference)

* **Determinism:** identical headers & `D` across runs and hosts (Ph0–Ph2).
* **Commitment correctness:** `commit_passed=true`; errors: `InvalidCurvePoint`, `BlindingReuse`, `RangeCheckOverflow`, `KeccakUnavailable`, `PedersenConfigMismatch` (Ph0).
* **Parity:** cross-backend `D` equality (Ph0–Ph1–Ph2).
* **Performance:** GPU speedup validated; pedersen benchmarks within documented bounds (Ph2).
* **Service quality:** rate-limit & auth enforced; cache hit rate measured; metrics exposed (Ph3).
* **Docs & DX:** examples compile; docs link-checked; SDK examples run (Ph4).
* **Security:** `cargo audit/deny` clean; fuzz seeds locked; review doc signed (Ph4).

---

## Repo Layout Reminder (post-tasks)

```
/src
  /air
  /bundles              # pedersen, range, arith
  /backend
     native/
     winterfell/
     plonky2/
     plonky3/
     snarkwrap/         # optional
  /crypto               # poseidon, rescue, merkle, keccak, hash_to_field
  /cli
  /evm                  # abi & digest helpers
/crates
  corelib/
  backends/
  gpu/
  server/
  registry/
  ffi-c/               
profiles/
tests/
  unit/
  integration/
  cross_backend/
  negative/
  fixtures/
bench/
docs/
sdks/
examples/
scripts/
```

---