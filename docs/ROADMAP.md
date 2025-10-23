# **Roadmap — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.3
**Status:** Living Document (updated per milestone completion)
**Goal:** Deliver a fully modular, deterministic, multi-backend STARK proving platform — usable both as a developer library and as a standalone proof service.

---

## 1. Vision

Provide a **universal STARK proving system** capable of:

* Authoring algebraic programs (AIR-IR) once, running them across multiple STARK engines **(including AIRs authored in YAML or via SDK)**.
* Allowing users to define custom bundles, public input schemas, and security profiles **(profiles are required; engines are capability-selected, never implicitly defaulted)**.
* Producing deterministic, auditable proofs regardless of the backend (Winterfell, Plonky2, Plonky3, etc.) **with an embedded Determinism Manifest (proof provenance vector)**.
* Serving both local/offline proving use cases and future distributed proof services.

**Design pillars:**

1. Determinism → identical inputs = identical outputs.
2. Portability → same AIR across all backends.
3. Extensibility → new backends, new hashes, zero rewrites.
4. Transparency → open validation reports, golden vectors, reproducible benches.
5. Developer-first → YAML-defined AIRs and pre-baked profiles reduce cryptographic overhead.
6. Safe composability → Backend chosen by capability matching; explicit errors (no silent defaults).

---

## 2. Development Phases

| Phase       | Name         | Focus                                                             | Target Duration |
| ----------- | ------------ | ----------------------------------------------------------------- | --------------- |
| **Phase 0** | Foundation   | Core IR, validation, CLI/SDK/FFI, Native + Winterfell adapters    | 6–8 weeks       |
| **Phase 1** | Portability  | Plonky2 backend, recursion scaffolding, cross-backend conformance | 8–10 weeks      |
| **Phase 2** | Acceleration | GPU kernels, Plonky3, SNARK wrappers, distributed proving         | 10–12 weeks     |
| **Phase 3** | Integration  | REST/gRPC API, Dockerized service, on-chain verifier generators   | 8 weeks         |
| **Phase 4** | Ecosystem    | SDK packages, documentation site, public registry of AIR bundles  | Ongoing         |

---

## 3. Phase 0 — Foundation (MVP)

**Objective:** Deliver a working, deterministic, general-purpose STARK prover with CLI + SDK and baseline backends.

### Deliverables

* AIR-IR compiler and schema validation
* Backend adapter traits + registry
* Native backend (reference)
* Winterfell adapter (v0.6)
* Proof profiles: `dev-fast`, `balanced`, `secure`
* CLI: `zkd prove`, `zkd verify`, `zkd backend ls`
* Stable C ABI + maintained CLI/SDK surface with official Phase-0 bindings for Python, Flutter/Dart, and WASI (Node/TS addon remains part of the CLI toolchain)
* DIY bindings cookbook detailing Go, .NET, Java/Kotlin, and Swift integration paths until official support returns in the Ecosystem phase
* Validation pipeline and `ValidationReport` emission
* Full test suite and golden vectors
* Docs: RFC, architecture, interfaces, validation, test-plan
* Commitment & Privacy Gadgets: Pedersen, PoseidonCommit, KeccakCommit
* Range-check bundle and validation coverage
* **Pre-baked Application Profiles:**
  Ready-made circuits for authentication, allowlist, attribute range, balance, uniqueness, solvency, private voting, file-inclusion, score-threshold, and age-gating use cases, plus a `profile-catalog.md` documenting schemas, presets, and validation vectors.
  Delivered with SDK/CLI helpers and included in test/validation suites.
* EVM-compatible Keccak and ABI encoders
* **YAML AIR authoring**: parser + compiler (`.yaml → .air`) with schema validation and deterministic output.
* **Adapter Selection Rule**: capability-based backend resolution (`--backend auto`) with `NoCompatibleBackend` error on mismatch.
* **Determinism Manifest**: include a proof-level provenance vector (compiler commit, backend id, system fingerprint, seed derivation) in all proof outputs.
* **Golden vectors (local)**: canonical vectors for toy/merkle/runsum used in CI parity across native and Winterfell.
* **EVM digest parity**: Solidity stub verifier and ABI encoders for Keccak-based commitments (digest equality tests).

### Exit Criteria

✅ Prove/verify identical proofs on native and Winterfell
✅ Deterministic seeds and transcript outputs
✅ Cross-backend digest `D` identical
✅ CI matrix passes with 80%+ coverage
✅ Validation report emitted for all runs
✅ FFI round-trip proofs succeed for toy/merkle/runsum across the supported Phase-0 bindings (C ABI harness, Python, Flutter/Dart, WASI) and operating systems
✅ **Determinism Manifest** embedded and validated in every proof (manifest hash present in `ValidationReport`).
✅ **Backend auto-selection** works per *Adapter Selection Rule*; incompatible configs fail with descriptive `NoCompatibleBackend`.
✅ **Golden vector parity**: identical digests across **native** and **Winterfell** for all Phase 0 vectors.
✅ **EVM digest parity**: off-chain KeccakCommit digest equals on-chain Solidity stub verification for provided examples.

---

## 4. Phase 1 — Portability (Plonky2 Integration)

**Objective:** Introduce Plonky2 backend and cross-backend compatibility matrix.

Build upon the new crypto/commitment primitives to enable cross-backend parity with Pedersen and Keccak digests.

### Key Tasks

* Implement Plonky2 adapter using existing AIR-IR lowering.
* Extend capability registry for recursion and lookup support.
* Add “recursion” profile flag.
* Integrate cross-backend round-trip tests (native ↔ winterfell ↔ plonky2).
* Add compatibility validator and user-facing error messages.
* Establish **Golden Vector Registry** format and CLI (`zkd vector add/check`); migrate Phase 0 vectors into the registry.
* Enforce **CI parity** against the registry for all adapters (native, winterfell, plonky2).

### Deliverables

* `backend/plonky2/` crate
* Updated `BackendRegistry`
* Extended `CapabilityMatrix` JSON schema
* CLI output enhancement: `zkd backend ls --capabilities`
* Test parity for 3 backends
* Phase 1 docs: `recursion.md`, updated examples
* **Golden Vector Registry** (repo directory + JSON index) with CI enforcement.
* `zkd vector validate` CLI subcommand.

### Exit Criteria

✅ Plonky2 proofs verify deterministically
✅ Capability mismatch detection works
✅ Recursion support declared and gated
✅ Docs + examples updated
✅ **Registry parity**: all Phase 1 vectors pass on **native ↔ winterfell ↔ plonky2** with identical digest `D`.
✅ **Capability reporting**: `zkd backend ls --capabilities` shows recursion/lookup/field/hash; recursion flag in profiles gates backend choice.

---

## 5. Phase 2 — Acceleration

**Objective:** Optimize proving performance and add GPU / recursion / SNARK wrapper support.

### Focus Areas

* GPU-accelerated FFT/FRI kernels (CUDA/OpenCL)
* Plonky3 backend with recursion improvements
* Optional SNARK wrapper for succinct proof verification
* Performance benchmark suite integrated into CI
* Adaptive parameter tuning (auto profile scaling)
* **Deterministic WASM/mobile builds**: single-threaded baseline with WASM SIMD where available; transcript identical to native.

### Deliverables

* `backend/plonky3/` adapter
* GPU runtime (optional `--features gpu`)
* Bench harness: `cargo bench --profile balanced`
* Proof compression ratio report
* Security envelope report (λ ≥ 100 confirmed)
* **WASM target artifacts** (browser/WASI) for core prover; identical Determinism Manifest to native builds.

### Exit Criteria

✅ GPU path 2–4× faster for large traces
✅ Plonky3 recursion verified
✅ Compression mode functional
✅ Bench suite produces reproducible CSVs
✅ **WASM/native determinism**: same digest `D` and manifest hash for Phase 0/1 vectors.

---

## 6. Phase 3 — Integration

**Objective:** Expose the proving system as a reusable service layer.

### Focus Areas

* REST/gRPC API endpoints
* Dockerized service (`docker run zk-prover`)
* API authentication + rate limiting
* Backend selection per request
* CI/CD pipeline with artifacts publishing
* **On-chain digest verifiers**: generator for minimal Solidity stubs tied to profile digests (not full circuit-specific verifiers).

### Deliverables

* `/api/v0/prove` and `/api/v0/verify` endpoints
* `Dockerfile` + Helm chart (optional)
* Service logs in JSONL format
* Python/TypeScript SDK wrappers
* Metrics dashboard (Prometheus/Grafana)
* **Verifier generator**: `zkd evm verifier --profile <id>` outputs a minimal Solidity digest-parity contract + Foundry tests.
* **EVM interop docs**: end-to-end example (off-chain proof → on-chain digest verify).

### Exit Criteria

✅ Service deployable via Docker
✅ REST/gRPC fully functional
✅ SDKs tested against deployed instance
✅ Automated bench metrics in Grafana
✅ **Digest-parity deployable**: generated Solidity verifier passes Foundry tests against Golden Vector proofs.

---

## 7. Phase 4 — Ecosystem & Tooling

**Objective:** Build developer tools and shared resources for ecosystem growth.

### Focus Areas

* AIR bundle registry and versioning **(+ profile catalog + public Golden Vector index)**
* Official examples repository
* Online documentation site (`docs.zkprov.dev`)
* Tutorials + code walkthroughs
* Multi-language bindings (Phase 0 ships C ABI + Python/Flutter/WASI; Phase 4 reintroduces official Go, .NET, Java/Kotlin, and Swift packages once resourced)
* Public CI matrix (GitHub Actions / GCP runners)

### Deliverables

* Registry CLI: `zkd registry publish <bundle>`
* Docs site: interactive AIR explorer
* Example library: merkle, range, hash-chain
* SDK npm/crate releases
* Developer onboarding guides
* *Reproducible Build Scripts (Optional):* document how to rebuild from source and verify golden proof digests.
* **Public Golden Vector portal**: browsable index, per-backend status badges, and downloadable vectors.
* **Bundle signing & verification**: bundles and profiles signed (Ed25519); registry enforces signature checks.

### Exit Criteria

✅ Public documentation site live
✅ Bundle registry operational
✅ Community contributions enabled
✅ SDKs packaged and versioned
✅ Public Golden Vector index online; latest release passes badges for all supported backends.

---

## 8. Long-Term Extensions (Post-v1.0)

* **Verifier Generators** — EVM, Cairo, RISC-Zero
* **Proof Aggregation** — batch verification and proof-of-proof
* **Deterministic Cloud Scaling** — distributed proof slicing
* **Adaptive Field Engines** — auto-select field/hash combos per use case
* **Formal Verification Layer** — proof soundness formally verified with Coq/Lean
* **ZK Bridge Tooling** — generate ZK proofs for cross-chain messaging (non-interactive)
* **Multi-proof composition** (non-recursive aggregation of identical AIR proofs via transcript stitching).
* **Secure-mode runtime** (constant-time field ops, witness zeroization toggled by profile).

---

## 9. Milestone Summary

| Milestone | Code Name         | Deliverables                           | Target Date          |
| --------- | ----------------- | -------------------------------------- | -------------------- |
| **M0**    | “Bootstrapped”    | Native + Winterfell MVP **(+ YAML AIR + Determinism Manifest + local golden vectors + EVM digest stub)** | ✅ (Phase 0 complete) |
| **M1**    | “Polyglot”        | Plonky2 backend + cross-backend parity **(+ Golden Vector Registry & CI)** | +2 months            |
| **M2**    | “Accelerant”      | GPU/Plonky3/SNARK wrapper **(+ WASM/native determinism)** | +5 months            |
| **M3**    | “Serviceable”     | REST/gRPC API + Docker service **(+ EVM verifier generator)** | +7 months            |
| **M4**    | “Public Registry” | Docs site + bundle registry **(+ public Golden Vector index, signed bundles)** | +9 months            |
| **M5**    | “Beyond 1.0”      | On-chain verifiers + formal proofs     | +12 months           |

---

## 10. Governance & Maintenance

* **Source of truth:** `main` branch protected; PR review required.
* **Versioning:** Semantic (`v0.x` for experimental, `v1.0` after Phase 2).
* **Registry policy:** Bundles signed with developer keys (Ed25519); Golden Vector entries PR-reviewed and gated by CI parity.
* **Deprecation:** Backends deprecated only with 2-version overlap; removal requires passing vectors to remain archived with status “deprecated”.
* **Testing cadence:** Nightly (balanced), weekly (secure).

---

## 11. Success Metrics

| Metric                             | Definition                            | Goal                 |
| ---------------------------------- | ------------------------------------- | -------------------- |
| Proof determinism rate             | % of proofs identical across builds and hosts (same AIR + inputs) | ≥ 99.999 %          |
| Cross-backend parity               | % of digest D matches across backends | 100%                 |
| Mean proof time (balanced profile) | Seconds for 2¹⁶ trace                 | ≤ 2.0 s              |
| Coverage                           | Line coverage                         | ≥ 80%                |
| Docs completeness                  | Published spec parity with RFCs       | 100%                 |
| CI reliability                     | Successful pipeline runs              | ≥ 95%                |
| Ecosystem adoption                 | Bundles published by external devs    | ≥ 5 in first quarter |
| Determinism manifest coverage      | % proofs containing valid Determinism Manifest                  | 100%                 |
| Golden vector pass rate            | % registry vectors with identical digests across all adapters   | 100%                 |
| Auto-selection accuracy            | % runs where capability-based selection matches a valid backend | ≥ 99.9%             |
| WASM/native digest parity          | % vectors identical between WASM and native builds              | 100%                 |

---

## 12. Rationale

The roadmap treats each backend, adapter, and layer as an independently testable module.
It enforces reproducibility before optimization, and open validation before speed.
The resulting platform becomes the **"LLVM of STARKs"** — a backend-agnostic, deterministic proving toolchain that outlives any single cryptographic library.

> YAML-defined AIRs, a strict capability-based adapter selection rule, an explicit Determinism Manifest, and a Golden Vector Registry transform ZKP adoption from research to routine engineering. The platform remains backend-agnostic, auditable, and reproducible across native, WASM, and mobile builds—without introducing zk-VM semantics.

**Mantra:** *“Prove once, run everywhere.”*

---
Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
