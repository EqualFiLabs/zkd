// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract VerifierStub {
    struct EvmDigestInput {
        uint64 backendIdHash;
        uint64 profileIdHash;
        uint64 pubioHash;
        uint64 bodyLen;
        bytes body;
    }
    function computeDigest(
        uint64 backendIdHash,
        uint64 profileIdHash,
        uint64 pubioHash,
        uint64 bodyLen,
        bytes calldata body
    ) public pure returns (bytes32 D) {
        EvmDigestInput memory payload = EvmDigestInput({
            backendIdHash: backendIdHash,
            profileIdHash: profileIdHash,
            pubioHash: pubioHash,
            bodyLen: bodyLen,
            body: body
        });

        D = keccak256(abi.encode(payload));
    }

    function verifyDigest(
        uint64 backendIdHash,
        uint64 profileIdHash,
        uint64 pubioHash,
        uint64 bodyLen,
        bytes calldata body,
        bytes32 expectedD
    ) external pure returns (bool) {
        return computeDigest(
            backendIdHash,
            profileIdHash,
            pubioHash,
            bodyLen,
            body
        ) == expectedD;
    }
}
