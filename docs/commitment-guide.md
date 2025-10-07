# Commitment Guide

This appendix summarizes how developers declare committed inputs, generate blindings, and validate digest parity when integrating AIR programs with external systems.

## Declaring Committed Inputs

Add a public input entry in your AIR manifest with an explicit binding:

```toml
[[public.input]]
name = "loan_amount"
type = "Point"
binding = "Pedersen(jubjub)"
```

The prover expands the declaration into `(Cx,Cy)` field elements and injects on-curve checks automatically. Use `PoseidonCommit` or `KeccakCommit` when scalar outputs are preferred.

## Generating Blindings

Each commitment combines a witness value `v` with a fresh blinding scalar `r`:

```text
C = v·H + r·G
```

Sample `r` uniformly for every commitment to avoid `BlindingReuse`. Store `r` only in prover-side secrets; it never appears in public inputs.

## Encoding & Transcript Binding

All commitment digests are encoded deterministically before absorption into the transcript:

* Pedersen points serialize as `(Cx,Cy)` little-endian field elements.
* Poseidon and Keccak commitments emit canonical scalars aligned with `interfaces.md`.
* Public input order is fixed; altering the sequence invalidates the transcript seed.

## On-Chain Parity Checks

To confirm compatibility with Solidity or other on-chain verifiers:

1. Recompute the Pedersen or Poseidon commitments in the target environment using identical domain tags.
2. For `KeccakCommit`, hash the canonical encoding with `keccak256` and compare directly to the prover output.
3. Validate that the transcript digest `D` matches between prover logs and on-chain recomputation.

These steps ensure EqualVeil/EqualLend integrations share a single deterministic commitment pipeline across devices and chains.
