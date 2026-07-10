import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/terminal.dart';

class TerminalScreen extends StatelessWidget {
  const TerminalScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Terminal')),
      body: Consumer<TerminalProvider>(
        builder: (context, provider, _) {
          if (provider.terminals.isEmpty) {
            return const Center(child: Text('No terminals', style: TextStyle(color: Colors.grey)));
          }
          return ListView.builder(
            itemCount: provider.terminals.length,
            itemBuilder: (context, i) {
              final t = provider.terminals[i];
              return Card(
                margin: const EdgeInsets.all(8),
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Terminal ${t.id}', style: const TextStyle(fontWeight: FontWeight.bold)),
                      const SizedBox(height: 8),
                      Text(t.output, style: const TextStyle(fontFamily: 'monospace', fontSize: 12)),
                    ],
                  ),
                ),
              );
            },
          );
        },
      ),
    );
  }
}
