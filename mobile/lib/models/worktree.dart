class Worktree {
  final String name;
  final String path;
  final String branch;

  Worktree({required this.name, required this.path, required this.branch});

  factory Worktree.fromJson(Map<String, dynamic> json) {
    return Worktree(
      name: json['name'] ?? '',
      path: json['path'] ?? '',
      branch: json['branch'] ?? '',
    );
  }
}

class WorktreeProvider extends ChangeNotifier {
  List<Worktree> _worktrees = [];

  List<Worktree> get worktrees => _worktrees;

  void update(List<Worktree> worktrees) {
    _worktrees = worktrees;
    notifyListeners();
  }
}
