import sys, json
from pathlib import Path
from graphify.detect import detect
from graphify.cluster import cluster, score_all
from graphify.analyze import god_nodes, surprising_connections, suggest_questions
from graphify.report import generate
from graphify.export import to_json
from graphify.build import build_from_json
import networkx as nx

project_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
out_dir = project_dir / 'graphify-out'
out_dir.mkdir(exist_ok=True)

detection = detect(project_dir)
G = nx.Graph()

# Add document nodes
for fpath in detection['files']['document']:
    p = Path(fpath)
    try:
        rel = p.relative_to(project_dir)
    except ValueError:
        rel = p
    node_id = p.stem
    label = p.stem.replace('-', ' ').replace('_', ' ').title()
    G.add_node(node_id, label=label, file_type='document', source_file=str(rel))

# Add code crate nodes (group by crate)
seen_crates = set()
for fpath in detection['files']['code']:
    p = Path(fpath)
    parts = p.parts
    try:
        crate_idx = parts.index('crates')
        if crate_idx + 1 < len(parts):
            crate_name = parts[crate_idx + 1]
            if crate_name not in seen_crates:
                seen_crates.add(crate_name)
                node_id = f"crate_{crate_name}"
                try:
                    rel = p.relative_to(project_dir)
                except ValueError:
                    rel = p
                G.add_node(node_id, label=f"Crate: {crate_name}", file_type='code', source_file=str(rel))
    except ValueError:
        pass

# Document reference edges
for fpath in detection['files']['document']:
    p = Path(fpath)
    try:
        content = p.read_text(encoding='utf-8', errors='ignore')
    except Exception:
        continue
    source_node = p.stem
    for other in detection['files']['document']:
        other_p = Path(other)
        if other_p.stem != source_node:
            ref_name = other_p.stem.replace('-', ' ').replace('_', ' ')
            if ref_name.lower() in content.lower() or other_p.stem.lower() in content.lower():
                G.add_edge(source_node, other_p.stem, relation='references',
                          confidence='EXTRACTED', confidence_score=1.0,
                          source_file=str(p.relative_to(project_dir)))

# Design doc to crate edges
for fpath in detection['files']['document']:
    p = Path(fpath)
    if p.stem.startswith('design-'):
        crate_suffix = p.stem.replace('design-', '')
        crate_node = f"crate_porpoise-{crate_suffix}"
        if G.has_node(crate_node):
            G.add_edge(p.stem, crate_node, relation='designs', 
                      confidence='EXTRACTED', confidence_score=1.0,
                      source_file=str(p.relative_to(project_dir)))

# Crate to core dependency edges
for crate in seen_crates:
    if crate != 'porpoise-core':
        core_node = 'crate_porpoise-core'
        if G.has_node(core_node):
            G.add_edge(core_node, f"crate_{crate}", relation='depends_on',
                      confidence='INFERRED', confidence_score=0.8, source_file='')

# Cluster and build report
communities = cluster(G)
cohesion = score_all(G, communities)
gods = god_nodes(G)
surprises = surprising_connections(G, communities)

labels = {}
for cid, nodes in communities.items():
    names = [G.nodes[n].get('label', n) for n in nodes[:3]]
    labels[cid] = ' / '.join(names)

tokens = {'input': 0, 'output': 0}
questions = suggest_questions(G, communities, labels)

report = generate(G, communities, cohesion, labels, gods, surprises, detection, tokens,
                 str(project_dir), suggested_questions=questions)
(out_dir / 'GRAPH_REPORT.md').write_text(report, encoding='utf-8')
to_json(G, communities, str(out_dir / 'graph.json'))

print(f"Graph: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges, {len(communities)} communities")
print(f"Report size: {(out_dir / 'GRAPH_REPORT.md').stat().st_size} bytes")
print(f"\nCommunities:")
for cid in sorted(communities.keys()):
    print(f"  C{cid}: {labels.get(cid, 'Unlabeled')} ({len(communities[cid])} nodes)")
print(f"\nGod nodes (top 5):")
for node, score in gods[:5]:
    label = G.nodes[node].get('label', node)
    print(f"  {label}: {score} connections")
