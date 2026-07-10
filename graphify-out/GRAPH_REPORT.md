# Graph Report - .  (2026-07-10)

## Corpus Check
- 204 files · ~91,775 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 57 nodes · 65 edges · 12 communities (10 shown, 2 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]

## God Nodes (most connected - your core abstractions)
1. `SkillRegistry` - 13 edges
2. `SkillManifest` - 6 edges
3. `home_dir()` - 5 edges
4. `handle_health()` - 5 edges
5. `AuthMethod` - 5 edges
6. `NavigationResult` - 3 edges
7. `NavigationStatus` - 3 edges
8. `Command` - 3 edges
9. `AppConfig` - 2 edges
10. `Create` - 1 edges

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Import Cycles
- None detected.

## Communities (12 total, 2 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.22
Nodes (7): Option, String, SkillManifest, SkillRegistry, Default, HashMap, Vec

### Community 1 - "Community 1"
Cohesion: 0.50
Nodes (5): AppConfig, home_dir(), Result, PathBuf, PorpoiseError

### Community 2 - "Community 2"
Cohesion: 0.25
Nodes (6): AuthMethod, Option, Result, String, Debug, Formatter

### Community 3 - "Community 3"
Cohesion: 0.40
Nodes (5): handle_health(), Result, DateTime, Utc, Value

### Community 4 - "Community 4"
Cohesion: 0.40
Nodes (4): Create, Delete, Read, Update

### Community 5 - "Community 5"
Cohesion: 0.83
Nodes (3): NavigationResult, NavigationStatus, String

### Community 6 - "Community 6"
Cohesion: 0.50
Nodes (3): Command, Send, Sync

## Knowledge Gaps
- **5 isolated node(s):** `Create`, `Read`, `Update`, `Delete`, `pre-commit.sh script`
  These have ≤1 connection - possible missing edges or undocumented components.
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SkillRegistry` connect `Community 0` to `Community 7`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **What connects `Create`, `Read`, `Update` to the rest of the system?**
  _5 weakly-connected nodes found - possible documentation gaps or missing edges._