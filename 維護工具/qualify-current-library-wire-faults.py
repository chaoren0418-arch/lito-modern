"""Bind synthetic wire reports emitted by the current full default lib run."""
from pathlib import Path
from datetime import datetime
import argparse,hashlib,json,shutil,zipfile,re
root=Path(__file__).resolve().parents[1]
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
p=argparse.ArgumentParser();p.add_argument('build_run',type=Path);a=p.parse_args()
run=a.build_run.resolve(strict=True);assert run.is_relative_to(root/'驗證紀錄/current')
q=read(run/'default-library-qualified.json');result=read(run/'runtime-full-current/receipt.json')
assert q['default_full_library_selection_verified'] and q['passed']==1929 and q['failed']==0 and result['status']=='passed_pinned_library_tests'
pins=read(run/'source-inputs.json');assert len(pins)==968
for name,h in pins.items():assert sha(root/name)==h,name
log=(run/'runtime-full-current/tests.log').read_text(encoding='utf-8-sig')
expected=['coalesced_old_socket_burst_cannot_retarget_or_repeat_quit','retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier','malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner','new_destination_does_not_accept_previous_endpoint_as_a_new_handoff','cancel_discards_old_inbound_burst_before_fresh_login','invalid_transport_envelopes_during_teleport_retire_scope']
for name in expected:assert re.search(r'^test modern_lito::native_login::teleport_wire_tests::'+re.escape(name)+r' \.\.\. ok$',log,re.M),name
fixture_path=root/'LiTo381-Modern/tests/fixtures/native-teleport-controller-current.json';fixture=read(fixture_path)
assert sha(Path(fixture['capture_path']))==fixture['capture_sha256'] and sha(Path(fixture['source_receipt']))==fixture['source_receipt_sha256']
assert fixture['reference_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
started=datetime.fromisoformat(result['started_utc'].replace('Z','+00:00')).timestamp();finished=datetime.fromisoformat(result['finished_utc'].replace('Z','+00:00')).timestamp()
stage=run/'wire-fault-qualified';assert not stage.exists();stage.mkdir()
names = {f'status-truncated-{n}' for n in range(14)} | {f'endpoint-truncated-{n}' for n in range(8)}
names |= {'status-trailing', 'status-unknown-word', 'status-duplicate', 'endpoint-before-status',
          'endpoint-wrong-slot', 'endpoint-invalid-index', 'endpoint-zero-port', 'endpoint-trailing',
          'frame-zero', 'frame-one', 'frame-partial', 'retired-duplicate-burst',
          'coalesced-old-socket-burst', 'new-epoch-replayed-endpoint', 'cancel-old-burst-fresh-login'}
verification = root / 'LiTo381-Modern/target/current/verification/native-teleport-wire/Debug'
files = {p.stem: p for p in verification.glob('*.json')}
assert set(files) == names and len(names) == 37
evidence = stage / 'reports'
evidence.mkdir(exist_ok=True)
reports = []
connections = 0
for name in sorted(names):
    p = files[name]
    assert started <= p.stat().st_mtime <= finished, (p.name, 'not emitted during this run')
    d = read(p)
    assert d['case'] == name and d['quiescent']
    assert d['synthetic_encrypted_loopback'] and d['authenticated_formal_sessions'] == 0
    assert d['fixture_sha256'] == sha(fixture_path)
    state = d['snapshot']
    assert not state['world']['actors'] and state['world']['map_id'] is None
    assert state['inventory']['owner'] is None and not state['inventory']['items']
    assert state['merchant']['owner'] is None and state['joined_room'] is None
    assert state['selected_character'] is None and state['services'] is None
    assert all(n == 1 for n in d['peer_connection_counts'])
    commands = d['client_command_opcodes']
    assert commands[0].count(0x27) == 1
    if name in {'retired-duplicate-burst', 'coalesced-old-socket-burst', 'new-epoch-replayed-endpoint'}:
        assert d['epoch'] == 2 and len(commands) == 2 and commands[0].count(0xd9) == 1
        assert 0xd9 not in commands[1]
        if name == 'coalesced-old-socket-burst':
            assert commands[1].count(1) == 1 and d['extra']['emitted_bodies'] == 8
    elif name == 'cancel-old-burst-fresh-login':
        assert d['epoch'] == 2 and commands[1] == [] and d['extra']['fresh_samples'] == 24
        assert state['redirect'] is None
    else:
        assert d['phase'] == 'failed' and d['error'] and d['epoch'] == 1
        assert state['redirect'] is None and 0xd9 not in commands[0]
    assert 'SyntheticOnly' not in p.read_text(encoding='utf-8')
    connections += sum(d['peer_connection_counts'])
    shutil.copy2(p, evidence / p.name)
    reports.append({'case': name, 'sha256': sha(p), 'epoch': d['epoch'], 'phase': d['phase']})
assert connections == 41

files={('reports/'+p.name):p for p in evidence.glob('*.json')}
for name in ['default-library-qualified.json','source-inputs.json','runtime-full-current/receipt.json','runtime-full-current/tests.log']:files['run/'+name]=run/name
for name in ['src/modern_lito/native_login.rs','src/modern_lito/native_teleport_controller_tests.rs','tests/fixtures/native-teleport-controller-current.json']:files['source/'+name]=root/'LiTo381-Modern'/name
files['qualifier.py']=Path(__file__)
archive=stage/'raw-wire-evidence.zip'
file_pins={name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
 for name,p in files.items():z.write(p,name)
with zipfile.ZipFile(archive) as z:
 assert set(z.namelist())==set(file_pins)
 for name,h in file_pins.items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==h
proof={'status':'qualified_current_default_library_synthetic_wire_fault_subset','profile':'Debug/test debug0','source_files':968,'source_manifest_sha256':q['source_manifest_sha256'],'product_sha256':q['product_sha256'],'feature_enabled':False,'compiled_with_cfg_test':True,'tests_passed':6,'test_names':expected,'fault_scenarios':37,'synthetic_TCP_connections':connections,'authenticated_formal_sessions':0,'reports':reports,'raw_file_pins':file_pins,'raw_archive_sha256':sha(archive),'archive_readback_verified':True,'normal_mains_unchanged':q['normal_mains_unchanged'],'Release_rerun':False,'formal_GUI_verified_by_this_subset':False,'whole_goal_complete':False}
(stage/'qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:proof[k] for k in ['status','tests_passed','fault_scenarios','synthetic_TCP_connections','authenticated_formal_sessions']},ensure_ascii=False))
