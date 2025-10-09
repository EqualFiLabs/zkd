/// <reference types="node" />

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

export declare function listBackends(): Promise<any>;
export declare function listProfiles(): Promise<any>;
export declare function prove(cfg: ProveConfig): Promise<ProveResult>;
export declare function verify(cfg: ProveConfig, proof: Buffer): Promise<VerifyResult>;
