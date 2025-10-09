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

function resolveCandidates(): string[] {
  const candidates: string[] = [];

  const releasePath = path.join(__dirname, 'build', 'Release', 'zkprov.node');
  candidates.push(releasePath);

  const debugPath = path.join(__dirname, 'build', 'Debug', 'zkprov.node');
  candidates.push(debugPath);

  const rootPath = path.join(__dirname, 'zkprov.node');
  candidates.push(rootPath);

  const platform = process.platform;
  const arch = process.arch;
  const prebuildsDir = path.join(__dirname, 'prebuilds');
  const dirCandidates = new Set<string>();
  dirCandidates.add(`${platform}-${arch}`);

  if (fs.existsSync(prebuildsDir)) {
    try {
      for (const entry of fs.readdirSync(prebuildsDir)) {
        if (entry.startsWith(`${platform}-${arch}`)) {
          dirCandidates.add(entry);
        }
      }
    } catch (err) {
      // Ignore directory read errors and continue with default candidates.
    }
  }

  const abi = process.versions && process.versions.modules ? process.versions.modules : undefined;
  const napi = process.versions && process.versions.napi ? process.versions.napi : undefined;
  const filenames = new Set<string>(['node.napi.node', 'node-napi.node']);

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

function loadNativeBinding(): NativeBinding {
  const attempted: string[] = [];
  const details: string[] = [];

  for (const candidate of resolveCandidates()) {
    attempted.push(candidate);
    if (!fs.existsSync(candidate)) {
      details.push(`- ${candidate} (not found)`);
      continue;
    }

    try {
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const binding = require(candidate) as NativeBinding;
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
