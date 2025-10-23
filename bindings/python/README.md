# ZKProv Python bindings

This package exposes a thin ctypes-based bridge to the ZKProv proving library.

```python
from zkprov import list_backends

for backend in list_backends():
    print(f"Available backend: {backend}")
```

The public API is still stabilizing; expect additional helpers for proof
creation and verification in upcoming releases.
