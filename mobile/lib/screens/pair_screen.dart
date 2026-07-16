import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/ws_service.dart';

/// Screen for pairing the mobile app with the Porpoise desktop daemon.
///
/// Supports manual host:port entry and a stub for QR-code scanning.
class PairScreen extends StatefulWidget {
  const PairScreen({super.key});

  @override
  State<PairScreen> createState() => _PairScreenState();
}

class _PairScreenState extends State<PairScreen> {
  final _hostController = TextEditingController(text: 'porpoise.local');
  final _portController = TextEditingController(text: '9876');
  bool _connecting = false;

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Pair with Desktop')),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
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
          ],
        ),
      ),
    );
  }

  void _scanQr() {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text(
          'QR scanning requires the qr_code_scanner_plus package. '
          'See the comment in pair_screen.dart for migration instructions.',
        ),
      ),
    );
    // TODO: Replace with real QR scanner when qr_code_scanner_plus is added:
    //   import 'package:qr_code_scanner_plus/qr_code_scanner_plus.dart';
    //   final result = await Navigator.push(
    //     context,
    //     MaterialPageRoute(
    //       builder: (_) => QRView(
    //         key: qrKey,
    //         onQRViewCreated: (controller) {
    //           controller.scannedDataStream.listen((barcode) {
    //             final parts = barcode.code!.split(':');
    //             if (parts.length == 2) {
    //               _connect(parts[0], int.parse(parts[1]));
    //             }
    //           });
    //         },
    //       ),
    //     ),
    //   );
  }

  Future<void> _pair() async {
    setState(() => _connecting = true);
    try {
      await _connect(
        _hostController.text,
        int.parse(_portController.text),
      );
    } on FormatException {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Invalid port number'),
            backgroundColor: Colors.red,
          ),
        );
        setState(() => _connecting = false);
      }
    } catch (_) {
      // Error already handled in _connect.
    }
  }

  Future<void> _connect(String host, int port) async {
    final ws = context.read<WsService>();
    try {
      await ws.connect(host, port);
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
