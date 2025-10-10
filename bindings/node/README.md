# @zkprov/node

Node.js bindings for the zkProv proving system. This package exposes the native N-API addon alongside a lightweight TypeScript wrapper.

## Installation

```bash
npm install @zkprov/node
```

## Usage

```js
const zkprov = require('@zkprov/node');

async function main() {
  await zkprov.ready();
  // Use the addon APIs here
}

main().catch(console.error);
```

## Building from source

Building requires a Rust toolchain because the native addon links against the
`zkprov-ffi-c` static library. The prebuild workflow automates this step and
sets up the environment required by `prebuildify`.

On Windows the native build uses MSVC, so the Rust archive must be produced for
the `x86_64-pc-windows-msvc` target (for example via
`cargo build --release --target x86_64-pc-windows-msvc -p zkprov-ffi-c`). This
ensures `cargo` emits a `.lib` file that the Node.js build can link against.

```bash
# Install dependencies without triggering a native rebuild yet
npm install --ignore-scripts

# Compile the Rust static library and generate platform-specific binaries
npm run prebuild

# Verify that the loader can fall back to the prebuild
rm -f build/Release/zkprov.node
node -e "require('./').listBackends().then(v=>console.log('ok', Boolean(v))).catch(e=>{console.error(e);process.exit(1)})"
```

The prebuild script runs `cargo build --release -p zkprov-ffi-c` under the hood
and exports the `ZKPROV_STATIC` variable to point at the resulting archive. If
the static artifact is missing, the script will abort with a clear message about
the path it tried to use.
