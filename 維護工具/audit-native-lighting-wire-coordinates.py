"""Retain static S_PUT_OBJECT coordinate flow; no original handler execution."""
from pathlib import Path
import hashlib
import json
import zipfile

ROOT = Path(__file__).resolve().parents[1]
ASM = Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm')
OUT = ROOT / '驗證紀錄/current/native-lighting-wire-coordinates'

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

assert sha(ASM) == '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
assert not OUT.exists(), 'Preserve the original audit result'
OUT.mkdir()
lines = ASM.read_bytes().decode('utf-8').replace('\r', '').split('\n')
selected = {}
for name in ['sub_4A2A50', 'sub_4A3990', 'sub_4E7A40', 'sub_4B8FC0', 'sub_485A40']:
    start = next(i for i, s in enumerate(lines) if s.startswith(name + ' proc'))
    end = next(i for i in range(start, len(lines)) if lines[i].startswith(name + ' endp'))
    selected[name] = {'first_newline_line': start + 1, 'last_newline_line': end + 1,
                      'assembly': '\n'.join(lines[start:end + 1])}

parser = selected['sub_4A2A50']['assembly']
assert 'case 104' in parser and 'mov     [eax], cx' in parser
handler = selected['sub_4A3990']['assembly']
assert '"hhdhccccdhsscdssccccshc"' in handler
for text in ['mov     edx, [ebp+var_34]', 'mov     ebx, [ebp+var_30]',
             'and     edx, 0FFFFh', 'and     ebx, 0FFFFh',
             'lea     edi, [edx+edx-8000h]',
             'push    ebx\nand     edx, 0FFFFh\npush    edi\ncall    sub_4E7A40']:
    assert text in handler, text
constructor = selected['sub_4E7A40']['assembly']
for text in ['mov     ecx, [ebp+arg_0]', 'mov     edx, [ebp+arg_4]',
             'mov     [esi+2Ch], ecx', 'mov     [esi+30h], edx']:
    assert text in constructor, text

# Pure algebra follows the retained original projection, for even camera U
# obtained by applying the same wire conversion. No native handler is executed.
cases = []
for camera in [(0, 0), (32768, 32768), (65535, 65535), (32123, 32456)]:
    for x in [0, 1, 32767, 32768, 65534, 65535, camera[0]]:
        for y in [0, 1, 32767, 32768, 65534, 65535, camera[1]]:
            u, v = 2 * x - 0x8000, y
            cu, cv = 2 * camera[0] - 0x8000, camera[1]
            du, dv = u - cu, v - cv
            px = 12 * (((du + 1) & ~1) + 2 * dv - 2)
            py = 6 * (2 * dv - (du & ~1))
            dx, dy = x - camera[0], y - camera[1]
            assert (px + 24, py + 6) == (24 * (dx + dy), 12 * (dy - dx) + 6)
            cases.append({'wire': [x, y], 'camera_wire': list(camera), 'native': [u, v],
                          'producer_center': [px + 24, py + 6]})

result = {'status': 'static_coordinate_flow_and_conditional_projection_algebra_retained',
          'original_assembly_sha256': sha(ASM),
          'wire_to_native_x': '2 * uint16(wire_x) - 0x8000',
          'wire_to_native_y': 'uint16(wire_y)',
          'native_coordinate_fields': ['actor+0x2C', 'actor+0x30'],
          'scope': 'S_PUT_OBJECT sub_4A3990 call path to sub_4E7A40 coordinate stores',
          'projection_algebra_cases': len(cases),
          'projection_condition': 'camera native U and V use the same wire-coordinate conversion',
          'conditional_producer_center': ['24*(wire_x-camera_x+wire_y-camera_y)',
                                          '12*(wire_y-camera_y-wire_x+camera_x)+6'],
          'camera_initialization_mapping_verified': False,
          'S_MOVE_coordinate_lifecycle_verified': False,
          'walking_offset_phase_and_coordinate_base_verified': False,
          'actor_flag_mapping_verified': False,
          'native_packet_handler_executed': False,
          'actual_game_sessions': 0, 'product_source_or_tests_changed': False,
          'full_world_framebuffer_verified': False, 'whole_goal_complete': False}
for filename, data in [('selected-assembly.json', selected), ('algebra-vectors.json', cases),
                       ('static-findings.json', result)]:
    (OUT / filename).write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
(OUT / 'audit.py').write_bytes(Path(__file__).read_bytes())
pins = {p.name: sha(p) for p in OUT.iterdir() if p.is_file()}
archive = OUT / 'static-evidence.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as packed:
    for name in pins:
        packed.write(OUT / name, name)
with zipfile.ZipFile(archive) as packed:
    for name, digest in pins.items():
        assert hashlib.sha256(packed.read(name)).hexdigest().upper() == digest
retention = {'files': pins, 'archive_sha256': sha(archive), 'readback_verified': True}
(OUT / 'retention.json').write_text(json.dumps(retention, indent=2) + '\n', encoding='utf-8')
print(json.dumps({**result, 'archive_sha256': retention['archive_sha256']}, ensure_ascii=False))
