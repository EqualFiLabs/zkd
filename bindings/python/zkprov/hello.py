import json
import sys

from . import list_backends, prove, verify


def main(argv=None):
    backs = list_backends()
    print("backends:", list(backs.keys()))
    cfg = dict(
        backend_id="native@0.0",
        field="Prime254",
        hash_id="blake3",
        fri_arity=2,
        profile_id="balanced",
        air_path="examples/air/toy.air",
        public_inputs_json='{"demo":true,"n":7}',
    )
    proof, meta = prove(**cfg)
    print("D=", meta.get("digest"), "len=", meta.get("proof_len"))
    ok, meta2 = verify(proof=proof, **cfg)
    print("verified:", ok, "D2=", meta2.get("digest"))
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
