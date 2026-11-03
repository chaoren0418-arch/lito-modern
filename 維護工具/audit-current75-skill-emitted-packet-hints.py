"""Index direct emitted packet methods and lexical native routes; static only."""
from pathlib import Path
from datetime import datetime, timezone
import ast
import hashlib
import json
import re
import zipfile
R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
encode = lambda value: (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
paths = V / 'native-current-skill-request-scope/handler-paths-20261020-071630-321475/paths.json'
mapping = read(paths)
source = mapping['source_manifest_sha256']
queue = read(R / '工作佇列.json')
assert source == queue['current_product_identity']['source_manifest_sha256']
assert sha(mapping['selected_methods']) == mapping['selected_methods_sha256']
handlers = read(mapping['selected_methods'])
protocol_receipt = V / 'native-current-skill-request-scope/protocol75-20261020-081500-851844/readback.json'
receipt = read(protocol_receipt)
assert receipt['jar_sha256'] == mapping['formal_jar_sha256'] == '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(receipt['jar']) == receipt['jar_sha256']
assert receipt['Protocol_class_sha256'] == '4C23161BF4DF6C09981DE829B4E361A5F61FEED9354BAB62CAC8E6DE1E062E4B'
protocol = Path(receipt['disassembly'])
assert sha(protocol) == receipt['disassembly_sha256']
lines = protocol.read_text('utf-8-sig').splitlines()
starts = [index for index, line in enumerate(lines) if re.match(r'^  \S.*\(.*\).*;\s*$', line)]
methods = {}
for number, start in enumerate(starts):
    end = starts[number + 1] if number + 1 < len(starts) else len(lines)
    header = lines[start].strip()
    text = '\n'.join(lines[start:end]) + '\n'
    name = re.search(r'([\w$]+)\(', header)
    descriptor = re.search(r'^    descriptor: (\([^\n]+)', text, re.M)
    assert name and descriptor
    methods[(name[1], descriptor[1])] = dict(header=header, descriptor=descriptor[1], start_line=start + 1,
                                             end_line=end, text=text, sha256=hashlib.sha256(text.encode()).hexdigest().upper())


def first_writer_literal(block):
    own = block['text'].splitlines()
    candidates = [index for index, line in enumerate(own) if 'Protocol$Writer.c:(I)' in line]
    if not candidates:
        return None
    previous = own[candidates[0] - 1]
    value = re.search(r':\s+(?:bipush|sipush)\s+(-?\d+)\s*$', previous)
    if not value:
        value = re.search(r':\s+iconst_(\d+)\s*$', previous)
    if not value:
        value = re.search(r'// int (-?\d+)\s*$', previous)
    return int(value[1]) if value and 0 <= int(value[1]) <= 255 else None


# Reuse only the already reviewed pure Rust function-boundary parser, not the
# old source-specific planner's CLI, authority, output or qualification code.
parser_file = R / '維護工具/prepare-current-shared-resource-library-plan.py'
parsed = ast.parse(parser_file.read_text('utf-8-sig'))
function = next(node for node in parsed.body if isinstance(node, ast.FunctionDef) and node.name == 'function_body')
namespace = {'re': re}
exec(compile(ast.Module(body=[function], type_ignores=[]), str(parser_file), 'exec'), namespace)
world_path = R / 'LiTo381-Modern/src/modern_lito/native_world.rs'
compiled_inputs = read(Path(queue['current_native_installed_resource_profile_normal_qualification']['run']) / 'compiled-inputs.json')
assert sha(world_path) == compiled_inputs[world_path.relative_to(R).as_posix()]
world_text = world_path.read_text('utf-8-sig')
world_start = world_text[:world_text.index('pub fn accept(')].count('\n') + 1
world_body, _, _ = namespace['function_body'](world_text, 'accept', world_start)
world_hints = {}
for line in world_body.splitlines():
    arm = re.match(r'^ {12}(0x[0-9A-Fa-f]+(?:\s*\|\s*0x[0-9A-Fa-f]+)*)(?:\s+if\s+.*?)?\s*=>', line)
    if arm:
        for value in re.findall(r'0x[0-9A-Fa-f]+', arm[1]):
            world_hints.setdefault(int(value, 16), []).append(line.strip())
    early = re.match(r'^ {8}if\s+op\s*==\s*(0x[0-9A-Fa-f]+)', line)
    if early:
        world_hints.setdefault(int(early[1], 16), []).append(line.strip())
    early = re.match(r'^ {8}if matches!\(op,\s*((?:0x[0-9A-Fa-f]+\s*\|?\s*)+)\)', line)
    if early:
        for value in re.findall(r'0x[0-9A-Fa-f]+', early[1]):
            world_hints.setdefault(int(value, 16), []).append(line.strip())
selected = {}
rows = []
call_pattern = re.compile(r'// Method local/lito381/server/Protocol\.([\w$]+):(\([^)]*\)\[B)')
for row in mapping['rows']:
    signatures = {entry['helper_signature'] for entry in row['handler_paths']}
    signatures.update(signature for entry in row['handler_paths'] for signature in entry['same_name_record_overloads_for_review'])
    calls = {}
    for signature in signatures:
        for name, descriptor in call_pattern.findall(handlers[signature]['text']):
            key = (name, descriptor)
            assert key in methods, key
            block = methods[key]
            selected[name + ':' + descriptor] = block
            opcode = first_writer_literal(block)
            calls[name + ':' + descriptor] = dict(method=name, descriptor=descriptor,
                                                  inferred_first_writer_c_literal=opcode,
                                                  opcode_hex=f'{opcode:02X}' if opcode is not None else None,
                                                  Protocol_method_start_line=block['start_line'], Protocol_method_sha256=block['sha256'],
                                                  native_World_accept_lexical_hints=world_hints.get(opcode, []),
                                                  route_or_payload_semantics_verified=False)
    rows.append(dict(skill_id=row['skill_id'], factory=row['historical_catalogue_definitions'][0]['factory'],
                     possible_family_helper_signatures=sorted(signatures), direct_emitted_Protocol_byte_array_calls=list(calls.values()),
                     actual_dispatch_branch_per_ID_verified=False, nested_helpers_async_callbacks_and_other_owners_not_exhausted=True,
                     live_runtime_verified=False))
assert len(rows) == 37
out = V / 'native-current-skill-request-scope' / ('emitted-packet-hints-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'selected-Protocol-methods.json').write_bytes(encode(selected))
record = dict(status='formal75_missing_skills_direct_emitted_methods_and_lexical_World_routes_static_hints_only',
              source_manifest_sha256=source, handler_paths=str(paths), handler_paths_sha256=sha(paths),
              selected_handler_methods=mapping['selected_methods'], selected_handler_methods_sha256=mapping['selected_methods_sha256'],
              Protocol_receipt=str(protocol_receipt), Protocol_receipt_sha256=sha(protocol_receipt),
              Protocol_class_sha256=receipt['Protocol_class_sha256'], Protocol_disassembly=str(protocol), Protocol_disassembly_sha256=sha(protocol),
              Rust_function_parser=str(parser_file), Rust_function_parser_sha256=sha(parser_file),
              native_World_source=str(world_path), native_World_source_sha256=sha(world_path),
              selected_Protocol_methods=str(out / 'selected-Protocol-methods.json'), selected_Protocol_methods_sha256=sha(out / 'selected-Protocol-methods.json'),
              rows=rows, missing_skill_IDs=37, direct_Protocol_methods=len(selected),
              interpretation='Factory-family supersets only: actual per-ID phase/branch and nested callbacks are not solved. First Writer.c literal is a bytecode hint, not a runtime frame. A World lexical gap is not a product decoder defect; Login/Inventory/Environment and other owners must be reviewed. No packet/layout/state/render/audio acceptance follows.',
              new_sessions=0, new_casts=0, normal_source_or_active_QA_modified=False,
              packet_or_runtime_qualification_granted=False, whole_goal_complete=False)
(out / 'hints.json').write_bytes(encode(record))
(out / 'auditor.py').write_bytes(Path(__file__).read_bytes())
files = {path.name: path for path in out.iterdir() if path.is_file()}
pins = {name: sha(path) for name, path in files.items()}
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name, path in files.items():
        archive.write(path, name)
with zipfile.ZipFile(out / 'raw.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in pins.items())
(out / 'readback.json').write_bytes(encode(dict(raw_files=pins, raw_archive_sha256=sha(out / 'raw.zip'), hints_sha256=sha(out / 'hints.json'), new_sessions=0, new_casts=0)))
print(json.dumps(dict(hints=str(out / 'hints.json'), skill_IDs=37, direct_Protocol_methods=len(selected), runtime_qualified=False), ensure_ascii=False))
