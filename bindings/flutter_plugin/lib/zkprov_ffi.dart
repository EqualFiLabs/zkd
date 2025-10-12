import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart' as ffi_pkg;

typedef _ZkpInitNative = ffi.Int32 Function();
typedef _ZkpInitDart = int Function();
typedef _ZkpListNative =
    ffi.Int32 Function(ffi.Pointer<ffi.Pointer<ffi_pkg.Utf8>>);
typedef _ZkpListDart = int Function(ffi.Pointer<ffi.Pointer<ffi_pkg.Utf8>>);
typedef _ZkpProveNative =
    ffi.Int32 Function(
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
typedef _ZkpProveDart =
    int Function(
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
typedef _ZkpVerifyNative =
    ffi.Int32 Function(
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
typedef _ZkpVerifyDart =
    int Function(
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
const int _zkpErrInvalidArg = 1;
const int _zkpErrBackend = 2;
const int _zkpErrProfile = 3;
const int _zkpErrProofCorrupt = 4;
const int _zkpErrVerifyFail = 5;
const int _zkpErrInternal = 6;
const String _envLibraryPath = 'ZKPROV_LIBRARY_PATH';

final ffi.DynamicLibrary _lib = _openLibrary();

final _ZkpInitDart _zkpInit = _lib.lookupFunction<_ZkpInitNative, _ZkpInitDart>(
  'zkp_init',
);
final _ZkpListDart _zkpListBackends = _lib
    .lookupFunction<_ZkpListNative, _ZkpListDart>(
      'zkp_list_backends',
      isLeaf: true,
    );
final _ZkpListDart _zkpListProfiles = _lib
    .lookupFunction<_ZkpListNative, _ZkpListDart>(
      'zkp_list_profiles',
      isLeaf: true,
    );
final _ZkpProveDart _zkpProve = _lib
    .lookupFunction<_ZkpProveNative, _ZkpProveDart>('zkp_prove', isLeaf: true);
final _ZkpVerifyDart _zkpVerify = _lib
    .lookupFunction<_ZkpVerifyNative, _ZkpVerifyDart>(
      'zkp_verify',
      isLeaf: true,
    );
final ffi.Pointer<ffi.NativeFunction<_ZkpFreeNative>> _zkpFreePtr = _lib
    .lookup<ffi.NativeFunction<_ZkpFreeNative>>('zkp_free');
final _ZkpFreeDart _zkpFree = _zkpFreePtr.asFunction<_ZkpFreeDart>(
  isLeaf: true,
);
final ffi.NativeFinalizer _freeFinalizer = ffi.NativeFinalizer(_zkpFreePtr);
final Expando<_NativeAllocation> _allocationTokens =
    Expando<_NativeAllocation>(
  '_zkprov_allocations',
);

class _NativeAllocation implements ffi.Finalizable {
  _NativeAllocation();
}

bool _initialized = false;

class ZkProvConfig {
  final String backendId, field, hashId, profileId, airPath, publicInputsJson;
  final int friArity;
  const ZkProvConfig({
    required this.backendId,
    required this.field,
    required this.hashId,
    required this.friArity,
    required this.profileId,
    required this.airPath,
    required this.publicInputsJson,
  });
}

class ZkProvException implements Exception {
  ZkProvException({
    required this.operation,
    required this.code,
    required this.msg,
    this.detail,
  });

  final String operation;
  final int code;
  final String msg;
  final String? detail;

  Map<String, dynamic> toJson() => <String, dynamic>{
        'code': code,
        'msg': msg,
        if (detail != null && detail!.isNotEmpty) 'detail': detail,
      };

  @override
  String toString() {
    final buffer = StringBuffer('ZkProvException(')
      ..write(operation)
      ..write(', code=')
      ..write(code)
      ..write(')')
      ..write(': ')
      ..write(msg);
    if (detail != null && detail!.isNotEmpty) {
      buffer
        ..write(' [detail: ')
        ..write(detail)
        ..write(']');
    }
    return buffer.toString();
  }
}

Future<Map<String, dynamic>> listBackends() => Future.sync(() {
  _ensureInitialized();
  final outJson = ffi_pkg.calloc<ffi.Pointer<ffi_pkg.Utf8>>();
  try {
    final code = _zkpListBackends(outJson);
    if (code != _zkpOk) {
      final ptr = outJson.value;
      if (!_isNull(ptr)) {
        _freeNative(ptr.cast());
        outJson.value = ffi.Pointer<ffi_pkg.Utf8>.fromAddress(0);
      }
    }
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
    if (code != _zkpOk) {
      final ptr = outJson.value;
      if (!_isNull(ptr)) {
        _freeNative(ptr.cast());
        outJson.value = ffi.Pointer<ffi_pkg.Utf8>.fromAddress(0);
      }
    }
    _checkResult('zkp_list_profiles', code);
    return _decodeJsonPointer(outJson.value);
  } finally {
    ffi_pkg.calloc.free(outJson);
  }
});

Future<ZkProvProveResult> prove(ZkProvConfig cfg) => Future.sync(() {
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
    if (code != _zkpOk) {
      final proofPtr = outProof.value;
      if (!_isNull(proofPtr)) {
        _freeNative(proofPtr.cast());
        outProof.value = ffi.Pointer<ffi.Uint8>.fromAddress(0);
      }
      final metaPtr = outMeta.value;
      if (!_isNull(metaPtr)) {
        _freeNative(metaPtr.cast());
        outMeta.value = ffi.Pointer<ffi_pkg.Utf8>.fromAddress(0);
      }
    }
    _checkResult('zkp_prove', code);
    final proofPtr = outProof.value;
    final proofLen = outProofLen.value;
    final meta = _decodeJsonPointer(outMeta.value);
    final digest = _metaDigest(meta);
    final proofLength = _metaProofLength(meta) ?? proofLen;
    if (_isNull(proofPtr) || proofLen == 0) {
      if (!_isNull(proofPtr)) {
        _freeNative(proofPtr.cast());
      }
      return ZkProvProveResult(
        proof: Uint8List(0),
        digest: digest,
        proofLength: proofLength,
        meta: meta,
      );
    }
    final proofView = proofPtr.asTypedList(proofLen);
    final proof = Uint8List.fromList(proofView);
    _freeNative(proofPtr.cast());
    return ZkProvProveResult(
      proof: proof,
      digest: digest,
      proofLength: proofLength,
      meta: meta,
    );
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

Future<ZkProvVerifyResult> verify(ZkProvConfig cfg, Uint8List proof) =>
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
        if (code != _zkpOk && code != _zkpErrVerifyFail) {
          final metaPtr = outMeta.value;
          if (!_isNull(metaPtr)) {
            _freeNative(metaPtr.cast());
            outMeta.value = ffi.Pointer<ffi_pkg.Utf8>.fromAddress(0);
          }
        }
        if (code == _zkpErrVerifyFail) {
          final metaPtr = outMeta.value;
          Map<String, dynamic> meta;
          if (_isNull(metaPtr)) {
            meta = <String, dynamic>{};
          } else {
            meta = _decodeJsonPointer(metaPtr);
          }
          meta.putIfAbsent('error_code', () => code);
          meta.putIfAbsent('message', () => _errorMessageForCode(code));
          return ZkProvVerifyResult(
            verified: false,
            digest: '',
            meta: meta,
          );
        }
        _checkResult('zkp_verify', code);
        final meta = _decodeJsonPointer(outMeta.value);
        return ZkProvVerifyResult(
          verified: true,
          digest: _metaDigest(meta),
          meta: meta,
        );
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

void _checkResult(String operation, int code, {String? detail}) {
  if (code == _zkpOk) {
    return;
  }
  final effectiveDetail = detail ?? 'operation=$operation';
  throw ZkProvException(
    operation: operation,
    code: code,
    msg: _errorMessageForCode(code),
    detail: effectiveDetail,
  );
}

String _errorMessageForCode(int code) {
  switch (code) {
    case _zkpErrInvalidArg:
      return 'Invalid argument';
    case _zkpErrBackend:
      return 'Backend error';
    case _zkpErrProfile:
      return 'Profile error';
    case _zkpErrProofCorrupt:
      return 'Proof corrupt';
    case _zkpErrVerifyFail:
      return 'Verification failed';
    case _zkpErrInternal:
    default:
      return 'Internal error';
  }
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

class ZkProvProveResult {
  const ZkProvProveResult({
    required this.proof,
    required this.digest,
    required this.proofLength,
    required this.meta,
  });

  final Uint8List proof;
  final String digest;
  final int proofLength;
  final Map<String, dynamic> meta;
}

class ZkProvVerifyResult {
  const ZkProvVerifyResult({
    required this.verified,
    required this.digest,
    required this.meta,
  });

  final bool verified;
  final String digest;
  final Map<String, dynamic> meta;
}

String _metaDigest(Map<String, dynamic> meta) {
  final dynamic value = meta['digest'];
  if (value is String) {
    return value;
  }
  if (value == null) {
    return '';
  }
  return value.toString();
}

int? _metaProofLength(Map<String, dynamic> meta) {
  final dynamic value = meta['proof_len'];
  if (value is int) {
    return value;
  }
  if (value is num) {
    return value.toInt();
  }
  if (value is String) {
    return int.tryParse(value);
  }
  return null;
}
