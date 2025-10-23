# Bindings Cookbook (Go / .NET / Java-Kotlin / Swift)

Official Phase-0 support maintains the C ABI, Python package, Flutter/Dart plugin, Node/TypeScript addon, and WASI module. Integrations for Go, .NET, Java/Kotlin, and Swift are deferred until the Ecosystem phase.  
This cookbook captures the thin wrappers teams should build today to remain forward-compatible. It mirrors the ABI stability tests shipped with the repo and highlights memory ownership, error handling, and packaging guidance.

## 0. Shared Rules

* Link against the canonical header at `include/zkprov.h` and the shared/static library emitted by `cargo build -p zkprov-ffi-c --release`.
* Treat the C ABI as the contract: exported symbols include `zkp_version`, `zkp_init`, `zkp_list_backends`, `zkp_list_profiles`, `zkp_prove`, `zkp_verify`, `zkp_alloc`, and `zkp_free`.
* All fallible calls return an `int32_t` status (`ZKP_OK == 0`). When a call succeeds, any out-parameters pointing to prover-owned memory must eventually be freed with `zkp_free`.  
  The JSON payloads returned by `zkp_prove`/`zkp_verify` embed the digest `D`, proof length, and structured error messages.
* Determinism guarantees anchor at the proof layer. Cross-language parity requires comparing the `digest` field in the metadata envelope against CLI/SDK results.
* Use the ABI stability test harness (`cargo test -p zkprov-ffi-c abi`) as a conformance checklist. Downstream bindings should add equivalent smoke tests before publishing packages.

---

## 1. Go (cgo)

### 1.1 Build Flags

```go
// #cgo CFLAGS: -I${SRCDIR}/../include
// #cgo LDFLAGS: -L${SRCDIR}/../target/release -lzkprov
// #include "zkprov.h"
import "C"
```

Ship prebuilt `libzkprov` artifacts alongside the Go module or instruct users to set `LD_LIBRARY_PATH`/`DYLD_LIBRARY_PATH`.

### 1.2 Memory & Error Handling

* Wrap prover-owned pointers with `runtime.SetFinalizer` to call `C.zkp_free`.
* Convert JSON envelopes using `encoding/json`; map `code`/`msg` fields into Go `error` types.
* Guard against zero-length buffers: when `out_proof_len` is `0`, the pointer must be treated as `nil`.

### 1.3 Minimal Round-Trip

```go
type Proof struct {
	Digest   string `json:"digest"`
	ProofLen uint64 `json:"proof_len"`
}

func Prove(cfg Config, airPath, publicInputs string) ([]byte, Proof, error) {
	var proofPtr *C.uint8_t
	var proofLen C.uint64_t
	var metaPtr *C.char

	status := C.zkp_prove(
		C.CString(cfg.Backend),
		C.CString(cfg.Field),
		C.CString(cfg.Hash),
		C.uint32_t(cfg.FriArity),
		C.CString(cfg.Profile),
		C.CString(airPath),
		C.CString(publicInputs),
		(**C.uint8_t)(unsafe.Pointer(&proofPtr)),
		(*C.uint64_t)(unsafe.Pointer(&proofLen)),
		(**C.char)(unsafe.Pointer(&metaPtr)),
	)
	if status != C.ZKP_OK {
		return nil, Proof{}, fmt.Errorf("zkp_prove failed: %d", int(status))
	}
	defer C.zkp_free(unsafe.Pointer(proofPtr))
	defer C.zkp_free(unsafe.Pointer(metaPtr))

	meta := C.GoString(metaPtr)
	var envelope struct {
		Ok bool  `json:"ok"`
		Proof
	}
	if err := json.Unmarshal([]byte(meta), &envelope); err != nil || !envelope.Ok {
		return nil, Proof{}, fmt.Errorf("invalid prove metadata: %s", meta)
	}

	proof := C.GoBytes(unsafe.Pointer(proofPtr), C.int(proofLen))
	return proof, envelope.Proof, nil
}
```

The verify path mirrors `Prove`, ensuring `proofPtr` and `metaPtr` stay valid for the duration of the call. Always compare the returned digest with the Prove metadata.

---

## 2. .NET (P/Invoke)

### 2.1 P/Invoke Signatures

```csharp
internal static class Native {
    private const string Lib = "zkprov";

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int zkp_version(ref IntPtr outJson);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int zkp_init();

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int zkp_list_backends(ref IntPtr outJson);
    // ... list_profiles omitted for brevity

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int zkp_prove(
        string backend,
        string field,
        string hash,
        uint friArity,
        string profile,
        string airPath,
        string publicInputs,
        ref IntPtr outProof,
        ref ulong outProofLen,
        ref IntPtr outJsonMeta);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int zkp_verify(
        string backend,
        string field,
        string hash,
        uint friArity,
        string profile,
        string airPath,
        string publicInputs,
        IntPtr proofPtr,
        ulong proofLen,
        ref IntPtr outJsonMeta);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void zkp_free(IntPtr ptr);
}
```

Package `libzkprov` under `runtimes/<rid>/native/` to let the .NET host resolve the shared library automatically.

### 2.2 SafeHandle Wrapper

```csharp
sealed class ZkProvBuffer : SafeHandle {
    ZkProvBuffer() : base(IntPtr.Zero, true) {}
    public override bool IsInvalid => handle == IntPtr.Zero;
    protected override bool ReleaseHandle() {
        Native.zkp_free(handle);
        return true;
    }
}
```

Marshal the proof buffer into a managed `byte[]` via `Span<byte>` and ensure `SafeHandle.DangerousGetHandle()` scopes are minimal.

### 2.3 Metadata Envelope

Use `System.Text.Json` to parse:

```csharp
record ProveMeta(bool ok, string digest, ulong proof_len);
```

The verify metadata returns `{ "ok": true, "verified": true, "digest": "0x..." }`. Treat non-`ok` responses as exceptions with the included `msg` field.

---

## 3. Java / Kotlin (JNI or JNA)

### 3.1 Fast Path: JNA

For quick integrations, JNA avoids manual JNI glue:

```java
public interface ZkProvLib extends Library {
    ZkProvLib INSTANCE = Native.load("zkprov", ZkProvLib.class);

    int zkp_init();
    int zkp_version(PointerByReference outJson);
    int zkp_prove(
        String backend,
        String field,
        String hash,
        int friArity,
        String profile,
        String airPath,
        String publicInputs,
        PointerByReference outProof,
        LongByReference outProofLen,
        PointerByReference outJsonMeta);

    int zkp_verify(
        String backend,
        String field,
        String hash,
        int friArity,
        String profile,
        String airPath,
        String publicInputs,
        Pointer proofPtr,
        long proofLen,
        PointerByReference outJsonMeta);

    void zkp_free(Pointer ptr);
}
```

Release every returned pointer via `zkp_free` using `Pointer.nativeValue()`. Convert the proof `Pointer` into a `byte[]` with `proofPtr.getByteArray(0, (int) proofLen)`.

### 3.2 JNI Appendix

High-throughput Android apps may prefer JNI:

* Generate headers with `javac -h`.
* Pin `jstring` parameters via `GetStringUTFChars` and call the C functions.
* Wrap output pointers in `NewDirectByteBuffer` for zero-copy access and call `zkp_free` during `Cleaner` finalization.
* Package the native libraries in `lib/arm64-v8a` inside an AAR; ensure `packagingOptions.pickFirst` includes each architecture.

### 3.3 Android Packaging

* Add `jniLibs` for `arm64-v8a` and optionally `x86_64` (simulators).
* Handle notarization / codesign via Gradle `packagingOptions`.
* Confirm `android:extractNativeLibs="true"` for API 30+ if bundling the `.so`.

---

## 4. Swift / iOS (SwiftPM System Library)

### 4.1 Package Layout

`Package.swift` excerpt:

```swift
// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "ZkProv",
    platforms: [.iOS(.v15), .macOS(.v13)],
    products: [
        .library(name: "ZkProv", targets: ["ZkProv"])
    ],
    targets: [
        .systemLibrary(
            name: "Czkprov",
            pkgConfig: "zkprov",
            providers: [
                .brew(["zkprov"]),
                .apt(["libzkprov-dev"])
            ]
        ),
        .target(
            name: "ZkProv",
            dependencies: ["Czkprov"],
            resources: [.copy("Resources")]
        ),
        .testTarget(name: "ZkProvTests", dependencies: ["ZkProv"])
    ]
)
```

Ship a `.xcframework` containing `libzkprov` and add a module map:

```text
module Czkprov [system] {
  header "zkprov.h"
  link "zkprov"
}
```

### 4.2 Wrapper Helpers

```swift
public struct ProofMeta: Decodable {
    public let ok: Bool
    public let digest: String
    public let proof_len: UInt64?
    public let verified: Bool?
}

public enum ZkProvError: Error {
    case ffi(Int32, String)
    case decode(String)
}

public func prove(config: Config, airPath: String, publicInputs: String) throws -> (Data, ProofMeta) {
    var proofPtr: UnsafeMutablePointer<UInt8>? = nil
    var proofLen: UInt64 = 0
    var metaPtr: UnsafeMutablePointer<CChar>? = nil

    let status = zkp_prove(
        config.backend, config.field, config.hash,
        config.friArity, config.profile,
        airPath, publicInputs,
        &proofPtr, &proofLen, &metaPtr)
    guard status == ZKP_OK.rawValue, let proofBase = proofPtr, let metaBase = metaPtr else {
        let msg = metaPtr.flatMap { String(cString: $0) } ?? "unknown"
        throw ZkProvError.ffi(status, msg)
    }
    defer {
        zkp_free(proofBase)
        zkp_free(metaBase)
    }

    let metaString = String(cString: metaBase)
    guard let meta = try? JSONDecoder().decode(ProofMeta.self, from: Data(metaString.utf8)), meta.ok else {
        throw ZkProvError.decode(metaString)
    }
    let proof = Data(bytes: proofBase, count: Int(proofLen))
    return (proof, meta)
}
```

Use `withCString` wrappers when converting Swift `String` values. For iOS, bundle the `.xcframework` and specify `VALIDATE_WORKSPACE = YES` within Xcode to ensure the library loads at runtime.

---

## 5. Conformance Checklist

1. Call `zkp_version` at startup and log the returned semantic version during integration.  
2. Run the toy AIR round-trip (`examples/air/toy.air`, `{"a":1,"b":[2,3]}`) and capture the digest `D`.  
3. Verify the same proof across CLI/SDK and the DIY binding; diffs indicate serialization drift.  
4. Run the ABI stability tests locally: `cargo test -p zkprov-ffi-c abi`. Ensure your bindings compile against the same header commit.  
5. Document build flags, dynamic loader hints, and `zkp_free` ownership semantics in your package README.  
6. When publishing packages, link to this cookbook and describe your conformance harness so downstream users can reproduce results.

Bindings that satisfy this checklist can evolve independently while remaining compatible with future official releases.
