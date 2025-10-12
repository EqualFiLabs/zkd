# zkprov_flutter

ZKProv Dart FFI bindings packaged as a Flutter plugin. This package provides the Dart-side entry point for loading native libraries via `dart:ffi` and ships with platform folders for bundling `.so` and `.xcframework` artifacts alongside a demo app under `example/`.

## Native library setup

The plugin does not build the native prover libraries automatically. Copy the prebuilt artifacts from the Rust workspace into the Flutter plugin before running the example or publishing the package.

1. Build the shared/static libraries from the repository root:
   ```bash
   # Android
   cargo ndk -t arm64-v8a -o target/android/release build -p zkprov-ffi-c --release

   # iOS (device + simulator)
   cargo build -p zkprov-ffi-c --target aarch64-apple-ios --release
   cargo build -p zkprov-ffi-c --target aarch64-apple-ios-sim --release
   ```
2. Copy the resulting binaries into the plugin:
   ```bash
   # Android: place the .so in jniLibs/arm64-v8a
   cp target/android/release/arm64-v8a/release/libzkprov.so \
     bindings/flutter_plugin/android/src/main/jniLibs/arm64-v8a/

   # iOS: bundle slices into an XCFramework
   xcodebuild -create-xcframework \
     -library target/aarch64-apple-ios/release/libzkprov.a \
     -library target/aarch64-apple-ios-sim/release/libzkprov.a \
     -output bindings/flutter_plugin/ios/ZkProv.xcframework
   ```

### Supported ABIs

- **Android:** `arm64-v8a`
- **iOS:** `ios-arm64` (device) and `ios-sim-arm64` (simulator)

Additional Android architectures can be produced with more `cargo ndk -t <abi>` invocations as needed.

## Usage

Import the generated bindings and call `prove`/`verify` with a `ZkProvConfig`. The digest `D` returned by both calls is deterministic for a given `(program, backend, profile, inputs)` tuple.

```dart
import 'package:zkprov_flutter/zkprov_ffi.dart';

final cfg = ZkProvConfig(
  backendId: 'native@0.0',
  field: 'Prime254',
  hashId: 'blake3',
  friArity: 2,
  profileId: 'balanced',
  airPath: '/path/to/program.air',
  publicInputsJson: '{"demo":true,"n":7}',
);

Future<void> demo() async {
  final proveResult = await prove(cfg);
  final proof = proveResult.proof;
  final digest = proveResult.digest; // Deterministic D

  final verifyResult = await verify(cfg, proof);
  assert(verifyResult.verified, 'Proof should verify');
}
```

Run Flutter maintenance commands from the plugin directory during development:

```bash
flutter pub get
flutter analyze
```
