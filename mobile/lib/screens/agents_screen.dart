import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/agent.dart';
import '../services/ws_service.dart';

class AgentsScreen extends StatelessWidget {
  const AgentsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Agents')),
      body: Consumer<AgentProvider>(
        builder: (context, provider, _) {
          if (provider.agents.isEmpty) {
            return const Center(child: Text('No agents running', style: TextStyle(color: Colors.grey)));
          }
          return ListView.builder(
            itemCount: provider.agents.length,
            itemBuilder: (context, i) {
              final agent = provider.agents[i];
              return ListTile(
                leading: CircleAvatar(child: Icon(_iconFor(agent.kind))),
                title: Text(agent.kind),
                subtitle: Text(agent.status),
                trailing: _statusDot(agent.status),
              );
            },
          );
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showSpawnDialog(context),
        child: const Icon(Icons.add),
      ),
    );
  }

  IconData _iconFor(String kind) {
    switch (kind) {
      case 'claude': return Icons.psychology;
      case 'codex': return Icons.code;
      case 'opencode': return Icons.terminal;
      default: return Icons.smart_toy;
    }
  }

  Widget _statusDot(String status) {
    final color = switch (status) {
      'running' => Colors.green,
      'thinking' => Colors.orange,
      'error' => Colors.red,
      _ => Colors.grey,
    };
    return Container(width: 12, height: 12, decoration: BoxDecoration(color: color, shape: BoxShape.circle));
  }

  void _showSpawnDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Spawn Agent'),
        content: const Text('Enter agent kind and prompt'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Cancel')),
          FilledButton(onPressed: () => Navigator.pop(ctx), child: const Text('Spawn')),
        ],
      ),
    );
  }
}
