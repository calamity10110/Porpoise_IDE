import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/ws_service.dart';

class PairScreen extends StatefulWidget {
  const PairScreen({super.key});

  @override
  State<PairScreen> createState() => _PairScreenState();
}

class _PairScreenState extends State<PairScreen> {
  final _hostController = TextEditingController(text: 'porpoise.local');
  final _portController = TextEditingController(text: '9876');

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
            TextField(controller: _hostController, decoration: const InputDecoration(labelText: 'Host', border: OutlineInputBorder())),
            const SizedBox(height: 16),
            TextField(controller: _portController, decoration: const InputDecoration(labelText: 'Port', border: OutlineInputBorder()), keyboardType: TextInputType.number),
            const SizedBox(height: 24),
            SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: _pair,
                icon: const Icon(Icons.qr_code_scanner),
                label: const Text('Scan QR Code'),
              ),
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: OutlinedButton(
                onPressed: _pair,
                child: const Text('Connect Manually'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _pair() async {
    final ws = context.read<WsService>();
    try {
      await ws.connect(_hostController.text, int.parse(_portController.text));
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Connected!')));
        Navigator.of(context).pop(true);
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Failed: $e')));
      }
    }
  }
}
