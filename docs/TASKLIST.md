# Tasklist — General-Purpose STARK Prover

**Parent RFC:** RFC-ZK01 v0.2  
**Status:** Living document (update as tasks close)

---

## Phase 0 — Foundation (MVP)

> Goal: deliver a working, deterministic prover with CLI + SDK and baseline backends.

### Task 1.1 — Repository Scaffold & Base Tooling
- Objective: initialize reproducible repo with Rust core + C ABI + docs.
- Files: `/Cargo.toml`, `/src/main.rs`, `/src/lib.rs`, `/crates/corelib/`, `/crates/ffi-c/`, `/docs/`, `/scripts/`, `/.gitignore`, `/.editorconfig`, `/.github/workflows/ci.yml`
- Steps: init repo, add workspace crates, dependencies (`serde`, `serde_json`, `thiserror`, `clap`, `rayon`, `blake3`), configure CI, add editorconfig/license/README, dummy proof main.
- DoD: repo builds/tests pass; CI succeeds; README includes Quickstart & layout.

### Task 1.2 — Backend Registry & Core Prover Interface
- Objective: implement trait system for adapters (native, Winterfell, Plonky2/3).
- Files: `/crates/corelib/src/backend.rs`, `/crates/corelib/src/registry.rs`, `/crates/corelib/src/errors.rs`
- Steps: define `ProverBackend`/`VerifierBackend`, registry lookup, capability struct, errors, unit tests.
- DoD: `cargo test -- corelib` passes; registry lists backends.

### Task 1.3 — Proof Profile System (Performance Tuning)
- Objective: parse `.toml` profiles controlling λ, FRI, memory.
- Files: `/crates/corelib/src/profile.rs`, `/profiles/{dev-fast.toml,balanced.toml,secure.toml,rec-mobile-fast.toml}`
- Steps: define `Profile` struct, implement load/validation, provide defaults, tests.
- DoD: profiles load/validate; CLI lists metadata.

### Task 1.4 — Native Reference Backend
- Objective: deterministic STARK-like backend for testing.
- Files: `/crates/backends/native/src/lib.rs`
- Steps: implement hash, simulate AIR compile/FRI, deterministic digest, register backend.
- DoD: `zkd prove -b native` deterministic; verify passes.

### Task 1.5 — CLI Tooling & Proof File Format
- Objective: build `zkd` CLI commands.
- Files: `/src/main.rs`, `/crates/corelib/src/cli.rs`, `/crates/corelib/src/proof.rs`
- Steps: define proof header + serialization, implement CLI prove/verify/backends/profiles, add stats, integration tests.
- DoD: CLI works with native backend; proof headers validated.

### Task 1.6 — Plonky2 Adapter
- Objective: bridge core API to Plonky2 recursion.
- Files: `/crates/backends/plonky2/src/lib.rs`
- Steps: add feature flag, implement adapter, map profile config, recursion capability, validate on toy AIR.
- DoD: Plonky2 backend functional; digest parity check passes.

### Task 1.7 — Plonky3 Adapter (Preferred Long-Term)
- Objective: integrate Plonky3 with recursion APIs.
- Files: `/crates/backends/plonky3/src/lib.rs`
- Steps: mirror Plonky2 adapter, efficient FRI mapping, recursion tests, compare perf in `bench_results.csv`.
- DoD: benches show comparable λ & digests.

### Task 1.8 — Recursion IR & Aggregator AIR
- Objective: implement recursion IR and aggregation template.
- Files: `/crates/corelib/src/recursion.rs`, `/tests/recursion_agg.rs`
- Steps: define `RecursionSpec`, digest rule, CLI `--inner` hook, unit tests for mismatches/limits.
- DoD: aggregation works; validation errors emit codes.

### Task 1.9 — C ABI Bindings
- Objective: export stable mobile/desktop FFI.
- Files: `/crates/ffi-c/src/lib.rs`, `/include/zkprov.h`
- Steps: implement 6 C ABI functions, map errors, expose alloc helpers, C harness validation.
- DoD: header compiles (`clang -Wall`); C example verifies proof.

### Task 1.10 — Dart FFI Plugin
- Objective: Flutter integration.
- Files: `/bindings/flutter_plugin/lib/zkprov_ffi.dart`, `/android/src/main/jniLibs/arm64-v8a/libzkprov.so`, `/ios/ZkProv.xcframework/`
- Steps: build plugin skeleton, implement bindings, sample UI, CI builds for Android/iOS.
- DoD: app verifies proofs; CI builds succeed.

### Task 1.11 — Validation & Report System
- Objective: structured JSON validation reports.
- Files: `/crates/corelib/src/validation.rs`, `/reports/`
- Steps: define `ValidationReport`, add checks, emit report files, failure-path tests.
- DoD: tests pass; invalid inputs report errors; benchmarks output CSV.

### Task 1.12 — Threat Model & Security Checks
- Objective: formalize threats and protections.
- Files: `/docs/threat-model.md`, `/crates/corelib/src/security.rs`
- Steps: enumerate risks, add proof checksum/integrity, version binding, tamper tests.
- DoD: tampering fails verification; threat model doc complete.

### Task 1.13 — Runbook & Deployment Scripts
- Objective: codify build/bench steps.
- Files: `/docs/runbook.md`, `/scripts/build.sh`, `/scripts/run_bench.sh`
- Steps: automate release build, bench script writing CSV, deployment notes.
- DoD: scripts run unattended; bench outputs expected range.

### Task 1.14 — Roadmap & Retrospective
- Objective: summarize Phase 0 & plan next.
- Files: `/docs/roadmap.md`, `/docs/retrospective.md`
- Steps: document wins/gaps, list Phase 1 extensions, update milestones.
- DoD: roadmap published; retrospective linked to RFC.

---

## Phase 1 — Portability (Plonky2 + cross-backend parity)

> Goal: add Plonky2 backend, enable recursion, enforce parity across native/Winterfell/Plonky2.

### Task 2.1 — Capabilities Matrix & Validator (extended)
- Objective: enforce backend/field/hash/arity compatibility.
- Files: `/crates/corelib/src/capabilities.rs`, `/backends/*.json`, `/crates/corelib/tests/cap_matrix.rs`
- Steps: extend `Capabilities`, load JSON per backend, implement `validate_config`.
- DoD: invalid combos rejected with precise error; unit tests cover ≥6 cases.

### Task 2.2 — AIR-IR → Backend Lowering Hooks
- Objective: provide lowering passes for adapters.
- Files: `/crates/corelib/src/air/lowering.rs`, `/crates/corelib/tests/air_lowering.rs`
- Steps: define `LoweredProgram`, implement `AirIR::lower`, property tests on sample AIRs.
- DoD: lowering stable across runs; no panics on degenerate inputs.

### Task 2.3 — Plonky2 Adapter: Basic Prove/Verify
- Objective: implement Plonky2 prove/verify.
- Files: `/crates/backends/plonky2/src/lib.rs`, `/crates/backends/plonky2/Cargo.toml`
- Steps: map profile to Plonky2 config, convert lowered program, wrap proof serialization.
- DoD: `zkd prove -b plonky2` works on toy AIR; digest `D` matches native/WF.

### Task 2.4 — Plonky2 Recursion Gadget (outer)
- Objective: support STARK-in-STARK recursion.
- Files: `/crates/backends/plonky2/src/recursion.rs`, `/tests/recursion_plonky2.rs`
- Steps: implement verification constraints, CLI `--inner` path, bench small aggregation.
- DoD: aggregation proof verifies; tampering triggers `RecursionDigestMismatch`.

### Task 2.5 — Cross-Backend Parity CI Job
- Objective: automated parity across three backends.
- Files: `/.github/workflows/ci.yml`, `/tests/cross_backend/parity_3x.rs`
- Steps: prove sample programs across backends, compare `D`, fail on mismatch.
- DoD: CI fails on parity drift; green otherwise.

### Task 2.6 — Mobile-Recommended Profiles (desktop-safe)
- Objective: publish mobile recursion profiles for Plonky2.
- Files: `/profiles/rec-mobile-fast.toml`, `/profiles/rec-mobile-balanced.toml`
- Steps: set `rows_max`/`max_inner`, conservative FRI params, add validator for caps.
- DoD: large traces rejected on mobile profiles with clear error.

### Task 2.7 — Docs & Examples for Plonky2
- Objective: document usage.
- Files: `/docs/recursion.md`, `/examples/plonky2/README.md`
- Steps: add backend selection examples, performance notes, parity caveats.
- DoD: example commands run locally.

---

## Phase 2 — Acceleration (GPU, Plonky3, SNARK wrapper)

> Goal: speed up proving and extend recursion while preserving determinism.

### Task 3.1 — GPU Feature Gate & Build Flags
- Objective: toggle GPU kernels cleanly.
- Files: `/Cargo.toml`, `/crates/corelib/src/gpu/mod.rs`, `/docs/runbook.md`
- Steps: add `gpu` feature, stub `GpuFftFri` with CPU fallback, detect device/log capability.
- DoD: builds succeed with/without `--features gpu`.

### Task 3.2 — GPU FFT/FRI Kernels (Phase 1)
- Objective: implement batched FFT and first FRI reduction on GPU.
- Files: `/crates/gpu/src/{fft.rs,fri.rs}`, `/tests/gpu/fft_roundtrip.rs`
- Steps: port radix-2 NTT to CUDA/OpenCL, validate forward/backward on 2¹⁴–2¹⁶ sizes.
- DoD: 2–3× speedup on large traces; unit tests pass numerically.

### Task 3.3 — Plonky3 Adapter
- Objective: integrate Plonky3 backend.
- Files: `/crates/backends/plonky3/src/lib.rs`, `/tests/parity_plonky3.rs`
- Steps: implement lowering/config mapping, recursion gadget, update parity tests.
- DoD: digest parity matches native/WF/Plonky2 on demos.

### Task 3.4 — SNARK Wrapper Adapter (optional)
- Objective: wrap multiple STARK proofs into succinct SNARK.
- Files: `/crates/backends/snarkwrap/src/lib.rs`, `/docs/architecture.md`
- Steps: define adapter verifying STARK transcript inside SNARK, expose verify-only SDK API.
- DoD: aggregated SNARK proof verifies example batch.

### Task 3.5 — Autotuner for Profiles
- Objective: suggest params for target λ/time/memory.
- Files: `/crates/corelib/src/tune.rs`, `/docs/runbook.md`
- Steps: estimate runtime from rows/degree/device, suggest profile overrides, CLI command.
- DoD: `zkd profile suggest` outputs reasonable configs.

### Task 3.6 — Bench Harness & CSV Publisher
- Objective: reproducible performance metrics.
- Files: `/scripts/bench_matrix.sh`, `/bench/bench_matrix.rs`, `/bench/results/*.csv`
- Steps: run program×backend×profile×gpu matrix, write CSV, gate regressions.
- DoD: CSV uploaded as CI artifact; regression gate active.

### Task 3.7 — Determinism Guard on GPU Path
- Objective: ensure GPU matches CPU outputs.
- Files: `/tests/determinism_gpu.rs`
- Steps: prove via CPU & GPU, compare headers and `D`.
- DoD: exact equality enforced.

---

## Phase 3 — Integration (REST/gRPC, Docker, SDKs)

> Goal: expose prover as service/library for other apps.

### Task 4.1 — REST API Server Skeleton
- Objective: provide `/v0/prove` & `/v0/verify` with job queue.
- Files: `/crates/server/src/main.rs`, `/openapi.yaml`
- Steps: implement endpoints, validate inputs, stream JSONL logs.
- DoD: curl round-trip works; OpenAPI served.

### Task 4.2 — Docker & CI Build
- Objective: containerize server & CLI.
- Files: `/Dockerfile`, `/docker-compose.yaml`, `/.github/workflows/docker.yml`
- Steps: multi-stage build, copy binaries, add healthcheck.
- DoD: `docker run zkprov:latest` runs server; e2e test passes.

### Task 4.3 — AuthN/Z & Rate Limiting
- Objective: API keys + rate limits.
- Files: `/crates/server/src/auth.rs`, `/docs/runbook.md`
- Steps: implement `X-Api-Key`, sliding-window limiter, log hashed key.
- DoD: load test shows limits enforced.

### Task 4.4 — TypeScript SDK
- Objective: Node/web client library.
- Files: `/sdks/ts/src/index.ts`, `/sdks/ts/package.json`, `/sdks/ts/README.md`
- Steps: wrap REST API, typed responses, build ESM/CJS.
- DoD: `npm pack` works; example script runs.

### Task 4.5 — Python SDK
- Objective: scripting/data science friendly wrapper.
- Files: `/sdks/py/zkprov/__init__.py`, `setup.py`, `README.md`
- Steps: implement same API, requests + retries, build wheels.
- DoD: `pip install -e .` works; example script verifies.

### Task 4.6 — Proof Cache & Idempotency Keys
- Objective: avoid recomputation for identical jobs.
- Files: `/crates/server/src/cache.rs`
- Steps: hash `(program_id, backend, profile, inputs)` to key, honor `Idempotency-Key`, return cached results.
- DoD: repeated prove hits cache; metrics show hit rate.

### Task 4.7 — Observability: Metrics & Tracing
- Objective: expose Prometheus metrics + traces.
- Files: `/crates/server/src/metrics.rs`, `/docs/runbook.md`
- Steps: add `/metrics`, record QPS/latency/failures, trace job IDs.
- DoD: Grafana dashboard shows backend latency.

### Task 4.8 — Pluggable Storage Adapters
- Objective: store proofs/reports in FS or cloud.
- Files: `/crates/server/src/storage/{fs.rs,s3.rs}`, `/config/service.toml`
- Steps: define `Storage` trait, implement FS + S3/GCS, config-driven selection.
- DoD: large proofs retrievable via presigned URLs.

---

## Phase 4 — Ecosystem & Tooling (Registry, Docs site, Bundles)

> Goal: grow adoption and simplify integration.

### Task 5.1 — Bundle Registry Format & CLI
- Objective: publish/resolve reusable AIR bundles.
- Files: `/crates/registry/src/lib.rs`, `/crates/cli/src/registry.rs`
- Steps: define `bundle.json`, implement `zkd registry publish/install`, lock hashes.
- DoD: `zkd registry install` pulls bundle, records in `zkd.lock`.

### Task 5.2 — Official Examples Repo (submodule)
- Objective: curated tested examples.
- Files: `/examples/*`, `.gitmodules`
- Steps: add merkle/range/hash-chain/polyeval demos, ensure CI runs them.
- DoD: examples pass on all supporting backends.

### Task 5.3 — Docs Site (static)
- Objective: publish docs with nav/search.
- Files: `/docs/site/*`, `/docs/sidebar.json`
- Steps: import RFCs/guides/API refs, deploy via Pages.
- DoD: site live with passing link check.

### Task 5.4 — Multi-Lang Bindings Index
- Objective: central index for bindings.
- Files: `/docs/bindings.md`, `/sdks/*/README.md`
- Steps: add canonical snippets, version matrix.
- DoD: code samples compile/run.

### Task 5.5 — Public CI Matrix (Backends × Profiles)
- Objective: publish nightly matrix on beefy hardware.
- Files: `/.github/workflows/matrix.yml`
- Steps: nightly bench jobs, attach CSV/flamegraphs, notify on regressions.
- DoD: trend graphs available via repo badges.

### Task 5.6 — Security Review & Hardening
- Objective: external review + fixes.
- Files: `/docs/security-review.md`, patches across core/backends.
- Steps: run `cargo audit/deny`, commit fuzz seeds, fix crashers.
- DoD: zero critical issues; review doc signed off.

### Task 5.7 — 1.0 Release Packaging
- Objective: prepare release artifacts.
- Files: `/CHANGELOG.md`, release GH action
- Steps: generate release notes, attach binaries (`zkd`, server image), publish SDK packages.
- DoD: `v1.0.0` tag live; artifacts downloadable.

---

## Phase Completion Gates (Summary)

* **Phase 1:** Plonky2 adapter operational; parity across 3 backends; mobile recursion profiles published.
* **Phase 2:** GPU acceleration optional; Plonky3 adapter; SNARK wrapper optional; CPU/GPU determinism ensured.
* **Phase 3:** REST/gRPC service in Docker; TS/Python SDKs; auth, rate limit, cache, metrics.
* **Phase 4:** Bundle registry; examples repo; docs site; security review; 1.0 release packaging.

---
