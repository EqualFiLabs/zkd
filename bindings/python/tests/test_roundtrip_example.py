from __future__ import annotations

import runpy
import sys
import types
from pathlib import Path

import pytest


def test_roundtrip_example_runs(capsys):
    script = Path(__file__).resolve().parents[3] / "examples/python/roundtrip.py"
    stub = types.ModuleType("zkprov")

    def list_backends():
        return {"native@0.0": {"field": ["Prime254"]}}

    def prove(**cfg):
        assert cfg["backend_id"] == "native@0.0"
        return b"fake-proof", {"digest": "0xabc", "proof_len": 123}

    def verify(*, proof, **cfg):
        assert proof == b"fake-proof"
        return True, {"digest": "0xabc"}

    stub.list_backends = list_backends
    stub.prove = prove
    stub.verify = verify

    original = sys.modules.get("zkprov")
    sys.modules["zkprov"] = stub

    try:
        with pytest.raises(SystemExit) as exc:
            runpy.run_path(str(script), run_name="__main__")
        assert exc.value.code == 0
    finally:
        if original is None:
            sys.modules.pop("zkprov", None)
        else:
            sys.modules["zkprov"] = original

    out = capsys.readouterr().out
    assert "backends: native@0.0" in out
    assert "verified: True" in out
    assert "digest: 0xabc" in out
