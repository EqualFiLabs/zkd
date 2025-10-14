"""
ZKProv Python bindings (ctypes).
Public API will expose: list_backends, list_profiles, prove, verify.
"""
from __future__ import annotations

import json
import os
import sys
import ctypes

from pathlib import Path

from ctypes import (
    CDLL,
    POINTER,
    c_char_p,
    c_int,
    c_uint32,
    c_uint64,
    c_uint8,
    c_void_p,
)


__all__ = ["list_backends", "list_profiles", "prove", "verify"]
HERE = Path(__file__).resolve().parent
NAME = {"darwin": "libzkprov.dylib", "win32": "zkprov.dll"}.get(
    sys.platform, "libzkprov.so"
)


def _load_lib() -> CDLL:
    """Resolve and load the native ZKProv library."""

    bundled = HERE / NAME
    if bundled.exists():
        return CDLL(str(bundled))

    env = os.environ.get("ZKPROV_LIB")
    if env:
        return CDLL(env)

    return CDLL(NAME)


_LIB = _load_lib()


# C prototypes:
# int32_t zkp_init(void);
_LIB.zkp_init.restype = c_int
_LIB.zkp_init.argtypes = []

# int32_t zkp_list_backends(char** out_json);
_LIB.zkp_list_backends.restype = c_int
_LIB.zkp_list_backends.argtypes = [POINTER(c_char_p)]

_LIB.zkp_list_profiles.restype = c_int
_LIB.zkp_list_profiles.argtypes = [POINTER(c_char_p)]

# int32_t zkp_prove(..., uint8_t** out_proof, uint64_t* out_len, char** out_json_meta);
_LIB.zkp_prove.restype = c_int
_LIB.zkp_prove.argtypes = [
    c_char_p,
    c_char_p,
    c_char_p,
    c_uint32,
    c_char_p,
    c_char_p,
    c_char_p,
    POINTER(POINTER(c_uint8)),
    POINTER(c_uint64),
    POINTER(c_char_p),
]

# int32_t zkp_verify(..., const uint8_t* proof, uint64_t len, char** out_json_meta);
_LIB.zkp_verify.restype = c_int
_LIB.zkp_verify.argtypes = [
    c_char_p,
    c_char_p,
    c_char_p,
    c_uint32,
    c_char_p,
    c_char_p,
    c_char_p,
    POINTER(c_uint8),
    c_uint64,
    POINTER(c_char_p),
]

# void zkp_free(void*);
_LIB.zkp_free.restype = None
_LIB.zkp_free.argtypes = [c_void_p]


def _decode_json(ptr: c_char_p):
    s = ""
    try:
        if ptr:
            raw = ctypes.cast(ptr, c_char_p).value
            s = raw.decode("utf-8") if raw else ""
            _LIB.zkp_free(ptr)  # free as per API contract
        return json.loads(s or "{}")
    except Exception as e:  # pragma: no cover - defensive guard
        raise RuntimeError(f"Invalid JSON from native: {e}\n{s!r}")


def _err(code, payload):
    # payload is dict from native JSON envelope (if any)
    msg = payload.get("msg") if isinstance(payload, dict) else str(payload)
    detail = payload.get("detail") if isinstance(payload, dict) else None
    raise RuntimeError(
        f"[ZKProv err {code}] {msg}" + (f" ({detail})" if detail else "")
    )


# one-time init (idempotent)
_rc = _LIB.zkp_init()
if _rc != 0:
    raise RuntimeError(f"zkp_init failed: code={_rc}")


def list_backends() -> dict:
    out = c_char_p()
    code = _LIB.zkp_list_backends(ctypes.byref(out))
    payload = _decode_json(out)
    if code != 0:
        _err(code, payload)
    return payload


def list_profiles() -> dict:
    out = c_char_p()
    code = _LIB.zkp_list_profiles(ctypes.byref(out))
    payload = _decode_json(out)
    if code != 0:
        _err(code, payload)
    return payload


class ProveConfig(ctypes.Structure):
    # plain Python object is fine; this is informational only
    pass


def prove(
    *,
    backend_id: str,
    field: str,
    hash_id: str,
    fri_arity: int,
    profile_id: str,
    air_path: str,
    public_inputs_json: str,
):
    out_proof = POINTER(c_uint8)()
    out_len = c_uint64(0)
    out_meta = c_char_p()

    code = _LIB.zkp_prove(
        backend_id.encode(),
        field.encode(),
        hash_id.encode(),
        c_uint32(fri_arity),
        profile_id.encode(),
        air_path.encode(),
        public_inputs_json.encode(),
        ctypes.byref(out_proof),
        ctypes.byref(out_len),
        ctypes.byref(out_meta),
    )
    meta = _decode_json(out_meta)
    if code != 0:
        # if native allocated a proof buffer on error, free it
        if out_proof:
            _LIB.zkp_free(out_proof)
        _err(code, meta)

    n = int(out_len.value)
    try:
        proof = ctypes.string_at(out_proof, n)  # copy into bytes
    finally:
        if out_proof:
            _LIB.zkp_free(out_proof)
    return proof, meta


def verify(
    *,
    backend_id: str,
    field: str,
    hash_id: str,
    fri_arity: int,
    profile_id: str,
    air_path: str,
    public_inputs_json: str,
    proof: bytes,
):
    buf = (c_uint8 * len(proof)).from_buffer_copy(proof)
    out_meta = c_char_p()
    code = _LIB.zkp_verify(
        backend_id.encode(),
        field.encode(),
        hash_id.encode(),
        c_uint32(fri_arity),
        profile_id.encode(),
        air_path.encode(),
        public_inputs_json.encode(),
        ctypes.cast(buf, POINTER(c_uint8)),
        c_uint64(len(proof)),
        ctypes.byref(out_meta),
    )
    meta = _decode_json(out_meta)
    if code != 0:
        _err(code, meta)
    return bool(meta.get("verified", False)), meta
