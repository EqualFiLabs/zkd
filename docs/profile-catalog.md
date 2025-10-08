# Pre-Baked Application Profile Catalog

**Parent RFC:** RFC-ZK01 v0.3  
**Purpose:** Provide ready-to-use, deterministic, composable ZK profiles for common privacy and verification use cases.  
**Location:** `/profiles/catalog.toml` and `/programs/{profile_id}.air`  
**Versioning:** Each entry follows semantic versioning and includes presets for `fast`, `balanced`, `tight`, and `mobile`.

---

## zk-auth-pedersen-secret — Passwordless / Secret Authentication

**Goal:** Prove knowledge of a secret opening a Pedersen commitment `C`, bound to a server challenge `(nonce, origin)`.

**Use Case:** Passwordless login, device-based authentication, key-ownership proof.

| Field | Type | Description |
|-------|------|-------------|
| `C` | felt | Pedersen commitment to the secret. |
| `nonce` | hex32 | Server-issued challenge to prevent replay. |
| `origin` | felt | Domain separation tag for application origin. |
| `nullifier` | felt | Optional linkability tag for rate-limit or uniqueness. |

**Private Inputs:** `secret`, `blinding`  
**Gadgets:** `PedersenCommit`, `PoseidonBind`  
**Notes:** Supports optional `nullifier` for linkability control; canonical digest binding ensures one-use per nonce.

---

## zk-allowlist-merkle — Set Membership + Challenge Binding

**Goal:** Prove membership of a leaf in a Merkle tree with root `R`, optionally bound to a challenge.

**Use Case:** Whitelisted access, DAO or group membership proofs.

| Field | Type | Description |
|-------|------|-------------|
| `root` | hex32 | Merkle root of the allowlist (fixed for the session). |
| `pk_hash` | hex32 | Hash of the user’s public key or identifier. |
| `path` | vec<hex32> | Authentication path from leaf to root. |
| `nonce` | hex32 | Session challenge to prevent replay. |
| `origin` | felt | Domain separation tag. |

**Private Inputs:** `merkle_path`  
**Gadgets:** `MerklePathVerify`, `PoseidonBind`  
**Notes:** Root versioned per deployment; tampering yields `TranscriptMismatch`.

---

## zk-attr-range — Attribute Range Proof

**Goal:** Prove a committed value lies within `[a,b]` without revealing it.

**Use Case:** KYC / age gating, verified credentials, bounded data attestations.

| Field | Type | Description |
|-------|------|-------------|
| `commitment` | felt | Pedersen commitment to the attribute value. |
| `min` | felt | Lower bound of acceptable range. |
| `max` | felt | Upper bound of acceptable range. |

**Private Inputs:** `value`, `blinding`  
**Gadgets:** `RangeCheck`, `PedersenCommit`  
**Notes:** Supports unsigned integer ranges up to 64 bits; digest binds `(min,max)` to prevent reuse.

---

## zk-balance-geq — Balance ≥ Threshold

**Goal:** Prove account balance is ≥ X without revealing the exact amount.

**Use Case:** Private solvency attestations, staking eligibility, off-chain audits.

| Field | Type | Description |
|-------|------|-------------|
| `commitment` | felt | Commitment to user’s balance. |
| `threshold` | felt | Minimum balance threshold to prove against. |
| `adapter_proof` | hex32 | Optional proof digest from external balance adapter. |

**Private Inputs:** `balance`, `blinding`  
**Gadgets:** `RangeCheck`, `PedersenCommit`, optional `AdapterSource(balance_feed)`  
**Notes:** Adapters can fetch balances from EVM/oracle/API; digest includes adapter proof hash.

---

## zk-uniqueness-nullifier — Rate-Limit / One-Action-Per-Epoch Proof

**Goal:** Prove uniqueness of an action within an epoch (e.g., one vote or claim).

**Use Case:** Anti-Sybil gating, faucet rate limits, anonymous reputation.

| Field | Type | Description |
|-------|------|-------------|
| `nullifier` | felt | Derived from `(secret, epoch)` to enforce one-use. |
| `epoch` | u64 | Numeric epoch counter. |

**Private Inputs:** `secret`  
**Gadgets:** `PoseidonNullifier`, `PoseidonBind`  
**Notes:** Linkable only within same epoch; prevents double actions.

---

## zk-proof-of-solvency-lite — Liabilities vs Assets Commitment Delta

**Goal:** Prove aggregate assets ≥ aggregate liabilities using only commitments.

**Use Case:** Exchange / protocol solvency attestations without disclosing addresses.

| Field | Type | Description |
|-------|------|-------------|
| `asset_root` | hex32 | Merkle root of committed asset values. |
| `liability_root` | hex32 | Merkle root of committed liabilities. |
| `delta_commitment` | felt | Pedersen commitment to (assets − liabilities). |

**Private Inputs:** `asset_values`, `liability_values`, `blinding`  
**Gadgets:** `MerklePathVerify`, `PedersenCommit`, `RangeCheck`  
**Notes:** Simplified non-recursive version for regular audits; supports optional verifier key binding.

---

## zk-vote-private — Private One-Ballot Voting

**Goal:** Prove that exactly one valid ballot was cast from an allowlist.

**Use Case:** Anonymous DAO or governance voting.

| Field | Type | Description |
|-------|------|-------------|
| `root` | hex32 | Merkle root of eligible voters. |
| `vote_commitment` | felt | Commitment to user’s vote. |
| `nonce` | hex32 | Random challenge for uniqueness. |
| `tally_binding` | hex32 | Digest binding vote to tally session. |

**Private Inputs:** `vote`, `path`, `blinding`  
**Gadgets:** `MerklePathVerify`, `PedersenCommit`, `PoseidonBind`  
**Notes:** Includes tally-binding challenge to prevent reuse; CI ensures cross-backend parity.

---

## zk-file-hash-inclusion — Document Existence / Inclusion Proof

**Goal:** Prove that a document hash is included in a committed Merkle root.

**Use Case:** Timestamping, notarization, provenance, IP claim proofs.

| Field | Type | Description |
|-------|------|-------------|
| `root` | hex32 | Merkle root of the document set. |
| `file_hash` | hex32 | Keccak or Poseidon hash of the file. |
| `path` | vec<hex32> | Merkle authentication path. |

**Private Inputs:** `merkle_path`  
**Gadgets:** `MerklePathVerify`, `PoseidonBind`  
**Notes:** Deterministic commitment ensures identical digest `D` for identical documents.

---

## zk-score-threshold — Reputation / Score ≥ Threshold

**Goal:** Prove that a hidden score or metric ≥ a public threshold.

**Use Case:** Reputation systems, credit scoring, eligibility checks.

| Field | Type | Description |
|-------|------|-------------|
| `commitment` | felt | Commitment to user’s score. |
| `threshold` | felt | Minimum score threshold. |
| `epoch` | u64 | Scoring epoch to prevent replay. |

**Private Inputs:** `score`, `blinding`  
**Gadgets:** `PedersenCommit`, `RangeCheck`  
**Notes:** Epoch binding prevents proof replay; deterministic across backends.

---

## zk-age-over — Fast Range Profile for Age Gating

**Goal:** Minimal circuit proving “age ≥ 18” or “age ≥ 21”.

**Use Case:** Web onboarding, regional content gating, privacy-preserving KYC.

| Field | Type | Description |
|-------|------|-------------|
| `commitment` | felt | Commitment to user’s age. |
| `bound` | felt | Age threshold (e.g., 18 or 21). |

**Private Inputs:** `age`, `blinding`  
**Gadgets:** `RangeCheckLite`, `PedersenCommit`  
**Presets:** `fast`, `mobile`  
**Notes:** Optimized for mobile; proving <1 s typical; digest tied to `(bound)`.

---

## Canonical Digest Construction

All profiles share identical digest computation for deterministic cross-backend behavior:

```

D = H(profile_id || profile_version || canon_pub_inputs || proof_bytes)

```

**Rules**

* Public input keys sorted lexicographically.  
* Field elements encoded as canonical decimal strings.  
* Gadgets and profile version bound into the transcript seed.  
* Identical inputs always yield identical digest `D`.

---

## Validation & CI Coverage

Every profile must pass:

1. **Cross-backend parity** (`native`, `winterfell`, `plonky2`).  
2. **ValidationReport inclusion** with `profile_id` and `profile_version`.  
3. **Negative tests:** `TranscriptMismatch`, `InvalidCurvePoint`, `RangeCheckOverflow`, `BlindingReuse`.  
4. **Performance gates:** runtime ≤ preset limit; memory ≤ declared cap.

---

## Adding New Profiles

1. Copy an entry from `/profiles/catalog.toml`.  
2. Add the `.air` program and schema definition.  
3. Extend this catalog with the new profile entry.  
4. Add validation tests under `/tests/profiles/`.  
5. Verify deterministic digest `D` and parity compliance.

---

**Maintainer:** EqualFi Labs — ZKD Core Team  
**Last Updated:** 2025-10-08
