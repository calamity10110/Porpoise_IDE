import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/worktree.dart';

class WorktreesScreen extends StatelessWidget {
  const WorktreesScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Worktrees')),
      body: Consumer<WorktreeProvider>(
        builder: (context, provider, _) {
          if (provider.worktrees.isEmpty) {
            return const Center(child: Text('No worktrees', style: TextStyle(color: Colors.grey)));
          }
          return ListView.builder(
            itemCount: provider.worktrees.length,
            itemBuilder: (context, i) {
              final wt = provider.worktrees[i];
              return ListTile(
                leading: const Icon(Icons.folder, color: Color(0xFF7aa2f7)),
                title: Text(wt.name),
                subtitle: Text('${wt.branch} • ${wt.path}'),
              );
            },
          );
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {},
        child: const Icon(Icons.add),
      ),
    );
  }
}
