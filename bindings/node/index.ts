import * as fs from 'fs';
import * as path from 'path';

export type ProveConfig = {
  backendId: string;
  field: string;
  hashId: string;
  friArity: number;
  profileId: string;
  airPath: string;
  publicInputsJson: string;
};

export type ProveResult = {
  proof: Buffer;
  meta: {
    digest: string;
    proof_len: number;
    [k: string]: any;
  };
};

export type VerifyResult = {
  verified: boolean;
  meta: {
    digest: string;
    [k: string]: any;
  };
};

type NativeBinding = {
  listBackends: () => Promise<any>;
  listProfiles: () => Promise<any>;
  prove: (cfg: ProveConfig) => Promise<ProveResult>;
  verify: (cfg: ProveConfig, proof: Buffer) => Promise<VerifyResult>;
};

function loadNativeBinding(): NativeBinding {
  const details: string[] = [];

  const tryLoad = (candidate: string): NativeBinding | undefined => {
    if (!fs.existsSync(candidate)) {
      details.push(`- ${candidate} (not found)`);
      return undefined;
    }

    try {
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      return require(candidate) as NativeBinding;
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

  const platformMap: Partial<Record<NodeJS.Platform, string>> = {
    win32: 'win32',
    darwin: 'darwin',
    linux: 'linux',
  };
  const archMap: Partial<Record<NodeJS.Architecture, string>> = {
    x64: 'x64',
    arm64: 'arm64',
  };

  const platform = platformMap[process.platform];
  const arch = archMap[process.arch];

  if (!platform || !arch) {
    const unsupportedParts: string[] = [];
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
    const prebuildDir = path.join(
      __dirname,
      'prebuilds',
      `${platform}-${arch}`,
    );

    if (!fs.existsSync(prebuildDir)) {
      details.push(`- ${prebuildDir} (not found)`);
    } else {
      const candidateNames = new Set<string>();

      const napiVersion = process.versions?.napi;
      const moduleVersion = process.versions?.modules;

      const prioritized = [
        'zkprov.napi.node',
        napiVersion ? `zkprov.napi-v${napiVersion}.node` : undefined,
        moduleVersion ? `node-v${moduleVersion}.node` : undefined,
        'node.napi.node',
      ].filter((value): value is string => Boolean(value));

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

let nativeBinding: NativeBinding | undefined;
let nativeLoadError: Error | undefined;

try {
  nativeBinding = loadNativeBinding();
} catch (err) {
  nativeLoadError = err instanceof Error ? err : new Error(String(err));
}

function getBinding(): NativeBinding {
  if (nativeBinding) {
    return nativeBinding;
  }

  throw nativeLoadError ?? new Error('Failed to load the zkprov native module.');
}

export async function listBackends(): Promise<any> {
  return getBinding().listBackends();
}

export async function listProfiles(): Promise<any> {
  return getBinding().listProfiles();
}

export async function prove(cfg: ProveConfig): Promise<ProveResult> {
  return getBinding().prove(cfg);
}

export async function verify(cfg: ProveConfig, proof: Buffer): Promise<VerifyResult> {
  return getBinding().verify(cfg, proof);
}
