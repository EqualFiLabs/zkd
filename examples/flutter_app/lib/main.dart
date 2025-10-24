import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show rootBundle;
import 'package:zkprov_flutter/zkprov_ffi.dart';

const cfg = ZkProvConfig(
  backendId: 'native@0.0',
  field: 'Prime254',
  hashId: 'blake3',
  friArity: 2,
  profileId: 'balanced',
  airPath: 'assets/toy.air',
  publicInputsJson: '{"demo":true,"n":7}',
);

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      home: DemoHomePage(),
    );
  }
}

class DemoHomePage extends StatefulWidget {
  const DemoHomePage({super.key});

  @override
  State<DemoHomePage> createState() => _DemoHomePageState();
}

class _DemoHomePageState extends State<DemoHomePage> {
  int? _backendCount;
  String? _firstBackendId;
  String _digest = '';
  bool? _verified;
  Uint8List? _proof;
  String? _statusMessage;
  bool _initInProgress = false;
  bool _proveInProgress = false;
  bool _verifyInProgress = false;
  String? _airPathOverride;

  void _updateState(VoidCallback fn) {
    if (!mounted) {
      return;
    }
    setState(fn);
  }

  Future<void> _ensureAirAsset() async {
    if (_airPathOverride != null) {
      return;
    }
    final data = await rootBundle.load(cfg.airPath);
    final tempDir = await Directory.systemTemp.createTemp('zkprov_demo');
    final file = await File('${tempDir.path}/toy.air').writeAsBytes(
      data.buffer.asUint8List(),
      flush: true,
    );
    _updateState(() {
      _airPathOverride = file.path;
    });
  }

  ZkProvConfig _configWithAir() {
    final airPath = _airPathOverride;
    if (airPath == null) {
      return cfg;
    }
    return ZkProvConfig(
      backendId: cfg.backendId,
      field: cfg.field,
      hashId: cfg.hashId,
      friArity: cfg.friArity,
      profileId: cfg.profileId,
      airPath: airPath,
      publicInputsJson: cfg.publicInputsJson,
    );
  }

  Future<void> _handleInit() async {
    if (_initInProgress) {
      return;
    }
    _updateState(() {
      _initInProgress = true;
      _statusMessage = 'Initializing...';
    });
    try {
      await _ensureAirAsset();
      final backends = await listBackends();
      final summary = _summarizeBackends(backends);
      _updateState(() {
        _backendCount = summary.count;
        _firstBackendId = summary.firstId;
        _statusMessage = summary.count > 0
            ? 'Init complete — ${summary.count} backend(s)'
            : 'Init complete';
      });
    } catch (error) {
      _updateState(() {
        _statusMessage = 'Init failed: $error';
      });
    } finally {
      _updateState(() {
        _initInProgress = false;
      });
    }
  }

  Future<void> _handleProve() async {
    if (_proveInProgress) {
      return;
    }
    _updateState(() {
      _proveInProgress = true;
      _statusMessage = 'Proving...';
      _verified = null;
    });
    try {
      await _ensureAirAsset();
      final result = await prove(_configWithAir());
      _updateState(() {
        _proof = result.proof;
        _digest = result.digest;
        _statusMessage = 'Proof ready (len=${result.proofLength})';
        _verified = null;
      });
    } catch (error) {
      _updateState(() {
        _statusMessage = 'Prove failed: $error';
      });
    } finally {
      _updateState(() {
        _proveInProgress = false;
      });
    }
  }

  Future<void> _handleVerify() async {
    if (_verifyInProgress) {
      return;
    }
    final proof = _proof;
    if (proof == null || proof.isEmpty) {
      _updateState(() {
        _statusMessage = 'Generate a proof before verifying';
        _verified = false;
      });
      return;
    }
    _updateState(() {
      _verifyInProgress = true;
      _statusMessage = 'Verifying...';
    });
    try {
      await _ensureAirAsset();
      final result = await verify(_configWithAir(), proof);
      _updateState(() {
        _verified = result.verified;
        if (result.digest.isNotEmpty) {
          _digest = result.digest;
        }
        _statusMessage = 'Verify complete';
      });
    } catch (error) {
      _updateState(() {
        _statusMessage = 'Verify failed: $error';
        _verified = false;
      });
    } finally {
      _updateState(() {
        _verifyInProgress = false;
      });
    }
  }

  Future<void> _handleTamperProof() async {
    if (_verifyInProgress || _proveInProgress) {
      return;
    }
    final proof = _proof;
    if (proof == null || proof.isEmpty) {
      _updateState(() {
        _statusMessage = 'Generate a proof before tampering';
      });
      return;
    }
    final mutated = Uint8List.fromList(proof);
    mutated[0] = (mutated[0] ^ 0x01) & 0xff;
    _updateState(() {
      _proof = mutated;
      _verified = null;
      _statusMessage = 'Proof tampered — flipped byte 0';
    });
    await _handleVerify();
  }

  @override
  Widget build(BuildContext context) {
    final backendDisplay =
        _backendCount == null ? '-' : _backendCount.toString();
    final firstBackendDisplay =
        (_firstBackendId == null || _firstBackendId!.isEmpty)
            ? '-'
            : _firstBackendId!;
    final digestDisplay = _digest.isEmpty ? '-' : _formatDigest(_digest);
    final verifiedDisplay =
        _verified == null ? '-' : _verified == true ? 'true' : 'false';
    final verificationColor = _verified == null
        ? Theme.of(context).textTheme.bodyMedium?.color
        : _verified == true
            ? Colors.green.shade700
            : Colors.red.shade600;

    return Scaffold(
      appBar: AppBar(
        title: const Text('ZKProv Demo'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Backends available: $backendDisplay'),
              const SizedBox(height: 4),
              Text('First backend id: $firstBackendDisplay'),
              const SizedBox(height: 12),
              Text('Digest D: $digestDisplay'),
              const SizedBox(height: 12),
              Text(
                'Verified: $verifiedDisplay',
                style: TextStyle(color: verificationColor),
              ),
              const SizedBox(height: 24),
              Wrap(
                spacing: 12,
                runSpacing: 12,
                children: [
                  ElevatedButton(
                    onPressed: _initInProgress ? null : _handleInit,
                    child: _initInProgress
                        ? const SizedBox(
                            height: 20,
                            width: 20,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('Init'),
                  ),
                  ElevatedButton(
                    onPressed: _proveInProgress ? null : _handleProve,
                    child: _proveInProgress
                        ? const SizedBox(
                            height: 20,
                            width: 20,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('Prove'),
                  ),
                  ElevatedButton(
                    onPressed: _verifyInProgress ? null : _handleVerify,
                    child: _verifyInProgress
                        ? const SizedBox(
                            height: 20,
                            width: 20,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('Verify'),
                  ),
                  OutlinedButton.icon(
                    onPressed:
                        _verifyInProgress ? null : () => _handleTamperProof(),
                    icon: const Icon(Icons.warning_amber_rounded),
                    label: const Text('Tamper proof'),
                  ),
                ],
              ),
              const SizedBox(height: 24),
              Text(_statusMessage ?? 'Ready'),
            ],
          ),
        ),
      ),
    );
  }
}

class _BackendListSummary {
  const _BackendListSummary({required this.count, this.firstId});

  final int count;
  final String? firstId;
}

_BackendListSummary _summarizeBackends(Map<String, dynamic> json) {
  final dynamic directItems = json['items'] ?? json['backends'];
  if (directItems is List) {
    final String? firstId =
        directItems.isNotEmpty ? _backendIdFromEntry(directItems.first) : null;
    return _BackendListSummary(
      count: directItems.length,
      firstId: firstId,
    );
  }
  if (directItems is Map) {
    String? firstId;
    if (directItems.isNotEmpty) {
      final entry = directItems.entries.first;
      firstId =
          _backendIdFromEntry(entry.value) ?? _stringValue(entry.key);
    }
    return _BackendListSummary(
      count: directItems.length,
      firstId: firstId,
    );
  }

  final int count = json['count'] is int
      ? json['count'] as int
      : (directItems is Iterable
          ? directItems.length
          : json.length);
  String? firstId;
  final dynamic candidates = directItems ?? json['default_backend'];
  if (candidates != null) {
    firstId = _backendIdFromEntry(candidates);
  }
  firstId ??= _backendIdFromEntry(json['default']) ??
      _findFirstBackendId(json.values);
  return _BackendListSummary(count: count, firstId: firstId);
}

String? _backendIdFromEntry(dynamic entry) {
  if (entry is Map) {
    final dynamic id =
        entry['backend'] ?? entry['backend_id'] ?? entry['id'] ?? entry['name'];
    return _stringValue(id);
  }
  if (entry is List && entry.isNotEmpty) {
    return _backendIdFromEntry(entry.first);
  }
  return _stringValue(entry);
}

String? _stringValue(dynamic value) {
  if (value is String && value.isNotEmpty) {
    return value;
  }
  return null;
}

String? _findFirstBackendId(Iterable<dynamic> values) {
  for (final value in values) {
    final candidate = _backendIdFromEntry(value);
    if (candidate != null) {
      return candidate;
    }
  }
  return null;
}

String _formatDigest(String digest) {
  if (digest.length <= 16) {
    return digest;
  }
  const keep = 8;
  return '${digest.substring(0, keep)}…${digest.substring(digest.length - keep)}';
}
