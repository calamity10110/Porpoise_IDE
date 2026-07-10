class Terminal {
  final String id;
  final String sessionId;
  final int rows;
  final int cols;
  String output;

  Terminal({required this.id, required this.sessionId, this.rows = 24, this.cols = 80, this.output = ''});

  factory Terminal.fromJson(Map<String, dynamic> json) {
    return Terminal(id: json['id'] ?? '', sessionId: json['session_id'] ?? '');
  }
}

class TerminalProvider extends ChangeNotifier {
  List<Terminal> _terminals = [];

  List<Terminal> get terminals => _terminals;

  void update(List<Terminal> terminals) {
    _terminals = terminals;
    notifyListeners();
  }

  void appendOutput(String terminalId, String text) {
    for (final t in _terminals) {
      if (t.id == terminalId) {
        t.output += text;
        break;
      }
    }
    notifyListeners();
  }
}
