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
      final count = _extractBackendCount(backends);
      _updateState(() {
        _backendCount = count;
        _statusMessage = 'Init complete';
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

  @override
  Widget build(BuildContext context) {
    final backendDisplay = _backendCount == null
        ? '-'
        : _backendCount.toString();
    final digestDisplay = _digest.isEmpty ? '-' : _digest;
    final verifiedDisplay =
        _verified == null ? '-' : _verified == true ? 'true' : 'false';

    return Scaffold(
      appBar: AppBar(
        title: const Text('ZKProv Demo'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Backends JSON length: $backendDisplay'),
            const SizedBox(height: 8),
            Text('Last digest D: $digestDisplay'),
            const SizedBox(height: 8),
            Text('Verified: $verifiedDisplay'),
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
              ],
            ),
            const SizedBox(height: 24),
            Text(_statusMessage ?? 'Ready'),
          ],
        ),
      ),
    );
  }
}

int _extractBackendCount(Map<String, dynamic> json) {
  final dynamic directItems = json['items'] ?? json['backends'];
  if (directItems is List) {
    return directItems.length;
  }
  if (directItems is Map) {
    return directItems.length;
  }
  return json.length;
}
