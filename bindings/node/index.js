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
    const prebuildPath = path.join(
      __dirname,
      'prebuilds',
      `${platform}-${arch}`,
      'zkprov.napi.node',
    );

    const prebuildBinding = tryLoad(prebuildPath);
    if (prebuildBinding) {
      return prebuildBinding;
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
