from __future__ import annotations

from pathlib import Path

import zkprov


def main() -> int:
    air_path = (Path(__file__).resolve().parents[1] / "air/toy.air").resolve()
    cfg = dict(
        backend_id="native@0.0",
        field="Prime254",
        hash_id="blake3",
        fri_arity=2,
        profile_id="balanced",
        air_path=str(air_path),
        public_inputs_json='{"demo":true,"n":7}',
    )

    print("backends:", ", ".join(sorted(zkprov.list_backends().keys())))
    proof, meta = zkprov.prove(**cfg)
    print("digest:", meta.get("digest"), "len:", meta.get("proof_len"))
    ok, meta2 = zkprov.verify(proof=proof, **cfg)
    print("verified:", ok, "digest2:", meta2.get("digest"))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
