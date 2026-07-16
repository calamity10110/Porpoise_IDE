import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/ws_service.dart';
import 'pair_screen.dart';

/// Settings screen with connection-status indicator and a disconnect button.
class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  StreamSubscription<bool>? _connectionSub;
  bool _connected = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final ws = context.read<WsService>();
      setState(() => _connected = ws.isConnected);
      _connectionSub = ws.connectionChanges.listen((connected) {
        if (mounted) setState(() => _connected = connected);
      });
    });
  }

  @override
  void dispose() {
    _connectionSub?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        children: [
          ListTile(
            leading: Icon(
              _connected ? Icons.link : Icons.link_off,
              color: _connected ? Colors.green : Colors.red,
            ),
            title: Text(_connected ? 'Connected' : 'Disconnected'),
            subtitle: Text(
              _connected
                  ? 'Porpoise daemon is connected'
                  : 'Not paired with a daemon',
            ),
            trailing: _connected
                ? FilledButton.tonalIcon(
                    onPressed: _disconnect,
                    icon: const Icon(Icons.power_settings_new, size: 18),
                    label: const Text('Disconnect'),
                    style: FilledButton.styleFrom(
                      foregroundColor: Colors.red.shade300,
                    ),
                  )
                : null,
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.link),
            title: const Text('Pair with Desktop'),
            subtitle: const Text('Connect to your Porpoise daemon'),
            trailing: const Icon(Icons.chevron_right),
            enabled: !_connected,
            onTap: () async {
              final result = await Navigator.push<bool>(
                context,
                MaterialPageRoute(builder: (_) => const PairScreen()),
              );
              if (result == true && mounted) {
                setState(() => _connected = true);
              }
            },
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
          const ListTile(
            leading: Icon(Icons.info),
            title: Text('About'),
            subtitle: Text('Porpoise v0.1.0'),
          ),
        ],
      ),
    );
  }

  void _disconnect() {
    final ws = context.read<WsService>();
    ws.disconnect();
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Disconnected from daemon')),
    );
  }
}
