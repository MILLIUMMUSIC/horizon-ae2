#!/usr/bin/env python3
"""Verify that the item-phase bridge preserves the pinned native tick body.

The current injection API cannot select a site inside this closure-bearing
method. A cancellable HEAD calls the source body carried in the mod, with three
explicit AE2 insertion regions. No original engine file is edited by this tool.
"""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def audit(engine, payload):
    start = engine.index('    fn tick_items(&mut self) {')
    end = engine.index('\n    /// Run vanilla block behaviour', start)
    original = engine[start:end]
    carried = payload[payload.index('pub(super) fn tick('):].strip()
    regions = re.findall(r'// AE2_BEGIN ([\w-]+)\n.*?// AE2_END \1\n', carried, re.S)
    if regions != ['initial', 'entity-return', 'outputs']:
        raise ValueError(f'unexpected item phase insertion regions: {regions}')
    stripped = re.sub(r'[ \t]*// AE2_BEGIN ([\w-]+)\n.*?[ \t]*// AE2_END \1\n', '', carried, flags=re.S)
    original_body = re.sub(r'\bself\b', 'game', original[original.index('{'):])
    stripped_body = stripped[stripped.index('{'):]
    def canonical(value):
        # Parse/format Rust rather than deleting whitespace: whitespace inside
        # literals and the newline terminating a line comment are semantic.
        run = subprocess.run(['rustfmt', '--edition', '2024', '--emit', 'stdout'],
                             input='fn carried_body() ' + value, text=True,
                             capture_output=True, check=True)
        return run.stdout
    original_canonical = canonical(original_body)
    if original_canonical != canonical(stripped_body):
        raise ValueError('carried item tick differs outside the three AE2 regions')
    # Anchor the per-entity callback after lethal environment removal and before
    # the next entity; a batch-RETURN callback samples a different world state.
    if not re.search(r'if !environment.alive \{.*?cells.remove\(i, item.position\);\s*\}\s*'
                     r'// AE2_BEGIN entity-return\s+if environment.alive', carried, re.S):
        raise ValueError('item callback is not at the surviving per-entity RETURN phase')
    return {
        'passed': True,
        'engine_file_sha256': hashlib.sha256(engine.encode()).hexdigest(),
        'payload_sha256': hashlib.sha256(payload.encode()).hexdigest(),
        'original_body_sha256': hashlib.sha256(original_canonical.encode()).hexdigest(),
        'regions': regions,
        'boundary': 'Unmodified pinned tick outside marked mod insertions; no source-game equivalence claim',
    }


def main():
    compat = json.loads((ROOT / 'compatibility.json').read_text())
    name = 'crates/horizon-client/src/game.rs'
    engine = (ROOT / 'target/upstream/horizon' / name).read_text()
    if hashlib.sha256(engine.encode()).hexdigest() != compat['source_sha256'][name]:
        raise ValueError('engine source does not match compatibility.json')
    report = audit(engine, (ROOT / 'mixins/ae2/item_tick.rs').read_text())
    report['horizon_revision'] = compat['horizon_revision']
    out = ROOT / 'target/item-tick-source-audit.json'
    out.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
