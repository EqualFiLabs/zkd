# **Recursion — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.2
**Related:** `architecture.md`, `interfaces.md`, `validation.md`, `test-plan.md`, `runbook.md`
**Status:** Draft (stabilizes after Phase-1 adapters land)

---

## 1) Purpose

Define how the prover composes proofs **recursively** so users can:

* Aggregate many sub-proofs into one (batching).
* Build incremental/streaming proofs (append new work over time).
* Produce **proof-carrying data** where a program verifies another proof as a step.
* Do all of the above with **backend adapters** (Plonky2/3 targeted first; Winterfell wrapper later) and **mobile-friendly profiles** that are slower but sane.

The recursion layer must be opt-in and leave non-recursive programs unchanged.

---

## 2) Terminology

* **Inner proof**: a proof produced by some AIR program (Pᵢ).
* **Outer/Wrapper proof**: a proof whose AIR verifies one or more inner proofs.
* **Aggregator**: a program specifically designed to verify a set of inner proofs and expose a single public output (e.g., root digest).
* **PCD (Proof-Carrying Data)**: a program step that accepts prior proof(s) and emits a new proof chaining the claim forward.
* **Recursion mode**: { `stark-in-stark`, `snark-wrapper` }.

---

## 3) Scope & Non-Goals

### In scope

* STARK-in-STARK recursion via supported backends (Plonky2/3 first).
* A **portable Recursion IR** that describes “verify-this-proof” constraints independent of any one backend serialization.
* Aggregation programs (N inner → 1 outer).
* Consistent public-input binding across recursion layers.

### Out of scope (v0)

* Fancy gadgets (e.g., lookups for foreign curves) beyond what backends expose.
* Cross-system recursion (e.g., verify Groth16 directly) — allowed via **SNARK wrapper** later.

---

## 4) Design Overview

### 4.1 Recursion IR (Backend-neutral)

We define a small IR for “verify proof X inside program Y”:

```
RecursionSpec {
  inner_program_id: ProgramID,        // hash of inner AIR (+backend/profile)
  inner_backend_id: BackendID,
  public_io_schema: PublicDecl[],     // which public IO are required
  commitment_shape: CommitShape,      // Merkle arity, FRI params > normalized form
  digest_rule: DigestRule,            // how to compute digest D of inner public outputs
  max_inner: usize,                   // max proofs aggregated per step
}
```

The **composer** lowers this spec to the chosen backend via the adapter; if the backend cannot implement the verification constraints, it must **refuse** with a capability error.

### 4.2 Aggregation Program Template

Outer AIR = base constraints + **Recursion Gadget**:

* Accept up to `max_inner` inner proofs.
* For each inner:

  * Parse header → check `(program_id, backend_id, profile_id)`.
  * Reconstruct transcript seed.
  * Verify Merkle paths & FRI consistency (backend-specific).
  * Compute public-output digest `Dᵢ` using `DigestRule`.
* Reduce to a single `D* = H(D₁ || … || Dₖ)` as the **outer public output**.

### 4.3 Modes

* **STARK-in-STARK** (primary): Implemented where the backend exposes verification constraints as polynomials (Plonky2/3).
* **SNARK Wrapper** (optional): Prove “I verified k STARKs” inside a succinct SNARK. Exposed via a separate adapter (post-Phase-2).

---

## 5) Backend Capabilities & Mapping

| Backend        | Recursion | Notes                                                                                       |
| -------------- | --------- | ------------------------------------------------------------------------------------------- |
| **Native**     | none      | Use for local dev only; cannot do recursion.                                                |
| **Winterfell** | limited   | No first-class recursion; path via wrapper or simulated verifier (expensive). Phase-2 item. |
| **Plonky2**    | yes       | FRI-based recursion, Goldilocks + Poseidon2; preferred first target.                        |
| **Plonky3**    | yes       | Cleaner APIs; preferred long-term.                                                          |

Adapters expose `capabilities.recursion = {none | stark-in-stark | snark-wrapper}`. The coordinator enforces this at config time.

---

## 6) Public Inputs & Digest Binding

To keep layers portable and deterministic:

* Each inner proof exposes **public outputs** (e.g., Merkle root, accumulator).
* The recursion gadget computes a **canonical digest**:

Commitment digests (Pedersen/Poseidon/Keccak) are treated identically to other public outputs when computing `Dᵢ` and chaining into `D*`.

```
Dᵢ = H( inner_program_id || inner_backend_id || inner_profile_id || canonical(public_outputs) )
D* = H( D₁ || D₂ || ... || Dₖ )
```

`D*` becomes the outer **PublicOutput**, and can be chained again.
Mobile and desktop produce **identical D** for the same inputs.

---

## 7) API Extensions

### 7.1 CLI

```
# Build an aggregation proof over K proofs
zkd prove \
  -p programs/aggregate.air \
  -i inputs/aggregate.json \
  -b plonky3@X.Y \
  --profile balanced \
  --inner proofs/p1.proof proofs/p2.proof ... \
  -o proofs/agg.proof
```

### 7.2 SDK (Rust)

```rust
pub fn prove_recursive(
  outer: &Program,
  public_inputs: &PublicInputs,
  inners: &[Proof],                 // validated headers only
) -> Result<Proof, ProverError>;
```

Adapters receive an already-validated list of inner proofs (header sanity, ID checks) and implement the verification constraints.

---

## 8) Profiles (Desktop vs Mobile Recommended)

Recursion is heavier than a single proof. We publish **recipes**:

### 8.1 Desktop Recommended

* `rec-balanced`

  * λ ≈ 100, blowup 16, queries 30, grind 18
  * Max inner proofs per step: 8
  * Target rows (outer): 2¹⁶–2¹⁷
* `rec-secure`

  * λ ≈ 120, blowup 32, queries 50, grind 20
  * Max inner: 16
  * Rows: 2¹⁷–2¹⁸

### 8.2 Mobile Recommended (“good enough”)

* `rec-mobile-fast`

  * λ ≈ 80, blowup 8–12, queries 22–28, grind 16
  * Max inner: 2–4
  * Rows: ≤ 2¹⁶
* `rec-mobile-balanced`

  * λ ≈ 96, blowup 16, queries 30, grind 18
  * Max inner: 4–6
  * Rows: ≤ 2¹⁶–2¹⁷

> Engine enforces **rows_max** and **max_inner** for mobile profiles to avoid OOM and thermal runaway. Desktop profiles have higher caps.

---

## 9) Data Formats & Determinism

* Inner proof header **must** include: `program_id`, `backend_id`, `profile_id`, `public_output_digest`.
* Outer program’s `program_id` includes its **backend** and **profile** so different adapter choices never collide.
* Public IO canonicalization (JSON key order, field LE encoding) is reused from base system.

---

## 10) Failure Modes & Validation

Additional to `validation.md`:

| Stage       | Check                                   | Error                     |
| ----------- | --------------------------------------- | ------------------------- |
| Pre-flight  | All inner headers parse + IDs match     | `RecursionHeaderError`    |
| Binding     | Recompute `Dᵢ` = digest(public outputs) | `RecursionDigestMismatch` |
| Constraints | Backend recursion verifier satisfied    | `RecursionConstraintFail` |
| Limits      | `k ≤ max_inner`, rows ≤ profile cap     | `RecursionLimitExceeded`  |

All appear in `ValidationReport.issues` with `stage="recursion"`.

---

## 11) Example Flows

### 11.1 Batch aggregation (8 transfers → 1 proof)

* Produce 8 inner proofs under `native`/`winterfell`.
* On desktop, run outer under `plonky3 rec-balanced` with `max_inner=8`.
* Ship single aggregated proof to consumer app.

### 11.2 Streaming PCD (append-only log)

* Each step: verify previous step’s digest `D_prev`, add new chunk, emit `D_next`.
* Mobile uses `rec-mobile-fast` for small chunks; server can re-aggregate nightly.

### 11.3 “Verify on mobile” pattern

* Heavy proving on server → aggregate → send small outer proof to phone.
* Phone verifies with `zkp_verify()` (C ABI) using mobile profile parameters.
* Same API, just slimmer work.

---

## 12) Performance Targets (non-binding but sober)

* **Desktop (8-core AVX2)**: outer proof for 8 inners (2¹⁶ rows each) in **1.5–4.0 s** (`rec-balanced`).
* **Modern phone (2024–25 flagship)**: outer for 2–4 inners in **3–10 s** (`rec-mobile-balanced`).
* **Proof size**: outer ≤ 1.5× a single inner (depends on backend transcript).

We’ll publish exact numbers per backend in `bench_results.csv`.

---

## 13) Test Plan (delta to `docs/test-plan.md`)

* **Cross-backend recursion parity**:

  * Same 2 inners; outer under Plonky2 vs Plonky3 → identical `D*`.
* **Negative**:

  * Tamper one inner’s public output → `RecursionDigestMismatch`.
  * Feed inner with unknown `program_id` → `RecursionHeaderError`.
* **Mobile profiles**:

  * Enforce caps; OOM simulated if exceeded; correct `RecursionLimitExceeded`.
* **Determinism**:

  * Desktop and mobile (same backend+profile) produce identical `D*`.

---

## 14) Integration Guidelines for Host Apps

* Load the library via **C ABI** (see `runbook.md` + FFI header).
* When using recursion:

  * Validate inner proof headers first (`zkp_verify` cheap-path) before passing to `prove_recursive`.
  * Prefer **digest handoffs**: host app consumes only `D*` unless it truly needs inner details.
  * Use **mobile recommended profiles** on iOS/Android; allow users to opt into slower “balanced” desktop presets on powerful devices.

---

## 15) Security Notes

* Recursion strengthens *composability*, not soundness by itself. Your λ is bounded by the **weakest layer**.
* Always check `inner_backend_id` and `inner_profile_id` before trusting an inner.
* Do **not** mix mobile/desktop proof settings within one aggregated claim unless the outer AIR encodes per-inner λ as part of the digest rule (future enhancement).

---

## 16) Roadmap Hooks

* Phase-1: Plonky2 adapter implements `stark-in-stark` recursion gadget.
* Phase-2: Plonky3 adapter; GPU acceleration; optional SNARK wrapper adapter.
* Phase-3: Service APIs add `/aggregate` endpoint with job queuing and proof cache.

---

## 17) Acceptance Criteria (Recursion v0)

* [ ] `RecursionSpec` implemented; adapter gating by capability.
* [ ] CLI `zkd prove --inner …` produces an aggregated proof.
* [ ] SDK `prove_recursive()` compiles and passes integration tests.
* [ ] Mobile profiles enforce `max_inner` and `rows_max`.
* [ ] Cross-backend parity: `D*` equal across Plonky2/3 for same inputs.
* [ ] Validation emits correct recursion errors on tampering/mismatch.
* [ ] Bench sheet includes at least: {2,4,8} inners × {rec-balanced, rec-mobile-balanced}.

---

## 18) Rationale

Recursion lets us scale proof workflows without locking into any single library. By pinning a **backend-neutral Recursion IR**, digest rules, and conservative mobile presets, we make the prover a **drop-in backend** for other applications: select a backend, select a profile, and go. Desktop can chase speed; phones get “good enough” without exploding. Everyone gets the same APIs and the same deterministic answers.

**Mantra:** *“Aggregate more, promise less, verify everywhere.”*

---
