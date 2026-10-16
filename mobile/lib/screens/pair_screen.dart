import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:qr_code_scanner_plus/qr_code_scanner_plus.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../services/ws_service.dart';

/// Screen for pairing the mobile app with the Porpoise desktop daemon.
///
/// Supports scanning the QR printed by `porpoise launch`, or manual
/// host/port/token entry.
class PairScreen extends StatefulWidget {
  const PairScreen({super.key});

  @override
  State<PairScreen> createState() => _PairScreenState();
}

class _PairScreenState extends State<PairScreen> {
  final _hostController = TextEditingController(text: 'porpoise.local');
  final _portController = TextEditingController(text: '9876');
  final _tokenController = TextEditingController();
  bool _connecting = false;

  @override
  void initState() {
    super.initState();
    // Rebuild on every keystroke so the pairing QR stays in sync.
    for (final controller in [_hostController, _portController, _tokenController]) {
      controller.addListener(() {
        if (mounted) setState(() {});
      });
    }
  }

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    _tokenController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Pair with Desktop')),
      body: SafeArea(
        child: SingleChildScrollView(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Icon(Icons.link, size: 64, color: Color(0xFF7aa2f7)),
                const SizedBox(height: 24),
                TextField(
                  controller: _hostController,
                  decoration: const InputDecoration(
                    labelText: 'Host',
                    border: OutlineInputBorder(),
                  ),
                  enabled: !_connecting,
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: _portController,
                  decoration: const InputDecoration(
                    labelText: 'Port',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
                  enabled: !_connecting,
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: _tokenController,
                  decoration: const InputDecoration(
                    labelText: 'Pairing token',
                    helperText: 'Shown by `porpoise launch` on the desktop',
                    border: OutlineInputBorder(),
                  ),
                  enabled: !_connecting,
                ),
                const SizedBox(height: 24),
                SizedBox(
                  width: double.infinity,
                  child: FilledButton.icon(
                    onPressed: _connecting ? null : _scanQr,
                    icon: const Icon(Icons.qr_code_scanner),
                    label: const Text('Scan QR Code'),
                  ),
                ),
                const SizedBox(height: 12),
                SizedBox(
                  width: double.infinity,
                  child: OutlinedButton(
                    onPressed: _connecting ? null : _pair,
                    style: OutlinedButton.styleFrom(
                      foregroundColor: Theme.of(context).colorScheme.primary,
                    ),
                    child: _connecting
                        ? const SizedBox(
                            width: 20,
                            height: 20,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('Connect Manually'),
                  ),
                ),
                if (_connecting) ...[
                  const SizedBox(height: 16),
                  const Text(
                    'Connecting...',
                    style: TextStyle(color: Colors.grey),
                  ),
                ],
                ..._buildQrDisplay(),
              ],
            ),
          ),
        ),
      ),
    );
  }

  /// Live QR of the current pairing info so a second device can pair by
  /// scanning this screen. Only shown once host, port and token are filled in.
  List<Widget> _buildQrDisplay() {
    final host = _hostController.text.trim();
    final port = _portController.text.trim();
    final token = _tokenController.text.trim();
    if (_connecting || host.isEmpty || port.isEmpty || token.isEmpty) {
      return const [];
    }
    return [
      const SizedBox(height: 32),
      Text(
        'Pairing QR (scan from another device)',
        textAlign: TextAlign.center,
        style: TextStyle(color: Colors.grey.shade600, fontSize: 12),
      ),
      const SizedBox(height: 8),
      Center(
        child: QrImageView(
          data: 'ws://$host:$port?token=$token',
          size: 180,
          backgroundColor: Colors.white,
        ),
      ),
    ];
  }

  Future<void> _scanQr() async {
    final payload = await Navigator.of(context).push<String>(
      MaterialPageRoute(builder: (_) => const _QrScanScreen()),
    );
    if (payload == null || !mounted) return;

    final parsed = _parsePairingPayload(payload);
    if (parsed == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Not a Porpoise pairing QR: $payload'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }
    setState(() {
      _hostController.text = parsed.$1;
      _portController.text = parsed.$2;
      _tokenController.text = parsed.$3;
      _connecting = true;
    });
    await _connect(parsed.$1, int.parse(parsed.$2), parsed.$3);
  }

  /// Parse the payload printed as a QR by `porpoise launch`:
  /// `ws(s)://<host>:<port>?token=<token>`. Returns (host, port, token).
  static (String, String, String)? _parsePairingPayload(String payload) {
    final Uri uri;
    try {
      uri = Uri.parse(payload.trim());
    } on FormatException {
      return null;
    }
    if ((uri.scheme != 'ws' && uri.scheme != 'wss') || uri.host.isEmpty) {
      return null;
    }
    final port = uri.port;
    final token = uri.queryParameters['token'];
    if (port <= 0 || token == null || token.isEmpty) return null;
    return (uri.host, port.toString(), token);
  }

  Future<void> _pair() async {
    final port = int.tryParse(_portController.text.trim());
    if (port == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Invalid port number'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }
    setState(() => _connecting = true);
    await _connect(_hostController.text.trim(), port, _tokenController.text.trim());
  }

  Future<void> _connect(String host, int port, String token) async {
    final ws = context.read<WsService>();
    try {
      await ws.connect(host, port, token: token);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Connected!'),
            backgroundColor: Colors.green,
          ),
        );
        Navigator.of(context).pop(true);
      }
    } catch (e) {
      if (mounted) {
        setState(() => _connecting = false);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Connection failed: $e'),
            backgroundColor: Colors.red,
            action: SnackBarAction(
              label: 'Retry',
              onPressed: _pair,
            ),
          ),
        );
      }
    }
  }
}

/// Full-screen camera view that pops with the scanned QR payload.
class _QrScanScreen extends StatefulWidget {
  const _QrScanScreen();

  @override
  State<_QrScanScreen> createState() => _QrScanScreenState();
}

class _QrScanScreenState extends State<_QrScanScreen> {
  final _qrKey = GlobalKey(debugLabel: 'pairing-qr');
  QRViewController? _controller;
  bool _popped = false;

  @override
  void reassemble() {
    super.reassemble();
    _controller?.resumeCamera();
  }

  @override
  void dispose() {
    _controller?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Scan Pairing QR')),
      body: QRView(
        key: _qrKey,
        onQRViewCreated: _onQrViewCreated,
        overlay: QrScannerOverlayShape(
          borderColor: const Color(0xFF7aa2f7),
          borderRadius: 12,
          borderLength: 28,
          borderWidth: 8,
          cutOutSize: 260,
        ),
      ),
    );
  }

  void _onQrViewCreated(QRViewController controller) {
    _controller = controller;
    controller.scannedDataStream.listen((barcode) {
      final code = barcode.code;
      if (code == null || _popped || !mounted) return;
      _popped = true;
      Navigator.of(context).pop(code);
    });
  }
}
