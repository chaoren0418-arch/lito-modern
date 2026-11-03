"""Independent wire, scene authority and deferred finite-audio reader."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-class2-same-map')
r=read(run/'receipt.json');assert r['status']=='executed_class2_WORLD_retirement' and r['all_started_processes_terminal']
assert r['actual_sessions']==6 and r['asset_metadata_unchanged'] and r['actual_world_tree_transitions']==4
runtime=Path(r['runtime']);build=Path(r['compile']);b=read(build/'receipt.json')
assert b['status']=='compiled_current_class2_candidate_diagnostic' and sha(build/'receipt.json')==r['compile_receipt_sha256']
assert sha(r['product'])==r['product_sha256']==b['product_sha256']
for name,pin in b['candidate_inputs'].items():
    assert sha(pin['path'])==pin['sha256']
    assert sha(build/name)==b['snapshot_candidate_inputs'][name]
for name,digest in read(build/'compiled-inputs.json').items():assert sha(ROOT/name)==digest
assert sha(r['reference_jar'])==r['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(r['common_catalog'])==r['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert sha(r['fixture_qualification'])==r['fixture_qualification_sha256']
for path,digest in r['harness_class_pins'].items():assert sha(path)==digest
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
server=read(runtime/'result.json');assert server['actual_client_sessions']==6 and server['game_sessions']==3
assert server['memberships_empty'] and server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
config=(runtime/'server.properties').read_text(encoding='utf-8-sig')
for line in ['database.mode=memory','database.read.only=true','state.persistence.enabled=false','classic.world.compatibility.enabled=true','battle.ai.enabled=false']:assert line+'\n' in config
fixture=read(runtime/'initial-disintegrate-fixture.json');assert fixture['before_all_sessions'] and fixture['runtime_mutations']==0 and fixture['natural_special_critical']
frames=[]
for p in (runtime/'captures').rglob('frames.jsonl'):
    frames.extend(json.loads(line) for line in p.read_text(encoding='utf-8-sig').splitlines())
requests=[bytes.fromhex(f['plain']) for f in frames if f['direction']=='client_to_server' and f['opcode']=='0x14']
queued=read(runtime/'b-cast-queued.json');cast=bytes.fromhex(queued['request_hex'])
assert len(cast)==11 and cast[:7]==bytes.fromhex('14040101000000')
target=queued['before']['world']['actors']['1'];assert cast[7:]==int(target['x']).to_bytes(2,'little')+int(target['y']).to_bytes(2,'little')
assert queued['before']['world']['hp']==queued['immediate']['world']['hp'] and queued['before']['world']['mp']==queued['immediate']['world']['mp'] and queued['before']['inventory']==queued['immediate']['inventory']
assert sorted(requests)==sorted([cast]+[bytes.fromhex('141002')]*4+[bytes.fromhex('140000')+own.to_bytes(4,'little') for own in [1,3]]*3)
attacks=[bytes.fromhex(f['plain']) for f in frames if f['direction']=='server_to_client' and f['opcode']=='0x23']
attacks=[wire for wire in attacks if len(wire)==29 and wire[1]==18 and int.from_bytes(wire[2:6],'little')==3 and int.from_bytes(wire[6:10],'little')==1]
assert len(attacks)==3 and len(set(attacks))==1 and int.from_bytes(attacks[0][16:18],'little')==11736
assert any(f['direction']=='server_to_client' and f['opcode']=='0x4D' and bytes.fromhex(f['plain'])==bytes([0x4d])+(4982).to_bytes(2,'little')+(5000).to_bytes(2,'little') for f in frames)
portals=[bytes.fromhex(f['plain']) for f in frames if f['direction']=='server_to_client' and f['opcode']=='0x5A']
expected=bytes([0x5a])+int(4).to_bytes(2,'little')+int(33051).to_bytes(2,'little')+int(32337).to_bytes(2,'little')
assert portals==[expected]*4
assert sum(f['direction']=='client_to_server' and f['plain']=='08' for f in frames)==4
assert sum(f['plain']=='MASKED_C_LOGIN' for f in frames)==6
assert not any(f.get('plain')=='21e0c40000' for f in frames),'controlled local S21 must not be represented as actual C02 traffic'
for role in ['a','b','c']:
    preflight=read(runtime/(role+'-motion-hurt-preflight.json'))
    assert preflight['actual_network_sessions']==0 and preflight['hurt_waited_for_motion_boundary'] and preflight['camera_history_rewind_guard_preserved']
    cases=preflight['eight_directions'];assert len(cases)==8 and {case['heading'] for case in cases}==set(range(8))
    for case in cases:
        assert case['new_incarnation_retired_hurt']
        rows=case['samples'];assert len(rows)==8
        assert all(row['phase']=='walk' for row in rows if row['now']<2_000_000_000)
        assert rows[-2]['phase']=='damage' and rows[-1]['phase']=='idle' and rows[-2]['position']==rows[-1]['position']
        origin=rows[0]['position'];distances=[sum(abs(row['position'][i]-origin[i]) for i in [0,1]) for row in rows]
        assert all(a<=b for a,b in zip(distances,distances[1:]))
server_trace=[json.loads(line)['state'] for line in (runtime/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
roles={};pictures={}
for role,own in [('a',1),('b',3)]:
    j=read(runtime/(role+'-result.json'));before=j['before'];after=j['after'];walk=j['after_walk']
    assert j['old_epoch']==j['new_epoch'] and after['world']['generation']==before['world']['generation']+1
    assert before['world']['map_id']==564 and after['world']['map_id']==4 and after['world']['completed_portals']==1
    assert before['selected_character']==after['selected_character'] and before['inventory']['owner']==after['inventory']['owner'] and before['inventory']['items']==after['inventory']['items']
    assert before['world']['hp']==j['immediate_tree_queue']['world']['hp'] and before['world']['mp']==j['immediate_tree_queue']['world']['mp'] and before['inventory']==j['immediate_tree_queue']['inventory']
    assert after['joined_room'] is None and after['channel'] is None and after['rooms']==[]
    assert after['environment']['ambient_override_counter']==0
    assert after['environment']['ambient_level']==after['environment']['daylight']['level'] and after['environment']['map']==4
    assert any(s['object_id']==own and s['phase']=='WORLD' and s['classic_world_map']==4 and s['hp']==after['world']['hp'][0] and s['mp']==after['world']['mp'][0] and s['test_account']+'('+s['source']+')'==after['selected_character'] for s in server_trace),'WORLD vitals/character lack actual server trace match'
    assert j['old_surface_rejected'] and j['old_cue_deliveries']==0 and j['source34_sink_calls']==0
    assert len(j['held_cue_outcomes'])==1 and j['held_cue_outcomes'][0]['first'] in ['Stale','AlreadyConsumed'] and j['held_cue_outcomes'][0]['duplicate']=='AlreadyConsumed'
    assert j['tree_request_hex']=='141002' and 131 in before['world']['learned_skills']
    if role=='a' and r['maxcasts']!=3:assert j['source_active_effects']>0
    motion_verified=False
    if role=='a' and r['maxcasts']==3:
        motion=j['target_motion'];assert motion['verified'] and motion['uses_normal_local_step_prediction']
        assert motion['actor_origin']!=motion['actor_destination']
        launch=motion['launch'];observed=motion['after_move'];state=motion['after_state'];frame=motion['render']['receipt']
        assert launch['cell']==observed['cell']==motion['actor_origin']
        assert launch['sequence']==observed['sequence'] and launch['graphics']==observed['graphics']==11736
        assert launch['target']==observed['target']==1
        actor=state['world']['actors']['1'];assert [actor['x'],actor['y']]==motion['actor_destination']
        peer=motion['server_step_observed_by_peer'];assert peer['observer_object_id']==3 and peer['network_observation'] and not peer['local_prediction_for_target']
        assert peer['generation']==state['world']['generation'] and peer['actor']['id']==1 and not peer['actor']['position_predicted']
        assert [peer['actor']['x'],peer['actor']['y']]==motion['actor_destination'] and peer['actor']['motion_from']==motion['actor_origin']
        assert frame['visual_revision']==state['world']['visual_revision'] and frame['world_generation']==state['world']['generation']
        expected_move=bytes([0x12])+(1).to_bytes(4,'little')+int(motion['actor_origin'][0]).to_bytes(2,'little')+int(motion['actor_origin'][1]).to_bytes(2,'little')+bytes([motion['heading']])
        assert any(f['direction']=='server_to_client' and f['opcode']=='0x12' and bytes.fromhex(f['plain'])==expected_move for f in frames)
        assert any(s['object_id']==1 and s['phase']=='BATTLE' and [s['x'],s['y']]==motion['actor_destination'] for s in server_trace)
        raw=motion['render']['png'];p=Path(raw[4:] if raw.startswith('\\\\?\\') else raw)
        with Image.open(p) as im:
            im.load();assert im.size==(800,600) and im.mode=='RGBA';pixel=hashlib.sha256(im.tobytes()).hexdigest().upper()
        pictures[str(p)]=dict(sha256=sha(p),rgba_sha256=pixel)
        motion_verified=True
    ownrow=walk['world']['actors'][str(own)];assert [ownrow['x'],ownrow['y']]==j['walk_target'] and walk['world']['local_corrections']==after['world']['local_corrections']
    dst=j['destination_audio'];assert not dst['deferred'] and dst['own_deliveries_after']==dst['own_deliveries_before']+1
    assert dst['request_hex']==(bytes.fromhex('140000')+own.to_bytes(4,'little')).hex()
    assert dst['before']['world']['hp']==dst['immediately_after_queue']['world']['hp'] and dst['before']['world']['mp']==dst['immediately_after_queue']['world']['mp'] and dst['before']['inventory']==dst['immediately_after_queue']['inventory']
    assert dst['after_ack']['world']['own_cast_packets']==dst['before']['world']['own_cast_packets']+1
    delivery=dst['own_delivery_events'][-1];assert delivery['epoch']==j['new_epoch'] and delivery['generation']==after['world']['generation'] and delivery['row']['event']['sound']==109 and delivery['outcome'].startswith('Played(Started')
    for label in ['source','destination']:
        raw=j[label+'_picture']['png'];p=Path(raw[4:] if raw.startswith('\\\\?\\') else raw)
        with Image.open(p) as im:
            im.load();assert im.size==(800,600) and im.mode=='RGBA';pixel=hashlib.sha256(im.tobytes()).hexdigest().upper()
        pictures[str(p)]=dict(sha256=sha(p),rgba_sha256=pixel)
    receipt=j['destination_picture']['receipt'];assert receipt['call_lightning_effects']['active']==0 and receipt['call_lightning_effects']['hurt_callbacks']==[]
    shutdown=read(runtime/(role+'-shutdown.json'));assert shutdown['error'] is None and shutdown['close_error'] is None and shutdown['client_quiescent'] and shutdown['view_quiescent']
    same=j['same_map'];sb=same['before'];sa=same['after'];sw=same['after_walk']
    assert sb['world']['map_id']==sa['world']['map_id']==4 and sa['world']['generation']==sb['world']['generation']+1
    assert same['old_epoch']==same['new_epoch']==j['new_epoch']
    assert sa['world']['completed_portals']==sb['world']['completed_portals']+1
    assert sb['selected_character']==sa['selected_character'] and sb['inventory']['owner']==sa['inventory']['owner'] and sb['inventory']['items']==sa['inventory']['items']
    assert same['old_surface_rejected'] and same['old_cue_deliveries']==0 and same['held_cue_outcomes']==[dict(first='Stale',duplicate='AlreadyConsumed')]
    assert sa['environment']['ambient_override_counter']==0 and sa['environment']['ambient_level']==sa['environment']['daylight']['level']
    assert sb['world']['hp']==same['immediate_tree_queue']['world']['hp'] and sb['world']['mp']==same['immediate_tree_queue']['world']['mp'] and sb['inventory']==same['immediate_tree_queue']['inventory']
    assert same['tree_request_hex']=='141002' and not same['actual_active_class2_flash_on_same_map']
    source=same['source_audio'];destination=same['destination_audio']
    assert source['deferred'] and not destination['deferred']
    assert source['own_deliveries_after']==source['own_deliveries_before']
    held=source['held']['row']['event'];assert held['actor']==own and held['sound']==109 and held['action']==0
    assert destination['own_deliveries_after']==destination['own_deliveries_before']+1
    for audio in [source,destination]:
        assert audio['request_hex']==(bytes.fromhex('140000')+own.to_bytes(4,'little')).hex()
        assert audio['before']['world']['hp']==audio['immediately_after_queue']['world']['hp'] and audio['before']['world']['mp']==audio['immediately_after_queue']['world']['mp'] and audio['before']['inventory']==audio['immediately_after_queue']['inventory']
        assert audio['after_ack']['world']['own_cast_packets']==audio['before']['world']['own_cast_packets']+1
    fresh=destination['own_delivery_events'][-1];assert fresh['generation']==sa['world']['generation'] and fresh['epoch']==same['new_epoch'] and fresh['row']['event']['sound']==109
    assert any(st['object_id']==own and st['phase']=='WORLD' and st['classic_world_map']==4 and st['hp']==sa['world']['hp'][0] and st['mp']==sa['world']['mp'][0] for st in server_trace)
    actor=sw['world']['actors'][str(own)];assert [actor['x'],actor['y']]==same['walk_target'] and sw['world']['local_corrections']==sa['world']['local_corrections']
    for label in ['source','destination']:
        raw=same[label+'_picture']['png'];p=Path(raw[4:] if raw.startswith('\\\\?\\') else raw)
        with Image.open(p) as im:
            im.load();assert im.size==(800,600) and im.mode=='RGBA';pixel=hashlib.sha256(im.tobytes()).hexdigest().upper()
        pictures[str(p)]=dict(sha256=sha(p),rgba_sha256=pixel)
    roles[role]=dict(source_active_effects=j['source_active_effects'],source_ambient_counter=j['source_ambient_counter'],source_generation=before['world']['generation'],destination_generation=after['world']['generation'],old_input_rejected=True,old34_sound_rejected=True,fresh109_sound_delivered_once=True,target_motion_placement_verified=motion_verified)
third=read(runtime/'c-result.json');assert third['natural_battle_result'] and third['no_battle_rule_disabled'] and third['frame_retired'] and third['final']['phase']=='battle_result'
assert third['final']['environment']['ambient_override_counter']==0
third_close=read(runtime/'c-shutdown.json');assert third_close['error'] is None and third_close['close_error'] is None and third_close['client_quiescent'] and third_close['view_quiescent']
jobs=[read(run/'original-runner-process.json')]+r['process_history']
assert all(j.get('exit_code',0)==0 and not j.get('cleanup_stopped_original_owned_handle') for j in jobs)
ids=','.join(str(j['pid']) for j in jobs)
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for job in jobs:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
files=[(p,'run/'+p.relative_to(run).as_posix()) for p in run.rglob('*') if p.is_file() and not p.is_relative_to(out)]
files.extend((p,'runtime/'+p.relative_to(runtime).as_posix()) for p in runtime.rglob('*') if p.is_file())
files.extend((p,'compile/'+p.relative_to(build).as_posix()) for p in build.iterdir() if p.is_file())
pins={name:sha(p) for p,name in files}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for p,name in files:z.write(p,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
q=dict(status='independently_qualified_class2_WORLD_and_same_numeric_map_retirement',run=str(run),profile=b['profile'],source_manifest_sha256=r['source_manifest_sha256'],candidate_inputs=b['candidate_inputs'],product_sha256=r['product_sha256'],original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,actual_sessions=6,actual_lightning_casts=1,actual_tree_requests=4,actual_S5A=4,actual_C08=4,same_numeric_map_transitions=2,same_map_real109_retirement_verified=True,same_map_active_class2_flash_verified=False,roles=roles,pictures=pictures,recorded_identities=jobs,recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),controlled_local_S21_replay=True,actual_C02_night_clock=False,motion_hurt_preflight_directions=8,active_effect_at_tree_request_verified=any(role["source_active_effects"]>0 for role in roles.values()),target_motion_placement_verified=roles["a"]["target_motion_placement_verified"],active_ambient_at_tree_request_verified=any(role['source_ambient_counter']>0 for role in roles.values()),normal_product_registered=False,GUI_verified=False,physical_audio_verified=False,C72_handoff_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q);(out/'reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=b['profile'],actual_sessions=6,active_ambient_at_tree_request=q['active_ambient_at_tree_request_verified']),ensure_ascii=False))
