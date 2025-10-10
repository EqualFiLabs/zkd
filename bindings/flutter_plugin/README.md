# zkprov_flutter

ZKProv Dart FFI bindings packaged as a Flutter plugin. This package provides the Dart-side entry point for loading native libraries via `dart:ffi` and ships with platform-specific folders for bundling `.so` and `.xcframework` artifacts.

## Structure

- `lib/zkprov_ffi.dart`: placeholder for the generated Dart FFI bindings.
- `android/src/main/jniLibs/`: location for bundled Android shared libraries.
- `ios/`: location for bundled iOS XCFrameworks and Podspec configuration.
- `example/`: minimal Flutter app showcasing how to depend on the plugin.

## Development

Run the usual Flutter maintenance commands from the plugin directory:

```bash
flutter pub get
flutter analyze
```

The example app includes a widget test that verifies the placeholder UI renders correctly.

### Building native libraries

The plugin expects prebuilt native artifacts to be copied into the platform folders before
publishing. The following commands illustrate how to build the libraries from the Rust
workspace and place them where Flutter will bundle them.

#### Android

Build the Android shared object for the desired ABI using `cargo-ndk`:

```bash
cargo ndk -t arm64-v8a -o target/android/release build -p zkprov-ffi-c --release
```

Copy the resulting `libzkprov.so` into `android/src/main/jniLibs/<abi>/`. For example:

```bash
cp target/android/release/arm64-v8a/release/libzkprov.so \
  bindings/flutter_plugin/android/src/main/jniLibs/arm64-v8a/
```

Repeat the build step with additional `-t` flags (`armeabi-v7a`, `x86_64`, etc.) if other
architectures are required.

#### iOS

Produce the iOS static libraries with `cargo` and assemble them into an XCFramework. One
option is to use `cargo-apple` or `cargo-xcode` (or a manual `xcodebuild` invocation) to
create slices for both `ios-arm64` and `ios-sim-arm64`, then run:

```bash
xcodebuild -create-xcframework \
  -library path/to/ios-arm64/libzkprov.a \
  -library path/to/ios-sim-arm64/libzkprov.a \
  -output bindings/flutter_plugin/ios/ZkProv.xcframework
```

Ensure that the generated `ZkProv.xcframework` replaces the placeholder files checked into
this repository.
