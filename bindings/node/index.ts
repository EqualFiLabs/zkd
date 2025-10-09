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

  const platform = process.platform;
  const arch = process.arch;
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
