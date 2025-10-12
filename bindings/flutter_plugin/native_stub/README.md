# zkprov-stub

This crate produces lightweight placeholder `libzkprov` shared libraries for
Flutter development. The artifacts expose the same FFI surface as the real
prover but return deterministic demo data so the sample UI can exercise the
`init → prove → verify` flow on Android without bundling the heavy native
runtime.

To rebuild the Android `.so` files:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target armv7-unknown-linux-gnueabihf
cargo build --release --target x86_64-unknown-linux-gnu
```

Copy the resulting `libzkprov.so` files into the matching
`android/src/main/jniLibs/<abi>/` folders in the Flutter plugin.
