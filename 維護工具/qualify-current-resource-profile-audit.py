"""Bind the two actual C02 codecs and the existing TW metadata, without a gameplay claim."""
from pathlib import Path
import hashlib
import json
import re
import struct
import zipfile

ROOT = Path(__file__).resolve().parents[1]
STAGE = ROOT / '驗證紀錄/current/native-effect-native-clock/common-finite-followup/protocol-effect-id-audit'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()


assert not (STAGE / 'qualified.json').exists()
reference = read(ROOT / '驗證紀錄/current/reference-snapshot.json')
assert sha(reference['snapshot']) == sha(reference['source']) == reference['sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
expected = [(1, 11722, 744, 4, 10), (19, 11723, 829, 15, 100),
            (35, 11724, 830, 20, 200), (57, 11725, 831, 48, 500)]
profiles = {}
for profile in ['lito-legacy', 'tw13081901-v1']:
    observation = read(STAGE / (profile + '-codec.json'))
    assert observation['jar_sha256'].upper() == reference['sha256']
    assert observation['resource_profile'] == profile and observation['actual_game_sessions'] == 0
    assert observation['gameplay_verified'] is False and len(observation['spells']) == 4
    for row, (skill, catalog, legacy, cost, reuse) in zip(observation['spells'], expected):
        wire = legacy if profile == 'lito-legacy' else catalog
        assert (row['skill'], row['catalog_gfx'], row['wire_gfx'], row['action'],
                row['mp_cost'], row['reuse_ms'], row['maximum_range']) == (skill, catalog, wire, 19, cost, reuse, -1)
        assert row['request'] == (bytes([0x14, (skill - 1) // 8, (skill - 1) % 8]) + struct.pack('<I', 2)).hex()
        assert row['effect'] == (bytes([0x37]) + struct.pack('<IH', 2, wire)).hex()
    profiles[profile] = observation['spells']

adapter = (STAGE / 'C02-SharedClientResources.javap.txt').read_text(encoding='utf-8-sig')
protocol = (STAGE / 'C02-Protocol.javap.txt').read_text(encoding='utf-8-sig')
assert 'String lito.resourceProfile' in adapter and 'String lito-legacy' in adapter
assert 'String tw13081901-v1' in adapter and 'String lito.commonCatalog' in adapter
assert 'tableswitch   { // 11722 to 11725' in protocol
assert 'SharedClientResources.effect:(II)I' in protocol

tw = read(STAGE / 'TW-selected-definitions.json')
assert tw['content_binding_verified'] and tw['binding_is_not_a_trusted_signature']
assert sha(tw['pack']) == tw['pack_sha256']
assert tw['actual_game_sessions'] == 0 and not tw['runtime_presentation_verified']
launcher = ROOT.parent / 'lito登入器/LiTo381-Launcher'
manifest_path = launcher / 'app/common_catalog_manifest.json'
catalog_path = launcher / 'app/common_catalog.bin'
manifest = read(manifest_path)
assert tw['decoded_sha256'] == manifest['source_sha256'].upper() == '7EBE2B464C8B1EA9C58AC985DD8FF4C823784D8231894262EDDCD7691CA94008'
assert sha(catalog_path) == manifest['payload_sha256'].upper() == 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert manifest['shared_count'] == 30000 and manifest['lito_count'] == 285
selected = {row['graphics']: row for row in tw['definitions']}
assert set(selected) == {744, 829, 830, 831, 11722, 11723, 11724, 11725, 11726}
for gfx, sound in [(11722, 109), (11723, 114), (11724, 115), (11725, 126)]:
    definition = selected[gfx]['definition']
    assert f'[{sound}' in definition and '104.attr(8)' in definition
assert '105.clothes(1 11726)' in selected[11725]['definition']

startup = launcher / 'app/server/Start-Shared-Formal.py'
startup_text = startup.read_text(encoding='utf-8-sig')
flag_lines = [dict(line=i + 1, text=line.strip()) for i, line in enumerate(startup_text.splitlines())
              if 'lito.resourceProfile=tw13081901-v1' in line]
assert len(flag_lines) == 1 and 'lito.commonCatalog=' in flag_lines[0]['text']
native = ROOT / 'LiTo381-Modern/src/modern_lito'
effects = (native / 'native_skill_effects.rs').read_text(encoding='utf-8')
shared = (native / 'shared_action_catalog.rs').read_text(encoding='utf-8')
assert 'Self::Skills => &[221, 744, 745, 754, 761, 871, 2165]' in effects
assert 'SharedArchiveKind::Text' in shared and 'SharedTwActionCatalog' not in shared

sprite_inputs = read(STAGE / 'sprite-entry-audit.json')
sprite_pins = []
for source in sprite_inputs['sources']:
    root = Path(source['root'])
    for entry in source['selected_entries']:
        idx = root / entry['archive']
        index = idx.read_bytes()
        count = struct.unpack_from('<I', index)[0]
        assert len(index) == 4 + 28 * count
        matches = []
        for i in range(count):
            offset, name, size = struct.unpack_from('<I20sI', index, 4 + i * 28)
            if name.split(b'\0')[0].decode('ascii', errors='replace').lower() == entry['name'].lower():
                matches.append((offset, size))
        assert matches == [(entry['offset'], entry['bytes'])]
        pak = idx.with_suffix('.pak')
        assert entry['offset'] + entry['bytes'] <= pak.stat().st_size
        with pak.open('rb') as f:
            f.seek(entry['offset'])
            payload = f.read(entry['bytes'])
        assert hashlib.sha256(payload).hexdigest().upper() == entry['sha256']
        sprite_pins.append(dict(root=str(root), index=idx.name, index_sha256=sha(idx), **entry))

inputs = {str(startup): sha(startup), str(manifest_path): sha(manifest_path), str(catalog_path): sha(catalog_path), tw['pack']: sha(tw['pack'])}
native_pins = {name: sha(native / name) for name in ['native_skill_effects.rs', 'shared_action_catalog.rs', 'tw_action_catalog.rs', 'resources.rs']}
files = {p.name: p for p in STAGE.iterdir() if p.is_file() and p.suffix in ['.json', '.txt', '.java', '.log']}
files['CurrentHealingContractOracle.java'] = ROOT / '維護工具/oracles/CurrentHealingContractOracle.java'
files['CurrentHealingContractOracle.class'] = ROOT / 'LiTo381-Modern/target/current/protocol-effect-id-audit/oracle/local/lito381/server/CurrentHealingContractOracle.class'
files['qualifier.py'] = Path(__file__)
for name in native_pins:
    files['native-source/' + name] = native / name
archive = STAGE / 'raw-audit-evidence.zip'
assert not archive.exists()
file_pins = {name: sha(path) for name, path in files.items()}
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, path in files.items():
        z.write(path, name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist()) == set(file_pins)
    for name, expected_sha in file_pins.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == expected_sha
proof = dict(status='qualified_two_C02_resource_profile_codec_branches_and_TW_input_provenance_only',
             jar_sha256=reference['sha256'], profiles=profiles, external_read_only_input_pins=inputs,
             launcher_source_profile_flag=flag_lines, native_source_pins=native_pins,
             current_native_TW_scene_integration_verified=False,
             existing_TW_reader_available=True, existing_encoded_SPR_inputs=sprite_pins,
             TW_pack_matches_current_server_common_catalog_source=True,
             full_heal_clothes_dependency=11726,
             original_assets_missing=False, actual_game_sessions=0,
             native_sprite_decode_or_render_verified=False, full_current_formal_service_profile_verified=False,
             existing_legacy_receipts_relabelled=False, archive_sha256=sha(archive),
             archive_readback_verified=True, raw_file_pins=file_pins, whole_goal_complete=False)
(STAGE / 'qualified.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(dict(status=proof['status'], profiles=2, codec_cases=8, actual_game_sessions=0, TW_source_matched=True)))
