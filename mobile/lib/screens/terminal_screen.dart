import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/terminal.dart';
import '../services/ws_service.dart';

class TerminalScreen extends StatefulWidget {
  const TerminalScreen({super.key});

  @override
  State<TerminalScreen> createState() => _TerminalScreenState();
}

class _TerminalScreenState extends State<TerminalScreen> {
  final _inputController = TextEditingController();
  String? _selectedTerminalId;

  @override
  void dispose() {
    _inputController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Terminal')),
      body: Consumer<TerminalProvider>(
        builder: (context, provider, _) {
          if (provider.terminals.isEmpty) {
            return const Center(
              child: Text('No terminals', style: TextStyle(color: Colors.grey)),
            );
          }

          if (_selectedTerminalId != null &&
              !provider.terminals.any((t) => t.id == _selectedTerminalId)) {
            _selectedTerminalId = null;
          }
          _selectedTerminalId ??= provider.terminals.first.id;

          final selectedTerminal = provider.terminals
              .firstWhere((t) => t.id == _selectedTerminalId);

          return Column(
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(12, 8, 12, 0),
                child: DropdownButtonFormField<String>(
                  value: _selectedTerminalId,
                  decoration: const InputDecoration(
                    labelText: 'Select Terminal',
                    border: OutlineInputBorder(),
                    contentPadding:
                        EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                    isDense: true,
                  ),
                  items: provider.terminals.map((t) {
                    return DropdownMenuItem(
                      value: t.id,
                      child: Text('Terminal ${t.id}'),
                    );
                  }).toList(),
                  onChanged: (id) {
                    if (id != null) setState(() => _selectedTerminalId = id);
                  },
                ),
              ),
              Expanded(
                child: SingleChildScrollView(
                  padding: const EdgeInsets.all(12),
                  child: SelectableText(
                    selectedTerminal.output.isEmpty
                        ? 'Waiting for output...'
                        : selectedTerminal.output,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 13,
                      height: 1.4,
                    ),
                  ),
                ),
              ),
              Container(
                padding: const EdgeInsets.fromLTRB(8, 4, 8, 8),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surface,
                  border: Border(
                    top: BorderSide(color: Theme.of(context).dividerColor),
                  ),
                ),
                child: Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: _inputController,
                        decoration: const InputDecoration(
                          hintText: 'Enter command...',
                          border: OutlineInputBorder(),
                          contentPadding: EdgeInsets.symmetric(
                            horizontal: 12,
                            vertical: 8,
                          ),
                          isDense: true,
                        ),
                        style: const TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 13,
                        ),
                        onSubmitted: (_) => _sendCommand(),
                      ),
                    ),
                    const SizedBox(width: 8),
                    IconButton.filled(
                      onPressed: _sendCommand,
                      icon: const Icon(Icons.send, size: 20),
                      tooltip: 'Send command',
                    ),
                  ],
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  void _sendCommand() {
    final text = _inputController.text.trim();
    if (text.isEmpty || _selectedTerminalId == null) return;

    final ws = context.read<WsService>();
    ws.call('terminal/send', {
      'terminal_id': _selectedTerminalId,
      'text': text,
    }).catchError((e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Send failed: $e')),
        );
      }
      return <String, dynamic>{};
    });

    _inputController.clear();
  }
}
