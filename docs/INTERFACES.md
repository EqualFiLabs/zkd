---

# **Interfaces — General-Purpose STARK Prover**

**Parent RFC:** RFC-ZK01 v0.3
**Purpose:** Formalize every external and internal interface exposed by the proving system, including the CLI, SDK, backend trait contracts, and JSON/TOML schemas.
**Status:** Draft → Frozen once the Phase 0 implementation compiles.

---

## 1. CLI Interfaces (`zkd`)

### 1.1 Commands

| Command              | Description                                                        |
| -------------------- | ------------------------------------------------------------------ |
| `zkd init`           | Scaffold a new proving workspace with default config and profiles. |
| `zkd compile`        | Compile a `.yaml` AIR into canonical `.air` binary form.           |
| `zkd prove`          | Build trace, resolve backend via capabilities, and emit proof blob.|
| `zkd prove --profile`| Run proof with a named profile bundle (e.g., `dev-fast`).         |
| `zkd verify`         | Re-run transcript and verify proof deterministically.              |
| `zkd verify --manifest` | Verify proof bytes using determinism manifest JSON.           |
| `zkd vector validate`| Run Golden Vector parity validation across registered backends.   |
| `zkd io schema`      | Display the declared public input/output schema of a program.      |
| `zkd profile ls`     | List all available proof-profile presets.                          |
| `zkd backend ls`     | Enumerate registered backend adapters and their capabilities.      |

### 1.2 Syntax Examples

```bash
# Compile YAML AIR into binary IR
zkd compile specs/balance.yml -o build/balance.air

# Prove a program with Winterfell backend
zkd prove \
  -p programs/merkle.air \
  -i inputs/merkle.json \
  --profile dev-fast \
  -o proofs/merkle-balanced.proof

# Verify the proof
zkd verify \
  -p programs/merkle.air \
  -i inputs/merkle.json \
  -b winterfell@0.6 \
  --profile balanced \
  -P proofs/merkle-balanced.proof

# Verify using determinism manifest JSON
zkd verify \
  --manifest proofs/merkle-balanced.proof.json \
  -P proofs/merkle-balanced.proof

# Run Golden Vector parity validation
zkd vector validate --root tests/golden_vectors
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

> **Embedding note:** Applications embedding the prover from other languages should see §3 for the C ABI and bindings that mirror these CLI workflows.

### 1.5 Golden Vector Validation

`zkd vector validate` traverses the [Golden Vector Registry](./golden-vectors.md), regenerates proofs for each registered backend, and compares digests and determinism manifests.
The command returns non-zero if any backend diverges or if the determinism vector hash fails validation.
Reports emit `vector_passed` and `manifest_hash` fields for downstream CI aggregation.

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

> **Embedding note:** Higher-level bindings in other languages wrap these SDK concepts via the C ABI outlined in §3.

> **YAML import:** `AirProgram::load_from_file("balance.yml")` accepts both `.air` and `.yaml` sources, compiling YAML deterministically into AIR-IR before proving.
> See the [commitment-aware AIR parser](../crates/corelib/src/air/parser.rs) for DSL examples that map directly onto the `AirIr` structs.

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

> **Embedding note:** The same JSON structure is emitted by the FFI boundary when the C ABI functions return fallible results; see §3.

---

## 3. C ABI & Multi-Language Bindings

The prover exports a stable C ABI so that non-Rust applications can embed the engine via shared libraries. Official Phase-0 bindings maintained by the core team cover Python (ctypes/cffi), Flutter/Dart (platform plugin), Node/TypeScript (N-API addon), and WASI (browser/serverless module) in addition to the raw C surface.

> **Availability:** Go (cgo), .NET (P/Invoke), Java/Kotlin (JNI + AAR), and Swift/iOS (SwiftPM) integrations are deferred to the Ecosystem phase. Teams needing them today should rely on the DIY bindings cookbook; these targets are non-normative for determinism guarantees, which remain anchored at the C ABI.

### 3.1 Exported Symbols

| Function | Signature (C) | Description |
| -------- | ------------- | ----------- |
| `zkp_init` | `zkp_error* zkp_init(const char* runtime_json, zkp_context** out_ctx);` | Initializes the prover runtime using a UTF-8 JSON configuration. Returns `NULL` on success or an error pointer otherwise. |
| `zkp_prove` | `zkp_error* zkp_prove(zkp_context* ctx, const char* request_json, zkp_buffer* out_proof);` | Builds traces, runs the selected backend, and writes the proof bytes/statistics into `out_proof`. |
| `zkp_verify` | `zkp_error* zkp_verify(zkp_context* ctx, const char* request_json, const uint8_t* proof_ptr, size_t proof_len);` | Replays the transcript and verifies the supplied proof blob. |
| `zkp_list_backends` | `const char* zkp_list_backends(zkp_context* ctx);` | Returns a JSON string describing registered backends and capabilities. Caller frees via `zkp_free`. |
| `zkp_list_profiles` | `const char* zkp_list_profiles(zkp_context* ctx);` | Returns JSON describing available profiles. |
| `zkp_version` | `int32_t zkp_version(char **out_json);` | Allocates a JSON envelope containing semantic version (and optional git hash). Caller frees via `zkp_free`. |
| `zkp_set_callback` | `void zkp_set_callback(zkp_context* ctx, zkp_event_cb cb, void* user_data);` | Registers a callback invoked for JSONL progress messages. |
| `zkp_cancel` | `void zkp_cancel(zkp_context* ctx);` | Requests cancellation of any in-flight proving job. |
| `zkp_free` | `void zkp_free(const void* ptr);` | Releases memory allocated by the prover (strings, buffers). |

`zkp_buffer` is an opaque struct containing `uint8_t* ptr` + `size_t len`. All UTF-8 parameters use canonical, NUL-terminated `const char*` buffers.

### 3.2 Error Model

All fallible functions return `NULL` on success. Errors are conveyed as heap-allocated UTF-8 JSON strings describing the failure:

```json
{ "error": "CapabilityMismatch", "message": "hash=keccak not supported by plonky2" }
```

Bindings unwrap these into native error types. The caller must release the returned pointer via `zkp_free`, including helper calls such as `zkp_version`.

### 3.3 Memory Management

Memory allocated by the prover (proof buffers, JSON strings, error messages) must be released with `zkp_free`. Host applications must not free these pointers with their language runtime allocators to avoid mismatched heaps. Conversely, buffers owned by the host remain owned by the host.

### 3.4 Event Callbacks

`zkp_set_callback(cb, user_data)` registers a callback receiving newline-delimited JSON messages:

```json
{ "stage": "prove", "percent": 42, "elapsed_ms": 103 }
```

The callback executes on internal worker threads. Bindings surface these events as async streams, loggers, or progress hooks. `user_data` is passed through verbatim to facilitate context pointers.

### 3.5 Thread Safety

`zkp_init` returns a thread-safe context. Concurrent calls to `zkp_prove`, `zkp_verify`, `zkp_list_*`, and `zkp_version` are supported as long as each proof invocation uses disjoint `zkp_buffer` outputs. Callback registration is thread-safe but should be performed during initialization to avoid races.

### 3.6 Usage Example

```c
#include "zkprov.h"

int main(void) {
    zkp_context* ctx = NULL;
    if (zkp_init("{\"log_level\":\"info\"}", &ctx)) {
        fprintf(stderr, "failed to init prover\n");
        return 1;
    }

    zkp_buffer proof = {0};
    const char* err = zkp_prove(ctx, "{\"program\":\"tests/fixtures/toy.air\"}", &proof);
    if (err) {
        fprintf(stderr, "%s\n", err);
        zkp_free(err);
        return 1;
    }

    err = zkp_verify(ctx, "{\"program\":\"tests/fixtures/toy.air\"}", proof.ptr, proof.len);
    if (err) {
        fprintf(stderr, "%s\n", err);
        zkp_free(err);
    }

    zkp_free(proof.ptr);
    zkp_free(ctx);
    return err ? 1 : 0;
}
```

Language bindings publish ergonomic wrappers around these calls:

* **Node/TypeScript** *(official)* — N-API addon exposing async `prove()`/`verify()` Promises and `loadProgram("*.yaml")` helpers.
* **Python** *(official)* — `ctypes`/`cffi` layer returning `dict` objects, `bytes` buffers, and `compile_yaml("balance.yml")` utilities.
* **Flutter/Dart** *(official)* — Dart FFI plugin wrapping the C ABI for Android/iOS with platform channel helpers.
* **WASI/WebAssembly** *(official)* — thin JS/Wasm glue calling the same exported functions for browser/runtime targets.
* **Go** *(DIY via cookbook; deferred)* — `cgo` package returning Go errors and slices.
* **.NET** *(DIY via cookbook; deferred)* — P/Invoke declarations mapping to `SafeHandle` wrappers.
* **Java/Kotlin** *(DIY via cookbook; deferred)* — JNI or JNA bridge with Android packaging guidance.
* **Swift/iOS** *(DIY via cookbook; deferred)* — SwiftPM `systemLibrary` target bridging the C ABI.

DIY bindings must continue to free all prover-owned buffers via `zkp_free` and follow the JSON error contract to stay compatible with future releases.

### 3.7 EVM ABI Helpers

The FFI exposes helper functions for Solidity interoperability:

| Function | Signature | Purpose |
| -------- | --------- | ------- |
| `zkp_keccak_commit` | `zkp_error* zkp_keccak_commit(zkp_context*, const uint8_t* preimage, size_t len, uint8_t out_digest[32]);` | Mirrors the `KeccakCommit` gadget and emits a Solidity-ready digest. |
| `zkp_evm_log_proof` | `zkp_error* zkp_evm_log_proof(zkp_context*, const char* proof_json, zkp_buffer* out_abi);` | Wraps `EvmLogProof` outputs into ABI-encoded payloads. |
| `zkp_verifier_stub_digest` | `zkp_error* zkp_verifier_stub_digest(const uint8_t* proof_bytes, size_t len, uint8_t out_digest[32]);` | Produces the digest consumed by the on-chain `VerifierStub`. |

Language bindings surface these helpers as convenience wrappers for contract deployments and Foundry tests.

---

## 4. Backend Adapter Interfaces

### 4.1 Trait Definitions

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

### 4.2 Capability Structure

```rust
pub struct Capabilities {
    pub fields: Vec<FieldId>,
    pub hashes: Vec<HashId>,
    pub fri_arities: Vec<u32>,
    pub recursion: RecursionMode,
    pub lookups: bool,
    pub curves: Vec<CurveId>,     // e.g. ["jubjub","pallas"]
    pub pedersen: bool,
    pub keccak: bool,
}
```

### 4.3 Registry API

```rust
pub struct BackendRegistry;
impl BackendRegistry {
    pub fn register<B: ProverBackend + VerifierBackend + 'static>(backend: B);
    pub fn list() -> Vec<BackendInfo>;
    pub fn get(id: &str) -> Option<Arc<dyn ProverBackend + VerifierBackend>>;
}
```

### 4.4 BackendInfo Schema (JSON)

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

### 4.4 Commitment Binding Example

```toml
[[public.input]]
name = "amount_commit"
type = "Point"
binding = "Pedersen(jubjub)"
```

This expands to two field elements `(Cx,Cy)` and enforces curve membership via the Pedersen bundle.

---

## 5. Data Serialization

| Type           | Encoding               | Notes                             |
| -------------- | ---------------------- | --------------------------------- |
| Field elements | Little-endian 32 bytes | Deterministic                     |
| ProofBlob      | Binary                 | Version-prefixed header + payload |
| PublicInputs   | Canonical JSON         | Stable key order                  |
| Profiles       | TOML                   | Parsed, validated, cached         |
| Events         | JSONL                  | Append-only                       |
| Curve point    | `(Cx,Cy)` little-endian fields | For Pedersen commitments        |
| Keccak digest  | 32-byte big-endian     | ABI-compatible with Solidity      |

**Proof Header Layout (bytes):**

```
[0-3]   = 0x50524F46 ("PROF")
[4-7]   = version (u32)
[8-15]  = backend_id hash (u64)
[16-23] = profile_id hash (u64)
[24-31] = proof length (u64)
[32-…]  = compressed proof body
```

### 5.1 Proof JSON Schema

`zkd prove --stats` and `zkd verify --manifest` emit JSON containing the determinism vector:

```json
{
  "program": "balance_check",
  "backend": "winterfell@0.6",
  "profile": "dev-fast",
  "digest": "0xabc123...",
  "determinism_vector": {
    "compiler_commit": "0a1b2c3d",
    "backend": "winterfell@0.6",
    "system": "linux-x86_64",
    "seed": "0001020304050607",
    "manifest_hash": "d3e4f5"
  },
  "vector_passed": true,
  "elapsed_ms": 128
}
```

Consumers must persist the determinism vector and manifest hash to reproduce proofs and to satisfy Golden Vector parity checks.

---

### Application Profiles & Presets

In addition to generic AIR programs, the prover ships a library of **pre-baked application profiles** encoding common zero-knowledge constructions.
Each profile defines:

* **Manifest** — describes the AIR, field modulus, public-input schema, and gadget bindings.
* **Presets** — safe default parameter sets (`fast`, `balanced`, `tight`, `mobile`).
* **Profile ID** — unique identifier (e.g., `zk-auth-pedersen-secret`, `zk-proof-of-solvency-lite`).

#### Typical Application Profiles

| ID | Description | Gadgets | Public Inputs |
|----|--------------|----------|---------------|
| `zk-auth-pedersen-secret` | Passwordless secret authentication bound to `(nonce, origin)` | PedersenCommit, PoseidonBind | `C`, `nonce`, `origin`, `nullifier` |
| `zk-allowlist-merkle` | Merkle allowlist membership plus session binding | MerklePathVerify, PoseidonBind | `root`, `pk_hash`, `path`, `nonce`, `origin` |
| `zk-attr-range` | Attribute within declared bounds | RangeCheck, PedersenCommit | `commitment`, `min`, `max` |
| `zk-balance-geq` | Balance ≥ threshold with optional adapter proof | RangeCheck, PedersenCommit | `commitment`, `threshold`, `adapter_proof` |
| `zk-uniqueness-nullifier` | One action per epoch using nullifier | PoseidonNullifier, PoseidonBind | `nullifier`, `epoch` |
| `zk-proof-of-solvency-lite` | Asset/liability delta commitment | MerklePathVerify, PedersenCommit, RangeCheck | `asset_root`, `liability_root`, `delta_commitment` |
| `zk-vote-private` | Private ballot casting from allowlist | MerklePathVerify, PedersenCommit, PoseidonBind | `root`, `vote_commitment`, `nonce`, `tally_binding` |
| `zk-file-hash-inclusion` | Document hash inclusion in Merkle root | MerklePathVerify, PoseidonBind | `root`, `file_hash`, `path` |
| `zk-score-threshold` | Hidden score ≥ threshold tied to epoch | PedersenCommit, RangeCheck | `commitment`, `threshold`, `epoch` |
| `zk-age-over` | Mobile-optimized age ≥ bound proof | RangeCheckLite, PedersenCommit | `commitment`, `bound` |

#### Preset Customization

Developers may override any preset via CLI/SDK parameters:

```bash
zkd prove --profile zk-auth-pedersen-secret --preset tight
```

or programmatically:

```ts
prove({ profile_id: "zk-auth-pedersen-secret", options: { preset: "mobile" } });
```

Presets reference performance profiles in `/profiles/*.toml` and map to deterministic resource limits.
Advanced users can clone and edit manifests for full customization.

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

Commitment and privacy bindings preserve determinism: all digests are canonical and reproducible across backends. By standardizing point encodings and domain tags, proofs remain verifiable and portable across devices and chains.

**Mantra:** *“Same input, same output, any backend.”*

---

Aligned with RFC-ZK01 v0.3 — Deterministic, Composable, Backend-Agnostic.
