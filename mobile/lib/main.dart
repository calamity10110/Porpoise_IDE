import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'services/ws_service.dart';
import 'screens/agents_screen.dart';
import 'screens/worktrees_screen.dart';
import 'screens/terminal_screen.dart';
import 'screens/settings_screen.dart';
import 'screens/pair_screen.dart';
import 'models/worktree.dart';
import 'models/agent.dart';
import 'models/terminal.dart';

void main() {
  runApp(const PorpoiseApp());
}

class PorpoiseApp extends StatelessWidget {
  const PorpoiseApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        Provider<WsService>(create: (_) => WsService()),
        ChangeNotifierProvider<WorktreeProvider>(create: (_) => WorktreeProvider()),
        ChangeNotifierProvider<AgentProvider>(create: (_) => AgentProvider()),
        ChangeNotifierProvider<TerminalProvider>(create: (_) => TerminalProvider()),
      ],
      child: MaterialApp(
        title: 'Porpoise',
        theme: ThemeData.dark().copyWith(
          colorScheme: ColorScheme.dark(
            primary: const Color(0xFF7aa2f7),
            secondary: const Color(0xFFbb9af7),
            surface: const Color(0xFF24283b),
          ),
          scaffoldBackgroundColor: const Color(0xFF1a1b26),
        ),
        home: const HomeScreen(),
      ),
    );
  }
}

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  int _selectedIndex = 0;

  final List<Widget> _screens = const [
    AgentsScreen(),
    WorktreesScreen(),
    TerminalScreen(),
    SettingsScreen(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: _screens[_selectedIndex],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _selectedIndex,
        onDestinationSelected: (i) => setState(() => _selectedIndex = i),
        destinations: const [
          NavigationDestination(icon: Icon(Icons.smart_toy), label: 'Agents'),
          NavigationDestination(icon: Icon(Icons.folder), label: 'Worktrees'),
          NavigationDestination(icon: Icon(Icons.terminal), label: 'Terminal'),
          NavigationDestination(icon: Icon(Icons.settings), label: 'Settings'),
        ],
      ),
    );
  }
}
