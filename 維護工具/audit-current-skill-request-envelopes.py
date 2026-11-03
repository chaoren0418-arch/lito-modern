"""Map retained C02 factory calls to direct Protocol request readers; static evidence only."""
from pathlib import Path
import argparse
import bisect
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / '驗證紀錄/current/native-current-skill-contract-audit'
ap = argparse.ArgumentParser()
ap.add_argument('--output', type=Path, required=True)
a = ap.parse_args()
output = a.output.resolve()
assert output.is_relative_to(OUT) and not output.exists()
rawpath = ROOT / '驗證紀錄/current/native-shared-higher-heals/legacy-bridge/C02-ClientSession-reviewed.javap.txt'
lines = rawpath.read_text('utf-8').splitlines()
factory = json.loads((OUT / 'current-learned-factories.json').read_text('utf-8-sig'))
decl = []
for i, line in enumerate(lines):
    if re.match(r'^  \S', line) and '(' in line and line.endswith(';'):
        m = re.search(r'([\w$]+)\([^;]*\)', line)
        if m:
            decl.append((i, m[1], line))
starts = [x[0] for x in decl]
byname = {}
for n, (start, name, header) in enumerate(decl):
    byname.setdefault(name, []).append((start, decl[n+1][0] if n+1 < len(decl) else len(lines), header))
methods = {m['factory']: m['record_type'] for row in factory['rows'] for m in row['matches']}
links = {name: [] for name in methods}
for i, line in enumerate(lines):
    m = re.search(r'// Method local/lito381/server/BattleSpellCatalog\.([\w$]+):\(I\)Llocal/lito381/server/BattleSpellCatalog\$([^;]+);', line)
    if not m or m[1] not in methods:
        continue
    caller = decl[bisect.bisect_right(starts, i)-1][2]
    record = m[2]
    helpers = []
    for j in range(i+1, min(i+33, len(lines))):
        if 'invokevirtual' in lines[j] and f'BattleSpellCatalog${record};' in lines[j]:
            mm = re.search(r'// Method (?:local/lito381/server/ClientSession\.)?([\w$]+):', lines[j])
            if mm:
                helpers.append(mm[1])
        if 'invokestatic' in lines[j] and 'BattleSpellCatalog.' in lines[j]:
            break
    for helper in sorted(set(helpers)):
        for start, end, header in byname.get(helper, []):
            if f'BattleSpellCatalog${record}' not in header or 'byte[]' not in header:
                continue
            block = '\n'.join(lines[start:end])
            decoders = sorted(set(re.findall(r'// Method local/lito381/server/Protocol\.([\w$]*Request):', block)))
            entry = {'dispatch_method': caller.strip(), 'factory_line': i+1, 'helper': helper,
                     'helper_signature': header.strip(), 'helper_lines': [start+1, end],
                     'direct_Protocol_request_decoders': decoders}
            if entry not in links[m[1]]:
                links[m[1]].append(entry)
report = {'status': 'static_C02_current_learned_factory_to_direct_request_decoder_map',
          'JAR_sha256': 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539',
          'retained_ClientSession_javap_sha256': hashlib.sha256(rawpath.read_bytes()).hexdigest().upper(),
          'retained_ClientSession_javap': str(rawpath), 'factory_count_observed': factory['factory_count'],
          'actual_learned_unique_ids': len(factory['rows']), 'factory_links': links,
          'rows': [{'skill_id': x['skill_id'], 'definitions': x['matches'], 'factory_errors': x['factory_errors']} for x in factory['rows']],
          'actual_new_sessions': 0, 'actual_new_casts': 0,
          'formal_phase_activation_or_full_packet_contract_verified': False, 'whole_goal_complete': False}
output.write_text(json.dumps(report, ensure_ascii=False, indent=2), 'utf-8')
print(json.dumps({'learned_IDs': len(factory['rows']), 'factories_used': len(methods),
                  'factories_with_direct_handler_links': sum(bool(v) for v in links.values()),
                  'new_sessions': 0}))
