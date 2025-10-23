import importlib
import sys
from pathlib import Path

import ctypes


class DummyCallable:
    def __init__(self, result=0):
        self._result = result
        self.restype = None
        self.argtypes = None

    def __call__(self, *args, **kwargs):  # pragma: no cover - exercised via ctypes usage
        return self._result


class DummyLib:
    def __init__(self):
        self.zkp_init = DummyCallable(0)
        self.zkp_list_backends = DummyCallable(0)
        self.zkp_list_profiles = DummyCallable(0)
        self.zkp_prove = DummyCallable(0)
        self.zkp_verify = DummyCallable(0)
        self.zkp_free = DummyCallable(None)


def _expected_library_name() -> str:
    if sys.platform == "darwin":
        return "libzkprov.dylib"
    if sys.platform == "win32":
        return "zkprov.dll"
    return "libzkprov.so"


def test_loads_packaged_library_first(monkeypatch):
    monkeypatch.delenv("ZKPROV_LIB", raising=False)
    module_name = "zkprov"
    sys.modules.pop(module_name, None)

    package_root = Path(__file__).resolve().parents[1]
    sys_path_entry = str(package_root)
    sys.path.insert(0, sys_path_entry)

    pkg_dir = package_root / "zkprov"
    expected = pkg_dir / _expected_library_name()
    expected.write_bytes(b"dummy")

    loaded_paths: list[str] = []

    def fake_cdll(path):
        path_str = str(path)
        loaded_paths.append(path_str)
        if Path(path_str) == expected:
            return DummyLib()
        raise OSError("not found")

    monkeypatch.setattr(ctypes, "CDLL", fake_cdll)

    try:
        module = importlib.import_module(module_name)
        assert isinstance(module._LIB, DummyLib)
        assert loaded_paths[0] == str(expected)
    finally:
        sys.modules.pop(module_name, None)
        if sys.path and sys.path[0] == sys_path_entry:
            sys.path.pop(0)
        if expected.exists():
            expected.unlink()
