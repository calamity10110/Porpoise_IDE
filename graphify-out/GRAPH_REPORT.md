# Graph Report - C:\Users\vuanh\Downloads\porpoises\Porpoise_ide  (2026-07-01)

## Corpus Check
- Corpus is ~33,940 words - fits in a single context window. You may not need a graph.

## Summary
- 37 nodes · 71 edges · 14 communities detected
- Extraction: 82% EXTRACTED · 18% INFERRED · 0% AMBIGUOUS · INFERRED: 13 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Agents  Architecture  Core Types|Agents / Architecture / Core Types]]
- [[_COMMUNITY_Crate porpoise-core  Crate porpoise-skills  Design Skills|Crate: porpoise-core / Crate: porpoise-skills / Design Skills]]
- [[_COMMUNITY_Crate porpoise-app  Design App|Crate: porpoise-app / Design App]]
- [[_COMMUNITY_Crate porpoise-db  Design Db|Crate: porpoise-db / Design Db]]
- [[_COMMUNITY_Crate porpoise-ssh  Design Ssh|Crate: porpoise-ssh / Design Ssh]]
- [[_COMMUNITY_Crate porpoise-agent  Design Agent|Crate: porpoise-agent / Design Agent]]
- [[_COMMUNITY_Crate porpoise-browser  Design Browser|Crate: porpoise-browser / Design Browser]]
- [[_COMMUNITY_Crate porpoise-relay  Design Relay|Crate: porpoise-relay / Design Relay]]
- [[_COMMUNITY_Crate porpoise-runtime  Design Runtime|Crate: porpoise-runtime / Design Runtime]]
- [[_COMMUNITY_Crate porpoise-cli  Design Cli|Crate: porpoise-cli / Design Cli]]
- [[_COMMUNITY_Crate porpoise-terminal  Design Terminal|Crate: porpoise-terminal / Design Terminal]]
- [[_COMMUNITY_Crate porpoise-git  Design Git|Crate: porpoise-git / Design Git]]
- [[_COMMUNITY_Crate porpoise-server  Design Server|Crate: porpoise-server / Design Server]]
- [[_COMMUNITY_Crate porpoise-network  Design Network|Crate: porpoise-network / Design Network]]

## God Nodes (most connected - your core abstractions)
1. `Roadmap` - 16 edges
2. `Agents` - 15 edges
3. `Crate: porpoise-core` - 14 edges
4. `Readme` - 8 edges
5. `Todo` - 6 edges
6. `Development Plan` - 6 edges
7. `Core Types` - 6 edges
8. `Runtime Design` - 5 edges
9. `Architecture` - 4 edges
10. `Protocol Design` - 4 edges

## Surprising Connections (you probably didn't know these)
- `Design Core` --designs--> `Crate: porpoise-core`  [EXTRACTED]
  docs\design\design-core.md → crates\porpoise-core\src\bus.rs
- `Crate: porpoise-agent` --depends_on--> `Crate: porpoise-core`  [INFERRED]
  crates\porpoise-agent\src\lib.rs → crates\porpoise-core\src\bus.rs
- `Crate: porpoise-app` --depends_on--> `Crate: porpoise-core`  [INFERRED]
  crates\porpoise-app\src\lib.rs → crates\porpoise-core\src\bus.rs
- `Crate: porpoise-browser` --depends_on--> `Crate: porpoise-core`  [INFERRED]
  crates\porpoise-browser\src\lib.rs → crates\porpoise-core\src\bus.rs
- `Crate: porpoise-cli` --depends_on--> `Crate: porpoise-core`  [INFERRED]
  crates\porpoise-cli\src\app.rs → crates\porpoise-core\src\bus.rs

## Communities

### Community 0 - "Agents / Architecture / Core Types"
Cohesion: 0.62
Nodes (10): Agents, Architecture, Core Types, Development Plan, Protocol Design, Readme, Roadmap, Runtime Design (+2 more)

### Community 1 - "Crate: porpoise-core / Crate: porpoise-skills / Design Skills"
Cohesion: 0.67
Nodes (3): Crate: porpoise-core, Crate: porpoise-skills, Design Skills

### Community 2 - "Crate: porpoise-app / Design App"
Cohesion: 1.0
Nodes (2): Crate: porpoise-app, Design App

### Community 3 - "Crate: porpoise-db / Design Db"
Cohesion: 1.0
Nodes (2): Crate: porpoise-db, Design Db

### Community 4 - "Crate: porpoise-ssh / Design Ssh"
Cohesion: 1.0
Nodes (2): Crate: porpoise-ssh, Design Ssh

### Community 5 - "Crate: porpoise-agent / Design Agent"
Cohesion: 1.0
Nodes (2): Crate: porpoise-agent, Design Agent

### Community 6 - "Crate: porpoise-browser / Design Browser"
Cohesion: 1.0
Nodes (2): Crate: porpoise-browser, Design Browser

### Community 7 - "Crate: porpoise-relay / Design Relay"
Cohesion: 1.0
Nodes (2): Crate: porpoise-relay, Design Relay

### Community 8 - "Crate: porpoise-runtime / Design Runtime"
Cohesion: 1.0
Nodes (2): Crate: porpoise-runtime, Design Runtime

### Community 9 - "Crate: porpoise-cli / Design Cli"
Cohesion: 1.0
Nodes (2): Crate: porpoise-cli, Design Cli

### Community 10 - "Crate: porpoise-terminal / Design Terminal"
Cohesion: 1.0
Nodes (2): Crate: porpoise-terminal, Design Terminal

### Community 11 - "Crate: porpoise-git / Design Git"
Cohesion: 1.0
Nodes (2): Crate: porpoise-git, Design Git

### Community 12 - "Crate: porpoise-server / Design Server"
Cohesion: 1.0
Nodes (2): Crate: porpoise-server, Design Server

### Community 13 - "Crate: porpoise-network / Design Network"
Cohesion: 1.0
Nodes (2): Crate: porpoise-network, Design Network

## Knowledge Gaps
- **2 isolated node(s):** `Design Cli`, `Design Relay`
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Crate: porpoise-app / Design App`** (2 nodes): `Crate: porpoise-app`, `Design App`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-db / Design Db`** (2 nodes): `Crate: porpoise-db`, `Design Db`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-ssh / Design Ssh`** (2 nodes): `Crate: porpoise-ssh`, `Design Ssh`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-agent / Design Agent`** (2 nodes): `Crate: porpoise-agent`, `Design Agent`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-browser / Design Browser`** (2 nodes): `Crate: porpoise-browser`, `Design Browser`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-relay / Design Relay`** (2 nodes): `Crate: porpoise-relay`, `Design Relay`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-runtime / Design Runtime`** (2 nodes): `Crate: porpoise-runtime`, `Design Runtime`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-cli / Design Cli`** (2 nodes): `Crate: porpoise-cli`, `Design Cli`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-terminal / Design Terminal`** (2 nodes): `Crate: porpoise-terminal`, `Design Terminal`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-git / Design Git`** (2 nodes): `Crate: porpoise-git`, `Design Git`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-server / Design Server`** (2 nodes): `Crate: porpoise-server`, `Design Server`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Crate: porpoise-network / Design Network`** (2 nodes): `Crate: porpoise-network`, `Design Network`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Crate: porpoise-core` connect `Crate: porpoise-core / Crate: porpoise-skills / Design Skills` to `Agents / Architecture / Core Types`, `Crate: porpoise-app / Design App`, `Crate: porpoise-db / Design Db`, `Crate: porpoise-ssh / Design Ssh`, `Crate: porpoise-agent / Design Agent`, `Crate: porpoise-browser / Design Browser`, `Crate: porpoise-relay / Design Relay`, `Crate: porpoise-runtime / Design Runtime`, `Crate: porpoise-cli / Design Cli`, `Crate: porpoise-terminal / Design Terminal`, `Crate: porpoise-git / Design Git`, `Crate: porpoise-server / Design Server`, `Crate: porpoise-network / Design Network`?**
  _High betweenness centrality (0.423) - this node is a cross-community bridge._
- **Why does `Roadmap` connect `Agents / Architecture / Core Types` to `Crate: porpoise-core / Crate: porpoise-skills / Design Skills`, `Crate: porpoise-app / Design App`, `Crate: porpoise-db / Design Db`, `Crate: porpoise-ssh / Design Ssh`, `Crate: porpoise-agent / Design Agent`, `Crate: porpoise-browser / Design Browser`, `Crate: porpoise-terminal / Design Terminal`, `Crate: porpoise-git / Design Git`, `Crate: porpoise-server / Design Server`, `Crate: porpoise-network / Design Network`?**
  _High betweenness centrality (0.297) - this node is a cross-community bridge._
- **Why does `Agents` connect `Agents / Architecture / Core Types` to `Crate: porpoise-db / Design Db`, `Crate: porpoise-ssh / Design Ssh`, `Crate: porpoise-agent / Design Agent`, `Crate: porpoise-browser / Design Browser`, `Crate: porpoise-runtime / Design Runtime`, `Crate: porpoise-server / Design Server`?**
  _High betweenness centrality (0.209) - this node is a cross-community bridge._
- **Are the 13 inferred relationships involving `Crate: porpoise-core` (e.g. with `Crate: porpoise-skills` and `Crate: porpoise-browser`) actually correct?**
  _`Crate: porpoise-core` has 13 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Design Cli`, `Design Relay` to the rest of the system?**
  _2 weakly-connected nodes found - possible documentation gaps or missing edges._