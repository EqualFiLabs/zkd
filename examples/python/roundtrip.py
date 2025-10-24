from __future__ import annotations

from pathlib import Path

import zkprov


def main() -> int:
    air_path = (Path(__file__).resolve().parents[1] / "air/toy.air").resolve()
    backends = zkprov.list_backends()
    if isinstance(backends, dict):
        backend_names = sorted(backends.keys())
    else:
        backend_names = sorted(
            item.get("id")
            for item in backends
            if isinstance(item, dict) and item.get("id")
        )
    cfg = dict(
        backend_id="native@0.0",
        field="Prime254",
        hash_id="blake3",
        fri_arity=2,
        profile_id="balanced",
        air_path=str(air_path),
        public_inputs_json='{"demo":true,"n":7}',
    )

    print("backends:", ", ".join(backend_names))
    proof, meta = zkprov.prove(**cfg)
    print("digest:", meta.get("digest"), "len:", meta.get("proof_len"))
    ok, meta2 = zkprov.verify(proof=proof, **cfg)
    print("verified:", ok, "digest2:", meta2.get("digest"))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
