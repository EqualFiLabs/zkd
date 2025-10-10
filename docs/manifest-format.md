# **Determinism Manifest Format**

**Parent RFC:** RFC-ZK01 §12.3

---

## 1. JSON Schema

```json
{
  "type": "object",
  "required": ["determinism_vector"],
  "properties": {
    "determinism_vector": {
      "type": "object",
      "required": ["compiler_commit", "backend", "system", "seed", "manifest_hash"],
      "properties": {
        "compiler_commit": { "type": "string" },
        "backend": { "type": "string" },
        "system": { "type": "string" },
        "seed": { "type": "string" },
        "manifest_hash": { "type": "string" }
      }
    }
  }
}
```

---

## 2. Field Semantics

* `compiler_commit` — Git SHA of the compiler producing the AIR/proof.
* `backend` — Fully-qualified backend identifier (`native@0.0`).
* `system` — Host triple (e.g., `linux-x86_64`).
* `seed` — Transcript seed used for deterministic randomness derivation.
* `manifest_hash` — Blake3 hash of the canonical manifest JSON.

---

## 3. Validation Logic

1. Serialize determinism vector with canonical JSON ordering.
2. Compute `manifest_hash = blake3(json_bytes)`.
3. Compare with recorded `manifest_hash`; mismatch → `DeterminismManifestMismatch`.
4. Persist manifest alongside proof for CI and auditor review.

---

## 4. Example

```json
{
  "program": "balance",
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

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
