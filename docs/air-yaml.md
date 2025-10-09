# **AIR YAML Schema Reference**

**Parent RFC:** [RFC-ZK01 §4.2](./RFC-ZK01.md#42-yaml-air-definition)

---

## 1. Schema Keys

```yaml
meta:
  name: string
  version: string?
  field: string
  hash: { poseidon2 | blake3 | rescue }
  backend: string?
  profile: string?
  degree_hint: integer?
columns:
  trace_cols: integer
  const_cols: integer?
  periodic_cols: integer?
constraints:
  transition_count: integer
  boundary_count: integer
public:
  inputs: list? # reserved for future extensions
commitments:
  pedersen: bool?
  curve: string?
rows_hint: integer?
```

---

## 2. Minimal Example

```yaml
meta:
  name: balance_check
  field: Prime254
  hash: poseidon2
columns:
  trace_cols: 4
constraints:
  transition_count: 2
  boundary_count: 1
```

---

## 3. CLI Usage

```bash
zkd compile specs/balance.yml -o build/balance.air
```

* `--manifest` emits determinism vector JSON.
* Output `.air` files are byte-identical across hosts.

---

## 4. Error Surface

| Error Code             | Condition                                      | Remediation                 |
| ---------------------- | ---------------------------------------------- | --------------------------- |
| `YamlParseError`       | Malformed YAML syntax                          | Fix indentation or quoting |
| `InvalidMetaName`      | `meta.name` fails regex `[A-Za-z0-9_-]{2,64}`  | Rename program             |
| `MissingTraceCols`     | `columns.trace_cols` absent or zero            | Supply positive integer    |
| `ConstraintUnderflow`  | `constraints.transition_count == 0`            | Provide at least one       |
| `RowsHintOutOfRange`   | `rows_hint` not a power of two within bounds   | Adjust to `2^k`, k∈[3,22]  |
| `UnsupportedHash`      | `meta.hash` not supported by compiler          | Choose advertised hash     |

---

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
