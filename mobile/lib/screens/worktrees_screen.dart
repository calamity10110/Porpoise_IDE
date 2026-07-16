import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/worktree.dart';
import '../services/ws_service.dart';

class WorktreesScreen extends StatelessWidget {
  const WorktreesScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Worktrees')),
      body: Consumer<WorktreeProvider>(
        builder: (context, provider, _) {
          if (provider.worktrees.isEmpty) {
            return const Center(
              child: Text('No worktrees', style: TextStyle(color: Colors.grey)),
            );
          }
          return ListView.builder(
            itemCount: provider.worktrees.length,
            itemBuilder: (context, i) {
              final wt = provider.worktrees[i];
              return ListTile(
                leading: const Icon(Icons.folder, color: Color(0xFF7aa2f7)),
                title: Text(wt.name),
                subtitle: Text('${wt.branch} \u2022 ${wt.path}'),
              );
            },
          );
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showCreateDialog(context),
        child: const Icon(Icons.add),
      ),
    );
  }

  void _showCreateDialog(BuildContext context) {
    final nameController = TextEditingController();
    final branchController = TextEditingController(text: 'main');

    showDialog(
      context: context,
      builder: (ctx) {
        var creating = false;
        return StatefulBuilder(
          builder: (innerContext, setDialogState) {
            return AlertDialog(
              title: const Text('Create Worktree'),
              content: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextField(
                    controller: nameController,
                    decoration: const InputDecoration(
                      labelText: 'Name',
                      border: OutlineInputBorder(),
                    ),
                    enabled: !creating,
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: branchController,
                    decoration: const InputDecoration(
                      labelText: 'Branch',
                      hintText: 'main',
                      border: OutlineInputBorder(),
                    ),
                    enabled: !creating,
                  ),
                ],
              ),
              actions: [
                TextButton(
                  onPressed: creating ? null : () => Navigator.pop(ctx),
                  child: const Text('Cancel'),
                ),
                FilledButton(
                  onPressed: creating
                      ? null
                      : () async {
                          setDialogState(() => creating = true);
                          try {
                            final ws = context.read<WsService>();
                            await ws.call('worktree/create', {
                              'name': nameController.text,
                              'branch': branchController.text,
                            });
                            if (ctx.mounted) Navigator.pop(ctx);
                          } catch (e) {
                            setDialogState(() => creating = false);
                            if (ctx.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(content: Text('Failed: $e')),
                              );
                            }
                          }
                        },
                  child: creating
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Create'),
                ),
              ],
            );
          },
        );
      },
    );
  }
}
