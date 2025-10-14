"""
ZKProv Python bindings (ctypes).
Public API will expose: list_backends, list_profiles, prove, verify.
"""
from __future__ import annotations

import os
import sys
import ctypes


__all__ = ["list_backends", "list_profiles", "prove", "verify"]


def _load_lib() -> ctypes.CDLL:
    """Resolve and load the native ZKProv library."""

    candidates: list[str] = []
    env_path = os.getenv("ZKPROV_LIB")
    if env_path:
        candidates.append(env_path)

    if sys.platform == "darwin":
        names = ["libzkprov.dylib"]
    elif sys.platform == "win32":
        names = ["zkprov.dll"]
    else:
        names = ["libzkprov.so"]

    cwd = os.getcwd()
    for name in names:
        candidates.append(os.path.join(cwd, name))

    if sys.platform == "darwin" or sys.platform.startswith("linux"):
        candidates.extend(names)

    tried: list[str] = []
    last_err: OSError | None = None
    seen: set[str] = set()

    for path in candidates:
        if not path or path in seen:
            continue
        seen.add(path)
        tried.append(path)
        try:
            return ctypes.CDLL(path)
        except OSError as exc:  # pragma: no cover - platform dependent
            last_err = exc

    raise OSError(
        f"Failed to load ZKProv native lib; tried {tried}: {last_err}"
    )


_LIB = _load_lib()
