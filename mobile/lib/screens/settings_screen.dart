import 'package:flutter/material.dart';
import 'pair_screen.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        children: [
          ListTile(
            leading: const Icon(Icons.link),
            title: const Text('Pair with Desktop'),
            subtitle: const Text('Connect to your Porpoise daemon'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => Navigator.push(context, MaterialPageRoute(builder: (_) => const PairScreen())),
          ),
          const Divider(),
          const ListTile(
            leading: Icon(Icons.account_circle),
            title: Text('Default Agent'),
            subtitle: Text('claude'),
          ),
          const Divider(),
          const ListTile(
            leading: Icon(Icons.palette),
            title: Text('Theme'),
            subtitle: Text('Dark'),
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.info),
            title: const Text('About'),
            subtitle: const Text('Porpoise v0.1.0'),
          ),
        ],
      ),
    );
  }
}
