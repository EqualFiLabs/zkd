# **Runbook — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.3
**Purpose:** Operational checklist for engineers running CI, release, and incident response workflows.

---

## 1. Preflight

1. `cargo fmt && cargo clippy` — ensure style.
2. `cargo test` — run unit + integration suites.
3. `zkd backend ls` — confirm capability registry matches expected adapters.

---

## 2. Compiling AIRs from YAML

1. Edit the YAML specification under `specs/*.yml`.
2. Compile deterministically:

   ```bash
   zkd compile specs/balance.yml -o build/balance.air
   ```

3. Commit both the `.yml` and generated `.air` artifacts.
4. Record the determinism manifest emitted by `zkd compile --manifest` for release notes.

---

## 3. Proving & Verification Steps

1. Prove with capability-based backend selection:

   ```bash
   zkd prove -p build/balance.air -i inputs/balance.json --profile dev-fast -o proofs/balance.proof
   ```

2. Verify using manifest-aware mode:

   ```bash
   zkd verify --manifest proofs/balance.proof.json -P proofs/balance.proof
   ```

3. Store proof bytes, manifest JSON, and determinism vector in release artifacts.

---

## 4. Golden Vector Validation

1. Regenerate and compare vectors:

   ```bash
   zkd vector validate --root tests/golden_vectors
   ```

2. Investigate any `vector_passed = false` results by comparing digests per backend.
3. Use `zkd vector add` when onboarding new AIRs, ensuring manifest hashes are reviewed in PRs.

---

## 5. Digest Comparison Across Backends

1. For targeted parity checks, run:

   ```bash
   zkd diff digest --program build/balance.air --inputs inputs/balance.json
   ```

2. The command prints a table of backend digests; escalations occur if any entry diverges.
3. Attach diff output to incident tickets when discrepancies arise.

---

## 6. Automation Hooks

* **Makefile:**

  ```make
  compile:
zkd compile specs/balance.yml -o build/balance.air

  vector:
./scripts/run_vector_tests.sh

  diff:
zkd diff digest --program build/balance.air --inputs inputs/balance.json
  ```

* **CI pipeline:**
  1. Install toolchain.
  2. Run `cargo test`.
  3. Execute `./scripts/run_vector_tests.sh`.
  4. Archive `proofs/*.proof` and `proofs/*.proof.json`.

---

## 7. Incident Response

1. If manifest hashes diverge, freeze releases and regenerate proofs from source.
2. Compare determinism vectors between failing and passing builds to isolate toolchain drift.
3. Update Golden Vector registry with audited digests once remediation is verified.

---

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
