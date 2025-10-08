// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "../contracts/VerifierStub.sol";

interface Vm {
    function readFile(string calldata path) external view returns (string memory);
    function readFileBinary(string calldata path) external view returns (bytes memory);
    function parseJson(string calldata json, string calldata key) external pure returns (bytes memory);
    function parseBytes(string calldata str) external pure returns (bytes memory);
}

contract VerifierStubTest {
    // hevm cheatcodes address
    Vm private constant vm = Vm(address(uint160(uint256(keccak256("hevm cheat code")))));

    VerifierStub private verifier = new VerifierStub();

    function testComputeDigestMatchesFixture() public {
        string memory meta = vm.readFile("testdata/meta.json");
        uint64 backendIdHash = abi.decode(vm.parseJson(meta, ".backendId"), (uint64));
        uint64 profileIdHash = abi.decode(vm.parseJson(meta, ".profileId"), (uint64));
        uint64 pubioHash = abi.decode(vm.parseJson(meta, ".pubioHash"), (uint64));
        uint64 bodyLen = abi.decode(vm.parseJson(meta, ".bodyLen"), (uint64));

        bytes memory body = vm.readFileBinary("testdata/body.bin");
        require(body.length == bodyLen, "body length mismatch");

        string memory digestHex = vm.readFile("testdata/digest.hex");
        bytes memory digestHexBytes = bytes(digestHex);
        if (digestHexBytes.length > 0 && digestHexBytes[digestHexBytes.length - 1] == 0x0a) {
            assembly {
                mstore(digestHexBytes, sub(mload(digestHexBytes), 1))
            }
        }

        bytes memory digestBytes = vm.parseBytes(string.concat("0x", string(digestHexBytes)));
        require(digestBytes.length == 32, "digest length invalid");

        bytes32 expectedDigest;
        assembly {
            expectedDigest := mload(add(digestBytes, 0x20))
        }

        bytes32 computed = verifier.computeDigest(
            backendIdHash,
            profileIdHash,
            pubioHash,
            bodyLen,
            body
        );

        require(computed == expectedDigest, "digest mismatch");
    }
}
