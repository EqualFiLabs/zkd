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

// When bundling for browsers (or whenever a preopen path is inconvenient) you
// can provide the AIR and inputs directly:
const air = await fs.promises.readFile('examples/air/toy.air');
const { proof: proof2 } = await z.proveFromBuffers(cfg, air, { demo: true, n: 7 });
const verify2 = await z.verifyFromBuffers(cfg, air, { demo: true, n: 7 }, proof2);
console.log('verified?', verify2.verified);
```

### Browser

- Include `zkprov_wasi.wasm` and `loader.js` under your static assets.
- Use a WASI polyfill; the loader auto-imports `@wasmer/wasi` from a CDN by default (you can vendor it locally).
- Ensure the AIR file is reachable via the preopened path you pass to `init`.

