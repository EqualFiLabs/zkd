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

```bash
npm run build
```
