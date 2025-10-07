---

# **Interfaces — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.2
**Purpose:** Formalize every external and internal interface exposed by the proving system, including the CLI, SDK, backend trait contracts, and JSON/TOML schemas.
**Status:** Draft → Frozen once the Phase 0 implementation compiles.

---

## 1. CLI Interfaces (`zkd`)

### 1.1 Commands

| Command          | Description                                                        |
| ---------------- | ------------------------------------------------------------------ |
| `zkd init`       | Scaffold a new proving workspace with default config and profiles. |
| `zkd prove`      | Build trace, run backend prover, and emit proof blob.              |
| `zkd verify`     | Re-run transcript and verify proof deterministically.              |
| `zkd io schema`  | Display the declared public input/output schema of a program.      |
| `zkd profile ls` | List all available proof-profile presets.                          |
| `zkd backend ls` | Enumerate registered backend adapters and their capabilities.      |

### 1.2 Syntax Examples

```bash
# Prove a program with Winterfell backend
zkd prove \
  -p programs/merkle.air \
  -i inputs/merkle.json \
  -b winterfell@0.6 \
  --profile balanced \
  -o proofs/merkle-balanced.proof

# Verify the proof
zkd verify \
  -p programs/merkle.air \
  -i inputs/merkle.json \
  -b winterfell@0.6 \
  --profile balanced \
  -P proofs/merkle-balanced.proof
```

### 1.3 CLI Exit Codes

| Code | Meaning                            |
| ---- | ---------------------------------- |
| `0`  | Success / proof verified           |
| `1`  | Verification failed                |
| `2`  | Invalid configuration              |
| `3`  | Backend capability mismatch        |
| `4`  | Proof file corrupted or unreadable |
| `5`  | Internal runtime error             |

### 1.4 Common Flags

| Flag              | Alias | Type   | Description                                   |
| ----------------- | ----- | ------ | --------------------------------------------- |
| `-p`, `--program` |       | Path   | AIR-IR file (`.air`)                          |
| `-i`, `--inputs`  |       | Path   | JSON public input file                        |
| `-b`, `--backend` |       | String | Backend ID (`winterfell@0.6`)                 |
| `--profile`       |       | String | Profile name (`dev-fast`,`balanced`,`secure`) |
| `-o`, `--output`  |       | Path   | Proof output path                             |
| `-P`              |       | Path   | Proof input path (for verification)           |
| `--stats`         |       | Bool   | Print runtime stats JSON                      |

---

## 2. SDK (Rust)

### 2.1 Core Types

```rust
pub struct Program {
    pub id: Uuid,
    pub air_ir: AirIR,
    pub backend: String,
    pub profile: String,
}

pub struct PublicInputs(pub serde_json::Value);

pub struct Proof {
    pub bytes: Vec<u8>,
    pub stats: ProofStats,
}

pub struct ProofStats {
    pub elapsed_ms: u64,
    pub mem_bytes: u64,
    pub blowup: u32,
    pub queries: u32,
}
```

### 2.2 Functions

```rust
pub fn prove(
    program: &Program,
    public_inputs: &PublicInputs,
) -> Result<Proof, ProverError>;

pub fn verify(
    program: &Program,
    public_inputs: &PublicInputs,
    proof: &Proof,
) -> Result<bool, VerifierError>;

pub fn list_backends() -> Vec<BackendInfo>;
pub fn list_profiles() -> Vec<ProfileInfo>;
```

### 2.3 Error Contracts

```rust
enum ProverError {
    InvalidConfig(String),
    BackendUnavailable(String),
    CapabilityMismatch(String),
    AirCompileError(String),
    TraceBuildFailed(String),
    ProofGenerationFailed(String),
}

enum VerifierError {
    ProofDecodeError(String),
    TranscriptMismatch(String),
    InvalidMerkleProof(String),
    ConstraintViolation(String),
}
```

Each error implements `Display` and serializes as a JSON object:

```json
{ "error": "CapabilityMismatch", "message": "hash=keccak not supported by plonky2" }
```

---

## 3. Backend Adapter Interfaces

### 3.1 Trait Definitions

```rust
pub trait ProverBackend: Send + Sync {
    fn id(&self) -> &'static str;                    // e.g. "winterfell@0.6"
    fn capabilities(&self) -> Capabilities;
    fn compile(
        &self,
        air: &AirIR,
        bundles: &BundleSet,
    ) -> Result<BackendProgram, BackendError>;
    fn prove(
        &self,
        prog: &BackendProgram,
        pub_io: &PublicIO,
        profile: &Profile,
    ) -> Result<ProofBlob, BackendError>;
}

pub trait VerifierBackend: Send + Sync {
    fn verify(
        &self,
        prog: &BackendProgram,
        pub_io: &PublicIO,
        proof: &ProofBlob,
        profile: &Profile,
    ) -> Result<bool, BackendError>;
}
```

### 3.2 Capability Structure

```rust
pub struct Capabilities {
    pub fields: Vec<FieldId>,
    pub hashes: Vec<HashId>,
    pub fri_arities: Vec<u32>,
    pub recursion: RecursionMode,
    pub lookups: bool,
}
```

### 3.3 Registry API

```rust
pub struct BackendRegistry;
impl BackendRegistry {
    pub fn register<B: ProverBackend + VerifierBackend + 'static>(backend: B);
    pub fn list() -> Vec<BackendInfo>;
    pub fn get(id: &str) -> Option<Arc<dyn ProverBackend + VerifierBackend>>;
}
```

### 3.4 BackendInfo Schema (JSON)

```json
{
  "id": "winterfell@0.6",
  "fields": ["BabyBear", "Goldilocks"],
  "hashes": ["poseidon2", "rescue"],
  "fri": { "arities": [2,4,8], "grinding": true },
  "recursion": "none",
  "lookups": false
}
```

---

## 4. Configuration Schemas

### 4.1 Program Configuration (`.air` header)

```toml
name     = "merkle_root"
version  = "0.1"
field    = "Goldilocks"
hash     = "poseidon2"
backend  = "winterfell@0.6"
profile  = "balanced"

[columns]
main = 12
periodic = 2

[public]
[[public.input]]
name = "root"
type = "Vector(Field,16)"
binding = "Absorb"

[[public.input]]
name = "limit"
type = "U32"
binding = "LiftAsConstCols"
```

### 4.2 Profile Schema (`profiles/*.toml`)

```toml
id          = "balanced"
lambda_bits = 100
fri_blowup  = 16
fri_queries = 30
grind_bits  = 18
merkle_arity = 2
```

### 4.3 Backend Registry File (`backends/*.json`)

```json
{
  "id": "plonky2@0.2.3",
  "fields": ["Goldilocks"],
  "hashes": ["poseidon2"],
  "fri": { "arities": [2,4,8], "grinding": true },
  "recursion": "stark-in-stark",
  "lookups": true
}
```

---

## 5. Data Serialization

| Type           | Encoding               | Notes                             |
| -------------- | ---------------------- | --------------------------------- |
| Field elements | Little-endian 32 bytes | Deterministic                     |
| ProofBlob      | Binary                 | Version-prefixed header + payload |
| PublicInputs   | Canonical JSON         | Stable key order                  |
| Profiles       | TOML                   | Parsed, validated, cached         |
| Events         | JSONL                  | Append-only                       |

**Proof Header Layout (bytes):**

```
[0-3]   = 0x50524F46 ("PROF")
[4-7]   = version (u32)
[8-15]  = backend_id hash (u64)
[16-23] = profile_id hash (u64)
[24-31] = proof length (u64)
[32-…]  = compressed proof body
```

---

## 6. Event Interfaces

### 6.1 Emitted JSONL

```json
{"event":"BackendRegistered","id":"winterfell@0.6","fields":["BabyBear","Goldilocks"],"timestamp":"2025-10-07T00:00:00Z"}
{"event":"ProofGenerated","program":"merkle_root","backend":"winterfell","elapsed_ms":521,"mem_bytes":6422528}
{"event":"ProofVerified","program":"merkle_root","backend":"winterfell","verified":true}
```

### 6.2 SDK Event Stream

```rust
pub enum Event {
    BackendRegistered(BackendInfo),
    ProofGenerated { program: String, backend: String, stats: ProofStats },
    ProofVerified { program: String, backend: String, verified: bool },
}
```

---

## 7. REST / gRPC (Optional Future)

To support distributed proving or API service mode, endpoints follow this schema:

| Method             | Path                                                                        | Description |
| ------------------ | --------------------------------------------------------------------------- | ----------- |
| `POST /v0/prove`   | Start proof job; body = `{ program_id, backend, profile, public_inputs }`.  |             |
| `POST /v0/verify`  | Verify proof; body = `{ program_id, backend, proof_bytes, public_inputs }`. |             |
| `GET /v0/backends` | Return list of registered backends.                                         |             |
| `GET /v0/profiles` | Return available profiles.                                                  |             |

Responses mirror SDK types (JSON-serialized).

---

## 8. Deterministic Hashing Domains

| Domain       | Label  | Purpose                                     |
| ------------ | ------ | ------------------------------------------- |
| `ProgramID`  | `PROG` | Unique AIR hash including backend + profile |
| `BundleID`   | `BUND` | Hash of bundle code                         |
| `Transcript` | `TRSC` | Domain separation for Fiat–Shamir           |
| `PublicIO`   | `PUBI` | Canonical encoding of inputs/outputs        |

These tags prefix all BLAKE3 domain hashes to prevent cross-protocol collisions.

---

## 9. Testing Interfaces

### 9.1 CLI Tests

```bash
forge test --match "cli_prove_verify"
zkd prove -p tests/toy.air -i tests/input.json -b native --profile dev-fast
zkd verify -p tests/toy.air -i tests/input.json -b native --profile dev-fast -P proofs/toy.proof
```

Expected output:

```
✅ Proof verified successfully.
Stats: elapsed_ms=120, mem_bytes=1048576
```

### 9.2 SDK Tests

```rust
#[test]
fn roundtrip_proof_verification() {
    let program = load_program("tests/toy.air");
    let inputs = load_inputs("tests/input.json");
    let proof = prove(&program, &inputs).unwrap();
    assert!(verify(&program, &inputs, &proof).unwrap());
}
```

---

## 10. Rationale

This interface design enforces **strict determinism** and **inter-backend portability** while remaining friendly to automation (CLI + SDK + agent pipelines).
Every call has explicit inputs, structured outputs, and reproducible serialization rules.
By locking profiles, backends, and field types through versioned registries, proofs stay verifiable across environments and time.

**Mantra:** *“Same input, same output, any backend.”*

---
