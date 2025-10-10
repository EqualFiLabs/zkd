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
