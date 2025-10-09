const fs = require('fs');
const path = require('path');

function resolveCandidates() {
  const candidates = [];

  const releasePath = path.join(__dirname, 'build', 'Release', 'zkprov.node');
  candidates.push(releasePath);

  const debugPath = path.join(__dirname, 'build', 'Debug', 'zkprov.node');
  candidates.push(debugPath);

  const rootPath = path.join(__dirname, 'zkprov.node');
  candidates.push(rootPath);

  const platform = process.platform;
  const arch = process.arch;
  const prebuildsDir = path.join(__dirname, 'prebuilds');
  const dirCandidates = new Set();
  dirCandidates.add(`${platform}-${arch}`);

  if (fs.existsSync(prebuildsDir)) {
    try {
      for (const entry of fs.readdirSync(prebuildsDir)) {
        if (entry.startsWith(`${platform}-${arch}`)) {
          dirCandidates.add(entry);
        }
      }
    } catch (_err) {
      // Ignore directory read errors and continue with default candidates.
    }
  }

  const abi = process.versions && process.versions.modules ? process.versions.modules : undefined;
  const napi = process.versions && process.versions.napi ? process.versions.napi : undefined;
  const filenames = new Set(['node.napi.node', 'node-napi.node']);

  if (abi) {
    filenames.add(`node.abi${abi}.node`);
    filenames.add(`node-${abi}.node`);
    filenames.add(`node.v${abi}.node`);
    filenames.add(`node-v${abi}.node`);
  }

  if (napi) {
    filenames.add(`napi-v${napi}.node`);
    filenames.add(`node.napi.${napi}.node`);
  }

  if (fs.existsSync(prebuildsDir)) {
    for (const dir of dirCandidates) {
      const fullDir = path.join(prebuildsDir, dir);
      for (const filename of filenames) {
        candidates.push(path.join(fullDir, filename));
      }
    }
  }

  return Array.from(new Set(candidates));
}

function loadNativeBinding() {
  const details = [];

  for (const candidate of resolveCandidates()) {
    if (!fs.existsSync(candidate)) {
      details.push(`- ${candidate} (not found)`);
      continue;
    }

    try {
      const binding = require(candidate);
      return binding;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      details.push(`- ${candidate} (load error: ${message})`);
    }
  }

  const errorLines = [
    'Failed to load the zkprov native module.',
    'Searched in the following locations:',
    ...details,
  ];

  throw new Error(errorLines.join('\n'));
}

let nativeBinding;
let nativeLoadError;

try {
  nativeBinding = loadNativeBinding();
} catch (err) {
  nativeLoadError = err instanceof Error ? err : new Error(String(err));
}

function getBinding() {
  if (nativeBinding) {
    return nativeBinding;
  }

  throw nativeLoadError || new Error('Failed to load the zkprov native module.');
}

async function listBackends() {
  return getBinding().listBackends();
}

async function listProfiles() {
  return getBinding().listProfiles();
}

async function prove(cfg) {
  return getBinding().prove(cfg);
}

async function verify(cfg, proof) {
  return getBinding().verify(cfg, proof);
}

module.exports = {
  listBackends,
  listProfiles,
  prove,
  verify,
};
