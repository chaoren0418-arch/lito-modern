"""Retain call-site evidence without assigning unknown imports pixel semantics."""
import hashlib
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).resolve().parents[1]
MAP = ROOT / '驗證紀錄/current/native-lighting-map-policy'
OUT = ROOT / '驗證紀錄/current/native-lighting-render-call-contexts'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
qualification = read(MAP / 'qualified.json')
selected = MAP / 'selected-assembly.json'
assert qualification['archive_readback_verified']
assert sha(selected) == qualification['file_pins']['selected-assembly.json']
world = read(selected)['sub_488860']
lines = world['assembly'].split('\n')
sites = []
for index, line in enumerate(lines):
    match = re.search(r'call\s+near ptr (5A[0-9A-F]+)h', line)
    if not match:
        continue
    context = '\n'.join(lines[max(0, index-12):min(len(lines), index+12)])
    target = '0x0' + match.group(1)
    resource_error_context = target == '0x05AC0B76' and 'aImageResourceE' in context and 'sub_497C60' in context
    sites.append({'target': target, 'source_newline_line': world['first_newline_line']+index,
                  'preceded_by_image_resource_error_handling': resource_error_context,
                  'context': context})
targets = sorted({site['target'] for site in sites})
assert targets == ['0x05A01CE6', '0x05A917B9', '0x05AC0000', '0x05AC09C4', '0x05AC0B76']
errors = [site for site in sites if site['target'] == '0x05AC0B76']
assert errors and all(site['preceded_by_image_resource_error_handling'] for site in errors)
thread_site = next(site for site in sites if site['target'] == '0x05A01CE6')
coordinate_site = next(site for site in sites if site['target'] == '0x05AC0000')
assert 'push    offset sub_487230' in thread_site['context']
assert 'lea     edx, [ebp+var_10]' in coordinate_site['context']
assert 'mov     [ebp+var_10], edi' in coordinate_site['context']
assert 'mov     [ebp+var_C], edi' in coordinate_site['context']
normal_path = ROOT / '驗證紀錄/current/native-light-spell/normal/qualified.json'
normal = read(normal_path)
pins = read(normal_path.parent / 'source-inputs.json')
assert len(pins) == 970
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
proof = {'status': 'original_render_far_call_sites_classified_by_context_only',
    'original_PE_sha256': qualification['original_PE_sha256'],
    'original_ASM_sha256': qualification['original_ASM_sha256'],
    'retained_selected_assembly_sha256': sha(selected), 'function': 'sub_488860',
    'far_call_sites': len(sites), 'distinct_targets': targets,
    'image_resource_error_call_sites': len(errors), 'sites': sites,
    'conclusion': 'Every selected5AC0B76 call is immediately in an Image Resource Error path; other selected calls include callback setup and coordinate-tail contexts. Call addresses alone do not establish a missing lighting routine.',
    'next_source_action': 'Trace successful draw virtual-method targets and ambient data flow before treating the far import targets as a lighting dependency.',
    'Windows_API_names_resolved': False, 'module_ownership_resolved': False,
    'pixel_lighting_algorithm_resolved': False, 'framebuffer_verified': False,
    'product_source_changes': 0, 'client_source_files_unchanged': 970,
    'game_sessions': 0, 'whole_goal_complete': False}
OUT.mkdir(exist_ok=False)
(OUT / 'audit.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
files = {'audit.json': OUT / 'audit.json', 'auditor.py': Path(__file__),
         'selected-assembly.json': selected, 'map-policy-qualified.json': MAP / 'qualified.json',
         'normal-qualified.json': normal_path}
file_pins = {name: sha(path) for name, path in files.items()}
archive = OUT / 'call-context-evidence.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
    for name, path in files.items():
        z.write(path, name)
with zipfile.ZipFile(archive) as z:
    for name, expected in file_pins.items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected
(OUT / 'qualified.json').write_text(json.dumps({key: value for key, value in proof.items() if key != 'sites'} |
    {'archive_sha256': sha(archive), 'file_pins': file_pins, 'archive_readback_verified': True},
    ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps({key: proof[key] for key in ['status', 'far_call_sites', 'image_resource_error_call_sites', 'pixel_lighting_algorithm_resolved', 'game_sessions']}, ensure_ascii=False))
