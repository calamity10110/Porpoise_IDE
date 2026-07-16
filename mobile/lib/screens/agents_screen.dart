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
            return const Center(
              child: Text('No agents running', style: TextStyle(color: Colors.grey)),
            );
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
                onTap: () => _showAgentOutput(context, agent),
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
      case 'claude':
        return Icons.psychology;
      case 'codex':
        return Icons.code;
      case 'opencode':
        return Icons.terminal;
      default:
        return Icons.smart_toy;
    }
  }

  Widget _statusDot(String status) {
    final color = switch (status) {
      'running' => Colors.green,
      'thinking' => Colors.orange,
      'error' => Colors.red,
      _ => Colors.grey,
    };
    return Container(
      width: 12,
      height: 12,
      decoration: BoxDecoration(color: color, shape: BoxShape.circle),
    );
  }

  void _showAgentOutput(BuildContext context, Agent agent) {
    final provider = context.read<AgentProvider>();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Agent: ${agent.kind} (${agent.id})'),
        content: SingleChildScrollView(
          child: SelectableText(
            provider.lastOutput.isEmpty
                ? 'No output yet.'
                : provider.lastOutput,
            style: const TextStyle(fontFamily: 'monospace', fontSize: 13),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  void _showSpawnDialog(BuildContext context) {
    final kindController = TextEditingController();
    final worktreeController = TextEditingController();
    final promptController = TextEditingController();

    showDialog(
      context: context,
      builder: (ctx) {
        var spawning = false;
        return StatefulBuilder(
          builder: (innerContext, setDialogState) {
            return AlertDialog(
              title: const Text('Spawn Agent'),
              content: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextField(
                      controller: kindController,
                      decoration: const InputDecoration(
                        labelText: 'Agent Kind',
                        hintText: 'claude, codex, opencode',
                        border: OutlineInputBorder(),
                      ),
                      enabled: !spawning,
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      controller: worktreeController,
                      decoration: const InputDecoration(
                        labelText: 'Worktree',
                        hintText: 'Optional: worktree name',
                        border: OutlineInputBorder(),
                      ),
                      enabled: !spawning,
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      controller: promptController,
                      decoration: const InputDecoration(
                        labelText: 'Prompt',
                        hintText: 'What should the agent do?',
                        border: OutlineInputBorder(),
                      ),
                      maxLines: 3,
                      enabled: !spawning,
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: spawning ? null : () => Navigator.pop(ctx),
                  child: const Text('Cancel'),
                ),
                FilledButton(
                  onPressed: spawning
                      ? null
                      : () async {
                          setDialogState(() => spawning = true);
                          try {
                            final ws = context.read<WsService>();
                            await ws.call('agent/spawn', {
                              'kind': kindController.text,
                              'worktree': worktreeController.text,
                              'prompt': promptController.text,
                            });
                            if (ctx.mounted) Navigator.pop(ctx);
                          } catch (e) {
                            setDialogState(() => spawning = false);
                            if (ctx.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(content: Text('Failed: $e')),
                              );
                            }
                          }
                        },
                  child: spawning
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Spawn'),
                ),
              ],
            );
          },
        );
      },
    );
  }
}
