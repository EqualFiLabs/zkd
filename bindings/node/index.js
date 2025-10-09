const fs = require('fs');
const path = require('path');

function loadNativeBinding() {
  const details = [];

  const tryLoad = (candidate) => {
    if (!fs.existsSync(candidate)) {
      details.push(`- ${candidate} (not found)`);
      return undefined;
    }

    try {
      return require(candidate);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      details.push(`- ${candidate} (load error: ${message})`);
      return undefined;
    }
  };

  const releasePath = path.join(__dirname, 'build', 'Release', 'zkprov.node');
  const releaseBinding = tryLoad(releasePath);
  if (releaseBinding) {
    return releaseBinding;
  }

  const platformMap = {
    win32: 'win32',
    darwin: 'darwin',
    linux: 'linux',
  };
  const archMap = {
    x64: 'x64',
    arm64: 'arm64',
  };

  const platform = platformMap[process.platform];
  const arch = archMap[process.arch];

  if (!platform || !arch) {
    const unsupportedParts = [];
    if (!platform) {
      unsupportedParts.push(`platform "${process.platform}"`);
    }
    if (!arch) {
      unsupportedParts.push(`architecture "${process.arch}"`);
    }
    const unsupported = unsupportedParts.join(' and ');
    details.push(
      `- prebuilds/${process.platform}-${process.arch} (unsupported ${unsupported})`,
    );
  } else {
    const prebuildDir = path.join(__dirname, 'prebuilds', `${platform}-${arch}`);

    if (!fs.existsSync(prebuildDir)) {
      details.push(`- ${prebuildDir} (not found)`);
    } else {
      const candidateNames = new Set();
      const napiVersion = process.versions?.napi;
      const moduleVersion = process.versions?.modules;
      const prioritized = [
        'zkprov.napi.node',
        napiVersion ? `zkprov.napi-v${napiVersion}.node` : undefined,
        moduleVersion ? `node-v${moduleVersion}.node` : undefined,
        'node.napi.node',
      ].filter((value) => Boolean(value));

      for (const name of prioritized) {
        candidateNames.add(name);
      }

      for (const entry of fs.readdirSync(prebuildDir)) {
        if (entry.endsWith('.node')) {
          candidateNames.add(entry);
        }
      }

      for (const candidate of candidateNames) {
        const prebuildBinding = tryLoad(path.join(prebuildDir, candidate));
        if (prebuildBinding) {
          return prebuildBinding;
        }
      }

      if (candidateNames.size === 0) {
        details.push(`- ${prebuildDir} (no native addon artifacts found)`);
      }
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
