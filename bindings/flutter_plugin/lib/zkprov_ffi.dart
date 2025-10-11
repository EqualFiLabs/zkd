import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart' as ffi_pkg;

typedef _ZkpInitNative = ffi.Int32 Function();
typedef _ZkpInitDart = int Function();
typedef _ZkpListNative = ffi.Int32 Function(ffi.Pointer<ffi.Pointer<ffi_pkg.Utf8>>);
typedef _ZkpListDart = int Function(ffi.Pointer<ffi.Pointer<ffi_pkg.Utf8>>);
typedef _ZkpProveNative = ffi.Int32 Function(
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Uint32,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi.Pointer<ffi.Uint8>>,
  ffi.Pointer<ffi.Uint64>,
  ffi.Pointer<ffi.Pointer<ffi_pkg.Utf8>>,
);
typedef _ZkpProveDart = int Function(
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  int,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi.Pointer<ffi.Uint8>>,
  ffi.Pointer<ffi.Uint64>,
  ffi.Pointer<ffi.Pointer<ffi_pkg.Utf8>>,
);
typedef _ZkpVerifyNative = ffi.Int32 Function(
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Uint32,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi.Uint8>,
  ffi.Uint64,
  ffi.Pointer<ffi.Pointer<ffi_pkg.Utf8>>,
);
typedef _ZkpVerifyDart = int Function(
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  int,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi_pkg.Utf8>,
  ffi.Pointer<ffi.Uint8>,
  int,
  ffi.Pointer<ffi.Pointer<ffi_pkg.Utf8>>,
);
typedef _ZkpFreeNative = ffi.Void Function(ffi.Pointer<ffi.Void>);
typedef _ZkpFreeDart = void Function(ffi.Pointer<ffi.Void>);

const int _zkpOk = 0;
const int _zkpErrVerifyFail = 5;
const String _envLibraryPath = 'ZKPROV_LIBRARY_PATH';

final ffi.DynamicLibrary _lib = _openLibrary();

final _ZkpInitDart _zkpInit =
    _lib.lookupFunction<_ZkpInitNative, _ZkpInitDart>('zkp_init');
final _ZkpListDart _zkpListBackends =
    _lib.lookupFunction<_ZkpListNative, _ZkpListDart>(
  'zkp_list_backends',
  isLeaf: true,
);
final _ZkpListDart _zkpListProfiles =
    _lib.lookupFunction<_ZkpListNative, _ZkpListDart>(
  'zkp_list_profiles',
  isLeaf: true,
);
final _ZkpProveDart _zkpProve =
    _lib.lookupFunction<_ZkpProveNative, _ZkpProveDart>(
  'zkp_prove',
  isLeaf: true,
);
final _ZkpVerifyDart _zkpVerify =
    _lib.lookupFunction<_ZkpVerifyNative, _ZkpVerifyDart>(
  'zkp_verify',
  isLeaf: true,
);
final ffi.Pointer<ffi.NativeFunction<_ZkpFreeNative>> _zkpFreePtr = _lib
    .lookup<ffi.NativeFunction<_ZkpFreeNative>>('zkp_free');
final _ZkpFreeDart _zkpFree =
    _zkpFreePtr.asFunction<_ZkpFreeDart>(isLeaf: true);
final ffi.NativeFinalizer _freeFinalizer = ffi.NativeFinalizer(_zkpFreePtr);
final Expando<_NativeAllocation> _allocationTokens =
    Expando<_NativeAllocation>('_zkprov_allocations');

class _NativeAllocation implements ffi.Finalizable {
  _NativeAllocation();
}

bool _initialized = false;

class Config {
  const Config({
    required this.backendId,
    required this.field,
    required this.hashId,
    required this.friArity,
    required this.profileId,
    required this.airPath,
    required this.publicInputs,
  });

  final String backendId;
  final String field;
  final String hashId;
  final int friArity;
  final String profileId;
  final String airPath;
  final Map<String, dynamic> publicInputs;

  String get publicInputsJson => jsonEncode(publicInputs);
}

class ZkProvException implements Exception {
  ZkProvException(this.operation, this.code, [this.message]);

  final String operation;
  final int code;
  final String? message;

  @override
  String toString() {
    final base = 'ZkProvException($operation, code=$code)';
    if (message == null || message!.isEmpty) {
      return base;
    }
    return '$base: $message';
  }
}

Future<Map<String, dynamic>> listBackends() => Future.sync(() {
      _ensureInitialized();
      final outJson = ffi_pkg.calloc<ffi.Pointer<ffi_pkg.Utf8>>();
      try {
        final code = _zkpListBackends(outJson);
        _checkResult('zkp_list_backends', code);
        return _decodeJsonPointer(outJson.value);
      } finally {
        ffi_pkg.calloc.free(outJson);
      }
    });

Future<Map<String, dynamic>> listProfiles() => Future.sync(() {
      _ensureInitialized();
      final outJson = ffi_pkg.calloc<ffi.Pointer<ffi_pkg.Utf8>>();
      try {
        final code = _zkpListProfiles(outJson);
        _checkResult('zkp_list_profiles', code);
        return _decodeJsonPointer(outJson.value);
      } finally {
        ffi_pkg.calloc.free(outJson);
      }
    });

Future<(Uint8List proof, Map<String, dynamic> meta)> prove(Config cfg) =>
    Future.sync(() {
      _ensureInitialized();
      final backend = _str(cfg.backendId);
      final field = _str(cfg.field);
      final hash = _str(cfg.hashId);
      final profile = _str(cfg.profileId);
      final air = _str(cfg.airPath);
      final inputs = _str(cfg.publicInputsJson);
      final outProof = ffi_pkg.calloc<ffi.Pointer<ffi.Uint8>>();
      final outProofLen = ffi_pkg.calloc<ffi.Uint64>();
      final outMeta = ffi_pkg.calloc<ffi.Pointer<ffi_pkg.Utf8>>();
      try {
        final code = _zkpProve(
          backend,
          field,
          hash,
          cfg.friArity,
          profile,
          air,
          inputs,
          outProof,
          outProofLen,
          outMeta,
        );
        _checkResult('zkp_prove', code);
        final proofPtr = outProof.value;
        final proofLen = outProofLen.value;
        final meta = _decodeJsonPointer(outMeta.value);
        if (_isNull(proofPtr) || proofLen == 0) {
          if (!_isNull(proofPtr)) {
            _freeNative(proofPtr.cast());
          }
          return (Uint8List(0), meta);
        }
        final proof = proofPtr.asTypedList(proofLen);
        _attachFinalizer(proof, proofPtr.cast());
        return (proof, meta);
      } finally {
        _freeNativeString(backend);
        _freeNativeString(field);
        _freeNativeString(hash);
        _freeNativeString(profile);
        _freeNativeString(air);
        _freeNativeString(inputs);
        ffi_pkg.calloc.free(outProof);
        ffi_pkg.calloc.free(outProofLen);
        ffi_pkg.calloc.free(outMeta);
      }
    });

Future<(bool verified, Map<String, dynamic> meta)> verify(
  Config cfg,
  Uint8List proof,
) =>
    Future.sync(() {
      _ensureInitialized();
      final backend = _str(cfg.backendId);
      final field = _str(cfg.field);
      final hash = _str(cfg.hashId);
      final profile = _str(cfg.profileId);
      final air = _str(cfg.airPath);
      final inputs = _str(cfg.publicInputsJson);
      final proofPtr = proof.isEmpty
          ? ffi.Pointer<ffi.Uint8>.fromAddress(0)
          : ffi_pkg.malloc<ffi.Uint8>(proof.length);
      if (proof.isNotEmpty) {
        final proofBuf = proofPtr.asTypedList(proof.length);
        proofBuf.setAll(0, proof);
      }
      final outMeta = ffi_pkg.calloc<ffi.Pointer<ffi_pkg.Utf8>>();
      try {
        final code = _zkpVerify(
          backend,
          field,
          hash,
          cfg.friArity,
          profile,
          air,
          inputs,
          proofPtr,
          proof.length,
          outMeta,
        );
        if (code == _zkpErrVerifyFail) {
          return (
            false,
            <String, dynamic>{'error_code': code, 'message': 'verification failed'},
          );
        }
        _checkResult('zkp_verify', code);
        final meta = _decodeJsonPointer(outMeta.value);
        return (true, meta);
      } finally {
        _freeNativeString(backend);
        _freeNativeString(field);
        _freeNativeString(hash);
        _freeNativeString(profile);
        _freeNativeString(air);
        _freeNativeString(inputs);
        if (proof.isNotEmpty) {
          ffi_pkg.malloc.free(proofPtr);
        }
        ffi_pkg.calloc.free(outMeta);
      }
    });

void _ensureInitialized() {
  if (_initialized) {
    return;
  }
  final code = _zkpInit();
  _checkResult('zkp_init', code);
  _initialized = true;
}

void _checkResult(String operation, int code) {
  if (code == _zkpOk) {
    return;
  }
  throw ZkProvException(operation, code);
}

ffi.DynamicLibrary _openLibrary() {
  final override = Platform.environment[_envLibraryPath];
  if (override != null && override.isNotEmpty) {
    return ffi.DynamicLibrary.open(override);
  }
  if (Platform.isAndroid) {
    return ffi.DynamicLibrary.open('libzkprov.so');
  }
  if (Platform.isIOS) {
    return ffi.DynamicLibrary.process();
  }
  if (Platform.isMacOS) {
    try {
      return ffi.DynamicLibrary.process();
    } on ArgumentError {
      // Fallback to a directly bundled dylib for local development.
    }
    return ffi.DynamicLibrary.open('libzkprov.dylib');
  }
  if (Platform.isLinux) {
    return ffi.DynamicLibrary.open('libzkprov.so');
  }
  if (Platform.isWindows) {
    return ffi.DynamicLibrary.open('zkprov.dll');
  }
  throw UnsupportedError(
    'Unsupported platform for zkprov: ${Platform.operatingSystem}. '
    'Set $_envLibraryPath to the native library path.',
  );
}

ffi.Pointer<ffi_pkg.Utf8> _str(String value) => value.toNativeUtf8();

String _fromUtf8(ffi.Pointer<ffi_pkg.Utf8> ptr) {
  if (_isNull(ptr)) {
    return '';
  }
  return ptr.toDartString();
}

bool _isNull<T extends ffi.NativeType>(ffi.Pointer<T> ptr) => ptr.address == 0;

void _freeNativeString(ffi.Pointer<ffi_pkg.Utf8> ptr) {
  if (_isNull(ptr)) {
    return;
  }
  ffi_pkg.malloc.free(ptr);
}

void _freeNative(ffi.Pointer<ffi.Void> ptr) {
  if (_isNull(ptr)) {
    return;
  }
  _zkpFree(ptr);
}

void _attachFinalizer(Object owner, ffi.Pointer<ffi.Void> ptr) {
  if (_isNull(ptr)) {
    return;
  }
  final token = _NativeAllocation();
  _freeFinalizer.attach(token, ptr);
  _allocationTokens[owner] = token;
}

Map<String, dynamic> _decodeJsonPointer(ffi.Pointer<ffi_pkg.Utf8> ptr) {
  if (_isNull(ptr)) {
    return <String, dynamic>{};
  }
  try {
    final json = _fromUtf8(ptr);
    final dynamic decoded = jsonDecode(json);
    if (decoded is Map<String, dynamic>) {
      return Map<String, dynamic>.from(decoded);
    }
    if (decoded is Map) {
      return Map<String, dynamic>.from(decoded.cast<String, dynamic>());
    }
    if (decoded is List) {
      return <String, dynamic>{'items': List<dynamic>.from(decoded)};
    }
    return <String, dynamic>{'value': decoded};
  } finally {
    _freeNative(ptr.cast());
  }
}
