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

### Task 0.9.1 — Node/TypeScript N-API Addon (OOB support)
- **Objective:** first‑class Node/TypeScript binding with prebuilt binaries.
- **Files:** `/bindings/node/{binding.gyp, src/addon.cc, index.ts, index.d.ts}`
- **Steps:** implement asynchronous `prove/verify` methods using N‑API; wrap configuration and output JSON into idiomatic objects; provide type definitions; implement a loader that resolves the correct `.node` binary per platform; integrate `prebuildify` to distribute precompiled artifacts.
- **DoD:** installing the package with `npm` and invoking `require('@zkprov/node').prove(...)` succeeds on Linux, macOS and Windows; CI jobs publish prebuilt binaries for each supported Node ABI.

### Task 0.9.2 — Dart FFI Plugin (Flutter)
- **Objective:** Flutter integration via Dart FFI (renumbered from the original Task 0.9.1).
- **Files:** `/bindings/flutter_plugin/lib/zkprov_ffi.dart`, `/android/src/main/jniLibs/arm64-v8a/libzkprov.so`, `/ios/ZkProv.xcframework/`
- **Steps:** implement Dart bindings to the C ABI using `dart:ffi`; provide per‑OS dynamic library loading logic; write helper functions for converting `Utf8` pointers; include finalizers that call `zkp_free` on returned pointers; add a small Flutter demo UI; configure the build scripts to package the correct shared libraries for Android and iOS.
- **DoD:** the Flutter demo app can generate and verify a toy proof locally on both Android and iOS; CI builds of the plugin succeed.

### Task 0.9.3 — Python Binding
* **Objective:** publish a Python package using `ctypes` or `cffi`.
* **Files:** `/bindings/python/{zkprov/__init__.py, setup.cfg, pyproject.toml}`
* **Steps:** declare the C function signatures for all exported symbols; ensure every function that returns a heap‑allocated pointer calls `zkp_free` when no longer needed (via context managers or helper functions); build manylinux, macOS and Windows wheels in CI; add a minimal example script that proves and verifies a toy program.
* **DoD:** `pip install zkprov` succeeds and running `python -m zkprov.hello` produces a proof and verifies it; CI produces wheels for all target platforms.

### Task 0.9.4 — Go (cgo) Binding
* **Objective:** provide an idiomatic Go wrapper over the C ABI.
* **Files:** `/bindings/go/{zkprov.go, go.mod}`
* **Steps:** use cgo to import the C functions; map return codes and JSON strings to Go errors and structs; ensure all heap‑allocated pointers are freed via finalizers or explicit `Close()` methods; expose a `Prove(ctx, Config) (Proof, error)` function and a corresponding `Verify()` helper; include a small example program.
* **DoD:** executing `go run examples/hello/main.go` proves and verifies a toy program without leaks; CI builds and tests the Go module across supported platforms.

### Task 0.9.5 — .NET (C# P/Invoke)
* **Objective:** ship a NuGet package with P/Invoke bindings.
* **Files:** `/bindings/dotnet/{ZkProv.csproj, ZkProv.cs}`
* **Steps:** declare `DllImport` signatures for each exported C function using the Cdecl calling convention; wrap returned pointers in a `SafeHandle` subclass to ensure `zkp_free` is called; package platform‑specific native binaries under `runtimes/*/native/`; publish a NuGet package with proper RID assets; write a simple console example.
* **DoD:** running `dotnet run examples/Hello` proves and verifies a proof; CI publishes a NuGet package containing native assets for Windows, Linux and macOS.

### Task 0.9.6 — Java/Kotlin (JNI) + Android AAR
* **Objective:** cover JVM and Android ecosystems via JNI.
* **Files:** `/bindings/java/{src/main/java/...}`, `/bindings/android/aar/`
* **Steps:** implement a JNI shim that calls into the C ABI and converts UTF‑8 strings to Java `String` objects; load the native library via `System.loadLibrary("zkprov")`; build an Android Archive (AAR) bundling the arm64‑v8a shared library; provide ProGuard/R8 keep rules; include a small Java/Kotlin example for desktop and an Android demo app.
* **DoD:** `gradle :bindings:android:assemble` produces a functioning AAR and the demo app verifies a proof on an Android device; Java desktop example verifies successfully; CI builds artifacts for all supported ABIs.

### Task 0.9.7 — Swift/iOS (SPM)
* **Objective:** create a Swift package over the iOS/macOS XCFramework.
* **Files:** `/bindings/swift/Package.swift`, Swift wrapper files
* **Steps:** define a module map exposing the C functions; wrap the C API in Swift functions returning `Result<T, Error>`; integrate the XCFramework into the Swift Package Manager manifest; add notes on code signing and entitlements; include a minimal Swift example app.
* **DoD:** the Swift package builds on macOS and iOS targets and a small example can prove and verify a toy program; CI ensures successful builds on both platforms.

### Task 0.9.8 — WASI/WASM (optional but recommended)
* **Objective:** provide a WebAssembly target for serverless and browser environments.
* **Files:** `/bindings/wasm/{zkprov_wasi.wasm, loader.js}`
* **Steps:** compile the core library to the `wasm32-wasi` target, exporting the same C ABI functions; write a small JavaScript glue layer that loads the WASM module and exposes `prove` and `verify` functions mirroring the Node API; ensure memory management via an exported `zkp_free` function; document how to import the WASM module in both Node and browser contexts.
* **DoD:** a Node or browser environment can load the WASM bundle and perform proof generation and verification via the JavaScript API; CI builds the WASM artifact and runs a smoke test.

### Task 0.9.9 — Examples & Troubleshooting
* **Objective:** create runnable examples and a troubleshooting guide for all FFI bindings.
* **Files:** `/examples/*`, `/docs/ffi-troubleshooting.md`
* **Steps:** provide minimal round‑trip demos (≈20 lines) for each supported language and platform; collect common integration issues (missing symbols, architecture mismatch, notarization or code‑signing errors, loader path problems) and document their resolutions; integrate these examples into CI.
* **DoD:** CI executes each example after building the corresponding binding; the troubleshooting guide includes a table of known issues and fixes.

### Task 0.9.10 — CI Matrix & Artifact Publishing
* **Objective:** automate cross‑platform builds, tests and publication for all bindings.
* **Files:** `/.github/workflows/ffi.yml`
* **Steps:** build core libraries for all target architectures; run language‑specific smoke tests using the examples from Task 0.9.9; upload prebuilt binaries and publish packages to the appropriate ecosystems (npm, PyPI, NuGet, Maven, pub.dev); ensure that failures in any binding break the CI.
* **DoD:** the GitHub Actions matrix shows green across all bindings; artifacts and packages are attached to releases and/or published to registries.

### Task 0.9.11 — AIR-IR Parser & Public I/O (commitment aware)
* **Objective:** parse `.air` including `Pedersen(curve)`, `PoseidonCommit`, and `KeccakCommit` bindings (renumbered from the original Task 0.9.2).
* **Files:** `/crates/corelib/src/air/{parser.rs,types.rs}`, `/tests/air_ir_{parser,degree}.rs`
* **Steps:** update the grammar to support the new commitment bindings; implement type and binding checks; ensure that degree accounting remains unchanged.
* **DoD:** `.air` files using commitment bindings parse and validate correctly.

### Task 0.9.12 — AIR-IR Parser & Public I/O (commitment aware)

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