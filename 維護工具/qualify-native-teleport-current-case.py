"""Bind a fresh GUI case to explicit current normal and default Rust evidence."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
root = Path(__file__).resolve().parents[1]
stage = root / '驗證紀錄/current/native-teleport-wire'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
ap = argparse.ArgumentParser()
ap.add_argument('case', type=Path)
ap.add_argument('--profile', choices=['Debug', 'Release'], required=True)
ap.add_argument('--normal-qualification',type=Path,required=True)
ap.add_argument('--rust-evidence',type=Path,required=True)
ap.add_argument('--cohort',type=Path,required=True)
ap.add_argument('--resource-profile',choices=['lito-legacy','tw13081901-v1'],default='lito-legacy')
ap.add_argument('--extended-item-details',action='store_true')
a = ap.parse_args()
case = a.case.resolve(strict=True)
assert case.is_relative_to(root / '驗證紀錄/current/native-teleport-gui')
r = read(case / 'receipt.json')
launch = read(case / 'server-launch-identity.json')
assert r['resource_profile'] == launch['resource_profile'] == a.resource_profile
assert r['extended_item_details'] is launch['extended_item_details'] is a.extended_item_details
assert launch['exe_sha256'] == r['exe_sha256'] and launch['jar_sha256'] == r['jar_sha256']
assert '-Dlito.resourceProfile='+a.resource_profile in launch['command_line']
assert '-Dlito.extendedItemDetails='+str(a.extended_item_details).lower() in launch['command_line']
if a.resource_profile == 'tw13081901-v1':
    assert a.extended_item_details
    assert r['common_catalog_sha256'] == launch['common_catalog_sha256'] == 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
    assert '-Dlito.commonCatalog=' in launch['command_line']
else:
    assert r['common_catalog_sha256'] is launch['common_catalog_sha256'] is None

normal_path=a.normal_qualification.resolve(strict=True);rust_path=a.rust_evidence.resolve(strict=True);cohort=a.cohort.resolve(strict=True)
for path in [normal_path,rust_path,cohort]:assert path.is_relative_to(root/'驗證紀錄/current')
q=read(normal_path);pins=read(normal_path.parent/'source-inputs.json');rust=read(rust_path)
assert q['features']==[] and q['source_manifest_sha256']==sha(normal_path.parent/'source-inputs.json')
main=q['profiles'][a.profile]
assert main['features']==[] and main['unique_entry'] and main['machine']=='AMD64' and main['optional_header']=='PE32+'
assert rust['profile']==a.profile and rust['features']==[] and rust['integration_targets']==90 and rust['products_retained']==91 and rust['failed']==0
assert rust['current_source_manifest_sha256']==q['source_manifest_sha256'] and rust['normal_mains'][a.profile]==main['sha256']
assert sha(root/'維護工具/qualify-default-native-regression.py')==rust['qualifier_sha256']
run=Path(rust['run']);assert run.is_relative_to(root/'驗證紀錄/current/default-native-regression')
assert sha(run/'receipt.json')==rust['receipt_sha256'];batch=read(run/'receipt.json');retained=read(run/'product-retention.json')
assert batch['status']=='verified_current_default90_integration_targets_and_main_units' and batch['normal_mains_after']==rust['normal_mains']
assert len(rust['per_target_pins'])==91 and set(rust['per_target_pins'])==set(retained['products'])
for name,item in rust['per_target_pins'].items():
 # The immutable Rust qualification already checked and archived every test image.
 # This GUI case executes main only; bind that proof and raw logs without rereading91 binaries.
 assert retained['products'][name]['sha256']==item['product_sha256']
 assert sha(run/'cases'/name/'runtime/stdout.log')==item['runtime_sha256']
 assert sha(run/'cases'/name/'inventory/stdout.log')==item['inventory_sha256']
assert not rust['full_library_unit_regression_verified'] and not rust['full_current_TCP_regression_verified']
scenario = r['scenario']
assert scenario in ['cross-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-late', 'cancel-expiry']
cross = scenario == 'cross-endpoint'
assert r['status'] == ('verified_native_teleport_GUI_cross_endpoint_shared_battle' if cross
                       else 'verified_native_teleport_GUI_' + scenario + '_and_fresh_reentry')
assert r['inputs_unchanged'] and r['reference_is_current'] and r['complete_build_inputs_verified']
assert r['exe_sha256'] == main['sha256'] == sha(Path(main['path']))
assert sha(case / 'compilation-inputs.json') == r['compilation_inputs_sha256']
compilation = read(case / 'compilation-inputs.json')
assert compilation['files'] == pins and compilation['profile'] == a.profile
assert compilation['main_sha256'] == r['exe_sha256'] and len(pins) == q['source_files']
for name, expected in pins.items():
    assert sha(root / name) == expected, name
for name, expected in r['source_pins'].items():
    assert sha(root / name) == expected
receipt_sha = sha(case / 'receipt.json')
authority_path = case / ('teleport-gui-authority-comparison.json' if cross else 'teleport-recovery-gui-comparison.json')
authority_validator = 'verify-native-teleport-gui-authority.py' if cross else 'verify-native-teleport-recovery-gui.py'
authority = read(authority_path)
glyph = read(case / 'teleport-glyph-comparison.json')
scenes = read(case / 'world-scene-pixel-comparison.json')
assert authority['receipt_sha256'] == glyph['receipt_sha256'] == scenes['receipt_sha256'] == receipt_sha
sessions = 6 if cross else 10 if scenario == 'wrong-service' else 8
assert authority['actual_client_sessions'] == sessions and glyph['matched_glyphs'] == 4 and scenes['images'] == (4 if cross else 6)
inventory_continuity = None
if a.resource_profile == 'tw13081901-v1' and cross:
    inventory_continuity = read(case / 'TW-inventory-continuity-comparison.json')
    assert inventory_continuity['receipt_sha256'] == receipt_sha and inventory_continuity['capture_sha256'] == sha(next((Path(r['runtime'])/'captures').glob('*/frames.jsonl')))
    assert inventory_continuity['objects'] == 25 and len(inventory_continuity['checks']) == 2
    assert all(x['source_destination_and_battle_items_identical'] and x['wire_detail_bytes_preserved'] for x in inventory_continuity['checks'])
recovery_inventory = None
if a.resource_profile == 'tw13081901-v1' and not cross:
    recovery_inventory = read(case / 'recovery-inventory-continuity-comparison.json')
    assert recovery_inventory['resource_profile'] == 'tw13081901-v1' and recovery_inventory['scenario'] == scenario
    assert recovery_inventory['receipt_sha256'] == receipt_sha and recovery_inventory['capture_sha256'] == sha(next((Path(r['runtime'])/'captures').glob('*/frames.jsonl')))
    assert recovery_inventory['objects'] == 25 and len(recovery_inventory['checks']) == 2
    assert all(x['fresh_rental_reentry_new_IDs_and_all_nonID_fields_match'] and x['reentry_native_inventory_still_matches_continued_snapshot'] and x['exact_wire_fields_and_extended_details_preserved'] for x in recovery_inventory['checks'])
    assert all(x['expired_old_IDs_never_regranted'] if scenario == 'cancel-expiry' else x['continued_stage_all_six_fields_preserved'] for x in recovery_inventory['checks'])
stale = None
if scenario == 'cancel-retry':
    stale = read(case / 'teleport-stale-command-comparison.json')
    assert r['replay_stale_commands'] and stale['receipt_sha256'] == receipt_sha
    assert stale['status'] == 'verified_64_stale_and_duplicate_GUI_notifications_without_extra_formal_requests'
steps = [s for s in r['steps'] if s['step'] == 'GUI-key-walk']
assert len(steps) == (14 if cross else 16)
headings = {0x68: 0, 0x26: 1, 0x66: 2, 0x27: 3, 0x62: 4, 0x28: 5, 0x64: 6, 0x25: 7}
directions = [(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)]
expected = []
for s in steps:
    heading = headings[s['key']]
    dx, dy = directions[heading]
    assert s['after'] == [s['before'][0] + dx, s['before'][1] + dy]
    expected.append(bytes([0x0a]) + int(s['before'][0]).to_bytes(2, 'little')
                    + int(s['before'][1]).to_bytes(2, 'little') + bytes([heading]))
cap = next((Path(r['runtime']) / 'captures').glob('*/frames.jsonl'))
frames = [json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()]
actual = [bytes.fromhex(x['plain']) for x in frames
          if x['direction'] == 'client_to_server' and x['plain'].startswith('0A ')]
assert actual == expected
out = {'status': 'qualified_default_feature_wire_fix_real_cross_endpoint_GUI' if cross else 'qualified_default_feature_wire_fix_real_recovery_GUI', 'profile': a.profile,
       'TW_full_cross_endpoint_inventory_continuity': bool(inventory_continuity),
       'TW_inventory_continuity_comparison_sha256': sha(case/'TW-inventory-continuity-comparison.json') if inventory_continuity else None,
       'TW_recovery_complete_inventory_continuity': bool(recovery_inventory),
       'TW_recovery_inventory_comparison_sha256': sha(case/'recovery-inventory-continuity-comparison.json') if recovery_inventory else None,
       'scenario': scenario, 'resource_profile':a.resource_profile, 'extended_item_details':a.extended_item_details,
       'common_catalog_sha256':r['common_catalog_sha256'], 'server_launch_identity_sha256':sha(case/'server-launch-identity.json'),
       'case': str(case), 'main_sha256': r['exe_sha256'], 'source_files': len(pins), 'source_inputs_unchanged': True,
       'receipt_sha256': receipt_sha, 'capture_sha256': sha(cap), 'reference_sha256': r['jar_sha256'],
       'normal_build_qualification_sha256':sha(normal_path),'default_Rust_evidence_sha256':sha(rust_path),'retained_Rust_results_after_Java_only_delta':bool(rust['Java_only_changes_outside_Rust_dependencies']),
       'actual_formal_sessions': sessions, 'exact_C_MOVE': len(steps), 'extra_C_MOVE': 0,
       'original_world_PNGs': scenes['images'], 'original_glyph_matches': 4,
       'stale_GUI_notifications': 64 if stale else 0,
       'stale_comparison_sha256': sha(case / 'teleport-stale-command-comparison.json') if stale else None,
       'authority_comparison_sha256': sha(authority_path),
       'glyph_comparison_sha256': sha(case / 'teleport-glyph-comparison.json'),
       'scene_comparison_sha256': sha(case / 'world-scene-pixel-comparison.json'),
       'synthetic_faults_injected': False, 'physical_input_verified': False,
       'full_library_regression_verified': False, 'whole_goal_complete': False}
(case / 'current-default-source-qualification.json').write_text(json.dumps(out, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
suffix = 'cross' if cross else 'recovery-' + scenario
(cohort / f'{a.profile}-real-{suffix}-qualified.json').write_text(json.dumps(out, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
dest = case / 'validation-sources'
dest.mkdir(exist_ok=True)
for name in [Path(__file__).name, authority_validator,
             'verify-teleport-glyph-pixels.py', 'verify-current-hud-teleport-scenes.py'] + (['verify-native-teleport-stale-commands.py'] if stale else []) + (['verify-TW-teleport-inventory-continuity.py'] if inventory_continuity else []) + (['verify-teleport-recovery-inventory-continuity.py'] if recovery_inventory else []):
    p = root / '維護工具' / name
    shutil.copy2(p, dest / (sha(p)[:16] + '-' + p.name))
print(json.dumps({k: out[k] for k in ['status','profile','actual_formal_sessions','exact_C_MOVE']}, ensure_ascii=False))
