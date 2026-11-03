"""Index actual formal75 factory/dispatch/byte-reader paths, without execution.

Every link retains source line spans and the complete selected method body.
Catalogue definitions remain historical observations; no current cast is inferred.
"""
from pathlib import Path
from datetime import datetime, timezone
import bisect
import hashlib
import json
import re
import zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
encode = lambda value: (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
w = read(R / '工作佇列.json')
source = w['current_product_identity']['source_manifest_sha256']
assert source == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
scope_path = V / 'native-current-skill-request-scope/20261020-044016-468694/scope.json'
scope = read(scope_path)
assert scope['source_manifest_sha256'] == source and scope['missing_count'] == 37
assert sha(scope['current_request_source']) == scope['current_request_source_sha256']
book_path = Path(scope['prior_observed_book'])
assert sha(book_path) == scope['prior_observed_book_sha256']
reference_path = V / 'native-hotel-return-contract/20261019-192705-656308/readback.json'
reference = read(reference_path)
original = Path(reference['original_disassembly'])
assert sha(original) == reference['original_disassembly_sha256'] == '964DC949F7EA6C575A8E725D025EA67B94B73ABF81C200BBE2BA190D72248CF1'
jar = R / 'LiTo381-Modern/target/current/server-reference/formal-server-75DA69B0.jar'
assert sha(jar) == reference['formal_jar_sha256'] == '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
original_receipt = read(reference['original_contract_readback'])
assert sha(reference['original_contract_readback']) == reference['original_contract_readback_sha256']
assert original_receipt['formal_jar_sha256'] == sha(jar)
lines = original.read_text('utf-8-sig').splitlines()
starts = [i for i, line in enumerate(lines) if re.match(r'^  \S.*\(.*\).*;\s*$', line)]
blocks = []
for index, start in enumerate(starts):
    end = starts[index + 1] if index + 1 < len(starts) else len(lines)
    header = lines[start].strip()
    name = re.search(r'([\w$]+)\(', header)
    assert name, header
    text = '\n'.join(lines[start:end]) + '\n'
    blocks.append(dict(header=header, method=name[1], start_line=start + 1, end_line=end, text=text,
                       sha256=hashlib.sha256(text.encode('utf-8')).hexdigest().upper()))
by_name = {}
for block in blocks:
    by_name.setdefault(block['method'], []).append(block)

factory_pattern = re.compile(r'// Method local/lito381/server/BattleSpellCatalog\.([\w$]+):\(I\)Llocal/lito381/server/BattleSpellCatalog\$([^;]+);')
factory_sites = [(index, match) for index, line in enumerate(lines) if (match := factory_pattern.search(line))]
selected = {}
rows = []
for definition in scope['missing_definitions']:
    skill = definition['skill_id']
    links = []
    for catalog in definition['definitions']:
        factory, record = catalog['factory'], catalog['record_type']
        for site, match in factory_sites:
            if (match[1], match[2]) != (factory, record):
                continue
            caller_index = bisect.bisect_right(starts, site) - 1
            assert caller_index >= 0
            caller = blocks[caller_index]
            end = starts[caller_index + 1] if caller_index + 1 < len(starts) else len(lines)
            store_pattern = re.compile(r'^\s*\d+:\s+astore(?:_(\d+)|\s+(\d+))\s*$')
            load_pattern = re.compile(r'^\s*\d+:\s+aload(?:_(\d+)|\s+(\d+))\s*$')
            stored = store_pattern.match(lines[site + 1]) if site + 1 < end else None
            if not stored:
                continue
            slot = int(stored[1] or stored[2])
            # Other record factories may be looked up before this one is used.
            # Follow the actual stored local until it is reassigned, rather than
            # truncating at an unrelated factory and losing the attack branch.
            next_store = next((number for number in range(site + 2, end)
                               if (m := store_pattern.match(lines[number])) and int(m[1] or m[2]) == slot), end)
            # Signature binds actual byte[] and exact catalogue record arguments;
            # unrelated effect accessors and proc helpers cannot become readers.
            helper_pattern = re.compile(r'// Method (?:local/lito381/server/ClientSession\.)?([\w$]+):\(\[BLlocal/lito381/server/BattleSpellCatalog\$' + re.escape(record) + r';(I?)\)V')
            for number in range(site + 2, next_store):
                helper = helper_pattern.search(lines[number])
                if not helper:
                    continue
                nearby = [index for index in range(max(site + 2, number - (20 if helper[2] else 1)), number)
                          if (m := load_pattern.match(lines[index])) and int(m[1] or m[2]) == slot]
                if not nearby:
                    continue
                suffix = ', int)' if helper[2] else ')'
                matches = [block for block in by_name.get(helper[1], []) if '(byte[], local.lito381.server.BattleSpellCatalog$' + record + suffix in block['header']]
                assert len(matches) == 1, (factory, helper[1], matches)
                block = matches[0]
                selected[block['header']] = block
                selected[caller['header']] = caller
                direct = sorted(set(re.findall(r'// Method local/lito381/server/Protocol\.([\w$]+Request):', block['text'])))
                forwards = []
                # Keep same-family overloads, so byte wrappers that forward parsed
                # requests retain their guard/cost/effect implementation as well.
                for sibling in by_name.get(helper[1], []):
                    if sibling['header'] != block['header'] and 'BattleSpellCatalog$' + record in sibling['header']:
                        selected[sibling['header']] = sibling
                        forwards.append(sibling['header'])
                link = dict(factory=factory, record_type=record, dispatch_method=caller['header'], factory_line=site + 1,
                            helper_call_line=number + 1, observed_factory_astore_and_helper_aload_slot=slot,
                            helper_argument_load_lines=[index + 1 for index in nearby],
                            extra_integer_argument_present=bool(helper[2]),
                            extra_integer_argument_stack_semantics_review_required=bool(helper[2]),
                            helper_signature=block['header'], helper_start_line=block['start_line'],
                            helper_end_line=block['end_line'], direct_Protocol_request_readers=direct,
                            same_name_record_overloads_for_review=forwards,
                            dispatch_branch_selection_per_skill_verified=False, runtime_activation_verified=False)
                if link not in links:
                    links.append(link)
    rows.append(dict(skill_id=skill, historical_catalogue_definitions=definition['definitions'], handler_paths=links,
                     static_link_present=bool(links), selected_request_shape_verified=False,
                     target_range_cost_lifecycle_semantics_review_complete=False))
assert sorted(row['skill_id'] for row in rows) == scope['missing_request_ids']
assert len(rows) == 37
out = V / 'native-current-skill-request-scope' / ('handler-paths-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'selected-methods.json').write_bytes(encode(selected))
(out / 'auditor.py').write_bytes(Path(__file__).read_bytes())
report = dict(status='formal75_missing_request_factory_handler_reader_paths_static_only', source_manifest_sha256=source,
              scope=str(scope_path), scope_sha256=sha(scope_path), historical_book=str(book_path), historical_book_sha256=sha(book_path),
              formal_jar_sha256=sha(jar), original_disassembly=str(original), original_disassembly_sha256=sha(original),
              original_contract_readback=str(reference_path), original_contract_readback_sha256=sha(reference_path),
              current_request_source_sha256=sha(scope['current_request_source']), missing_IDs=37,
              IDs_with_static_handler_paths=sum(row['static_link_present'] for row in rows),
              unresolved_IDs=[row['skill_id'] for row in rows if not row['static_link_present']],
              selected_methods=str(out / 'selected-methods.json'), selected_methods_sha256=sha(out / 'selected-methods.json'),
              selected_method_count=len(selected), rows=rows, runtime_started=False, new_sessions=0, new_casts=0,
              normal_product_or_active_QA_modified=False, request_shape_or_runtime_qualification_granted=False,
              whole_goal_complete=False)
(out / 'paths.json').write_bytes(encode(report))
files = {path.name: path for path in out.iterdir() if path.is_file()}
pins = {name: sha(path) for name, path in files.items()}
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name, path in files.items():
        archive.write(path, name)
with zipfile.ZipFile(out / 'raw.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in pins.items())
(out / 'readback.json').write_bytes(encode(dict(raw_files=pins, raw_archive_sha256=sha(out / 'raw.zip'), paths_sha256=sha(out / 'paths.json'),
                                              normal_product_or_active_QA_modified=False, new_sessions=0, new_casts=0)))
print(json.dumps(dict(paths=str(out / 'paths.json'), linked_IDs=report['IDs_with_static_handler_paths'], unresolved_IDs=report['unresolved_IDs'], selected_method_count=len(selected), static_only=True), ensure_ascii=False))
