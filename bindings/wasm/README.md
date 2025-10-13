# zkprov WASM bindings

## Usage

### Node

```js
import { init } from '@zkprov/wasm/loader.js';
const z = await init({
  wasmPath: require.resolve('@zkprov/wasm/zkprov_wasi.wasm'),
  preopens: { '/work': process.cwd() },
});
const { proof, meta } = await z.prove(cfg);
const vr = await z.verify(cfg, proof);
```

### Browser

- Include `zkprov_wasi.wasm` and `loader.js` under your static assets.
- Use a WASI polyfill; the loader auto-imports `@wasmer/wasi` from a CDN by default (you can vendor it locally).
- Ensure the AIR file is reachable via the preopened path you pass to `init`.

