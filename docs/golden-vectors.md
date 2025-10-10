# **Golden Vector Registry**

**Parent RFC:** RFC-ZK01 §13.1

---

## 1. Purpose

Golden vectors capture deterministic proof outputs for reference AIRs across supported backends.
They ensure regressions are caught early and provide auditors with reproducibility checkpoints.

---

## 2. Directory Layout

```
tests/golden_vectors/
  balance/
    native.json
    winterfell.json
  merkle/
    native.json
```

Each JSON file records the digest, determinism vector, and manifest hash for a single backend.

---

## 3. JSON Entry Example

```json
{
  "program": "balance",
  "backend": "native@0.0",
  "profile": "dev-fast",
  "digest": "0xabc123...",
  "determinism_vector": {
    "compiler_commit": "0a1b2c3d",
    "backend": "native@0.0",
    "system": "linux-x86_64",
    "seed": "0001020304050607",
    "manifest_hash": "d3e4f5"
  }
}
```

---

## 4. CLI Commands

```bash
zkd vector add --program specs/balance.yml --backend native@0.0
zkd vector check --root tests/golden_vectors
zkd vector validate --root tests/golden_vectors --fail-fast
```

Vectors must be regenerated whenever AIR logic or backend code changes.

---

## 5. CI Enforcement

CI runs `zkd vector validate` for every backend, failing if any digest diverges or if determinism manifests are missing.
Registry updates require review of manifest hashes to prevent supply-chain tampering.

---

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
