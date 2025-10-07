# **Roadmap — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.2
**Status:** Living Document (updated per milestone completion)
**Goal:** Deliver a fully modular, deterministic, multi-backend STARK proving platform — usable both as a developer library and as a standalone proof service.

---

## 1. Vision

Provide a **universal STARK proving system** capable of:

* Authoring algebraic programs (AIR-IR) once, running them across multiple STARK engines.
* Allowing users to define custom bundles, public input schemas, and security profiles.
* Producing deterministic, auditable proofs regardless of the backend (Winterfell, Plonky2, Plonky3, etc.).
* Serving both local/offline proving use cases and future distributed proof services.

**Design pillars:**

1. Determinism → identical inputs = identical outputs.
2. Portability → same AIR across all backends.
3. Extensibility → new backends, new hashes, zero rewrites.
4. Transparency → open validation reports, golden vectors, reproducible benches.

---

## 2. Development Phases

| Phase       | Name         | Focus                                                             | Target Duration |
| ----------- | ------------ | ----------------------------------------------------------------- | --------------- |
| **Phase 0** | Foundation   | Core IR, validation, CLI/SDK, Native + Winterfell adapters        | 6–8 weeks       |
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
* Validation pipeline and `ValidationReport` emission
* Full test suite and golden vectors
* Docs: RFC, architecture, interfaces, validation, test-plan

### Exit Criteria

✅ Prove/verify identical proofs on native and Winterfell
✅ Deterministic seeds and transcript outputs
✅ Cross-backend digest `D` identical
✅ CI matrix passes with 80%+ coverage
✅ Validation report emitted for all runs

---

## 4. Phase 1 — Portability (Plonky2 Integration)

**Objective:** Introduce Plonky2 backend and cross-backend compatibility matrix.

### Key Tasks

* Implement Plonky2 adapter using existing AIR-IR lowering.
* Extend capability registry for recursion and lookup support.
* Add “recursion” profile flag.
* Integrate cross-backend round-trip tests (native ↔ winterfell ↔ plonky2).
* Add compatibility validator and user-facing error messages.

### Deliverables

* `backend/plonky2/` crate
* Updated `BackendRegistry`
* Extended `CapabilityMatrix` JSON schema
* CLI output enhancement: `zkd backend ls --capabilities`
* Test parity for 3 backends
* Phase 1 docs: `recursion.md`, updated examples

### Exit Criteria

✅ Plonky2 proofs verify deterministically
✅ Capability mismatch detection works
✅ Recursion support declared and gated
✅ Docs + examples updated

---

## 5. Phase 2 — Acceleration

**Objective:** Optimize proving performance and add GPU / recursion / SNARK wrapper support.

### Focus Areas

* GPU-accelerated FFT/FRI kernels (CUDA/OpenCL)
* Plonky3 backend with recursion improvements
* Optional SNARK wrapper for succinct proof verification
* Performance benchmark suite integrated into CI
* Adaptive parameter tuning (auto profile scaling)

### Deliverables

* `backend/plonky3/` adapter
* GPU runtime (optional `--features gpu`)
* Bench harness: `cargo bench --profile balanced`
* Proof compression ratio report
* Security envelope report (λ ≥ 100 confirmed)

### Exit Criteria

✅ GPU path 2–4× faster for large traces
✅ Plonky3 recursion verified
✅ Compression mode functional
✅ Bench suite produces reproducible CSVs

---

## 6. Phase 3 — Integration

**Objective:** Expose the proving system as a reusable service layer.

### Focus Areas

* REST/gRPC API endpoints
* Dockerized service (`docker run zk-prover`)
* API authentication + rate limiting
* Backend selection per request
* CI/CD pipeline with artifacts publishing

### Deliverables

* `/api/v0/prove` and `/api/v0/verify` endpoints
* `Dockerfile` + Helm chart (optional)
* Service logs in JSONL format
* Python/TypeScript SDK wrappers
* Metrics dashboard (Prometheus/Grafana)

### Exit Criteria

✅ Service deployable via Docker
✅ REST/gRPC fully functional
✅ SDKs tested against deployed instance
✅ Automated bench metrics in Grafana

---

## 7. Phase 4 — Ecosystem & Tooling

**Objective:** Build developer tools and shared resources for ecosystem growth.

### Focus Areas

* AIR bundle registry and versioning
* Official examples repository
* Online documentation site (`docs.zkprov.dev`)
* Tutorials + code walkthroughs
* Multi-language bindings (Rust, Python, JS)
* Public CI matrix (GitHub Actions / GCP runners)

### Deliverables

* Registry CLI: `zkd registry publish <bundle>`
* Docs site: interactive AIR explorer
* Example library: merkle, range, hash-chain
* SDK npm/crate releases
* Developer onboarding guides

### Exit Criteria

✅ Public documentation site live
✅ Bundle registry operational
✅ Community contributions enabled
✅ SDKs packaged and versioned

---

## 8. Long-Term Extensions (Post-v1.0)

* **Verifier Generators** — EVM, Cairo, RISC-Zero
* **Proof Aggregation** — batch verification and proof-of-proof
* **Deterministic Cloud Scaling** — distributed proof slicing
* **Adaptive Field Engines** — auto-select field/hash combos per use case
* **Formal Verification Layer** — proof soundness formally verified with Coq/Lean
* **ZK Bridge Tooling** — generate ZK proofs for cross-chain messaging (non-interactive)

---

## 9. Milestone Summary

| Milestone | Code Name         | Deliverables                           | Target Date          |
| --------- | ----------------- | -------------------------------------- | -------------------- |
| **M0**    | “Bootstrapped”    | Native + Winterfell MVP                | ✅ (Phase 0 complete) |
| **M1**    | “Polyglot”        | Plonky2 backend + cross-backend parity | +2 months            |
| **M2**    | “Accelerant”      | GPU/Plonky3/SNARK wrapper              | +5 months            |
| **M3**    | “Serviceable”     | REST/gRPC API + Docker service         | +7 months            |
| **M4**    | “Public Registry” | Docs site + bundle registry            | +9 months            |
| **M5**    | “Beyond 1.0”      | On-chain verifiers + formal proofs     | +12 months           |

---

## 10. Governance & Maintenance

* **Source of truth:** `main` branch protected; PR review required.
* **Versioning:** Semantic (`v0.x` for experimental, `v1.0` after Phase 2).
* **Registry policy:** Bundles signed with developer keys (Ed25519).
* **Deprecation:** Backends deprecated only with 2-version overlap.
* **Testing cadence:** Nightly (balanced), weekly (secure).

---

## 11. Success Metrics

| Metric                             | Definition                            | Goal                 |
| ---------------------------------- | ------------------------------------- | -------------------- |
| Proof determinism rate             | % of proofs identical across runs     | ≥ 99.999%            |
| Cross-backend parity               | % of digest D matches across backends | 100%                 |
| Mean proof time (balanced profile) | Seconds for 2¹⁶ trace                 | ≤ 2.0 s              |
| Coverage                           | Line coverage                         | ≥ 80%                |
| Docs completeness                  | Published spec parity with RFCs       | 100%                 |
| CI reliability                     | Successful pipeline runs              | ≥ 95%                |
| Ecosystem adoption                 | Bundles published by external devs    | ≥ 5 in first quarter |

---

## 12. Rationale

The roadmap treats each backend, adapter, and layer as an independently testable module.
It enforces reproducibility before optimization, and open validation before speed.
The resulting platform becomes the **"LLVM of STARKs"** — a backend-agnostic, deterministic proving toolchain that outlives any single cryptographic library.

**Mantra:** *“Prove once, run everywhere.”*

---
