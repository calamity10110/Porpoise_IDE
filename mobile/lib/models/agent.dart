class Agent {
  final String id;
  final String kind;
  final String status;

  Agent({required this.id, required this.kind, required this.status});

  factory Agent.fromJson(Map<String, dynamic> json) {
    return Agent(id: json['id'] ?? '', kind: json['kind'] ?? '', status: json['status'] ?? '');
  }
}

class AgentProvider extends ChangeNotifier {
  List<Agent> _agents = [];
  String _lastOutput = '';

  List<Agent> get agents => _agents;
  String get lastOutput => _lastOutput;

  void update(List<Agent> agents) {
    _agents = agents;
    notifyListeners();
  }

  void appendOutput(String text) {
    _lastOutput = text;
    notifyListeners();
  }
}
