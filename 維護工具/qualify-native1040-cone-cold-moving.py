"""Read actual C0A/S12 movement and captured-flight/current-impact observations."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, zipfile

ROOT = Path(__file__).resolve().parents[1]
def read(p):
    return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

ap = argparse.ArgumentParser()
ap.add_argument('qualification', type=Path)
a = ap.parse_args()
qp = a.qualification.resolve(strict=True)
assert qp.is_relative_to(ROOT/'驗證紀錄/current/native1040-cone-cold-live')
q = read(qp)
assert q['status'] == 'independently_qualified_current1040_C02_source_scene_admission_and_refusal'
assert q['normal_sources_body_compiled'] and not q['source_overrides_used']
assert q['kind'] == 'cold' and q['actual_accepted_casts'] == 2
assert q['original_exit_code'] == 0 and q['recorded_owned_all_terminal']
run = Path(q['run']); r = read(run/'receipt.json'); runtime = Path(r['runtime'])
assert sha(qp.parent/'raw.zip') == q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'raw.zip') as z:
    for n, h in q['raw_files'].items():
        assert hashlib.sha256(z.read(n)).hexdigest().upper() == h
        if not n.startswith(('driver/', 'build/')) and n != 'reader.py':
            assert sha(runtime/n) == h

gate = read(runtime/'b-flight-target-gate.json')
move = read(runtime/'a-target-move-sent.json')
observed = read(runtime/'b-target-motion-observed.json')
old, new = gate['target_before'], move['destination']
assert move['source'] == old == observed['old'] and new == observed['new'] and new != old
assert move['real_movement_request_C0A'] and not move['fixture_authority_mutation']
assert gate['controlled_presentation_pause'] and observed['controlled_presentation_pause']
assert not gate['performance_acceptance'] and not observed['performance_acceptance']
actor = observed['actor']; previous = gate['captured_world']['actors']['1']
assert actor['id'] == previous['id'] == 1 and actor['incarnation'] == previous['incarnation']
assert not actor['position_predicted']
assert observed['world_generation'] == gate['captured_world']['generation']
dxdy = [(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)]
dx,dy = dxdy[move['heading']]
assert new == [old[0]+dx, old[1]+dy]
raw = []
for f in sorted((runtime/'captures').rglob('frames.jsonl')):
    for i,line in enumerate(f.read_text(encoding='utf-8-sig').splitlines(),1):
        v = json.loads(line)
        if re.fullmatch('[a-fA-F0-9 ]*',v.get('plain','')):
            raw.append((str(f.relative_to(runtime)),i,v['direction'],bytes.fromhex(v['plain'])))
c0a = b'\x0a'+old[0].to_bytes(2,'little')+old[1].to_bytes(2,'little')+bytes([move['heading']])
s12 = b'\x12'+(1).to_bytes(4,'little')+c0a[1:]
sent = [[f,i] for f,i,d,b in raw if d=='client_to_server' and b==c0a]
received = [[f,i] for f,i,d,b in raw if d=='server_to_client' and b==s12]
assert len(sent)==1 and received
states = [json.loads(s)['state'] for s in (runtime/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
assert any(s['object_id']==1 and s['phase']=='BATTLE' and [s['x'],s['y']]==new for s in states)

client = read(runtime/'b-client.json'); first = client['completed'][0]
cue = next(c for c in first['after']['world']['projectiles'] if c['projectile']['id']==gate['effect']['projectile'])
assert cue['target']==1 and cue['target_incarnation']==actor['incarnation']
assert cue['projectile']['to']==old
origin=cue['projectile']['from']; ddx=old[0]-origin[0]; ddy=old[1]-origin[1]
major=max(abs(ddx),abs(ddy)); assert 0<major<32
# Independently evaluate the small-domain fixed16 line from the captured entry.
# Native X is 2*wireX-32768; each advance adds and rounds before even-X alignment.
vx=(abs(ddx)*65536//major)*(1 if ddx>=0 else -1)*2
vy=(abs(ddy)*65536//major)*(1 if ddy>=0 else -1)
fx=((2*origin[0]-32768)<<16)&0xffffffff; fy=(origin[1]<<16)&0xffffffff
path=[]
for _ in range(max(abs(2*ddx),abs(ddy))):
    fx=(fx+vx)&0xffffffff; fy=(fy+vy)&0xffffffff
    nx=(((fx+32768)&0xffffffff)>>16)&~1; ny=((fy+32768)&0xffffffff)>>16
    p=[(nx+32768)//2,ny]
    path.append(p)
    if p==old: break
assert path[-1]==old and gate['fly_position'] in path[:-1]
assert 1<=max(abs(old[i]-gate['fly_position'][i]) for i in [0,1])<=2
timeline=read(runtime/'b-trace.json')['render_frames']
effects=[e for sample in timeline for e in sample['cone-cold']['effects'] if e['projectile']==cue['projectile']['id'] and e['sequence']==cue['action_sequence']]
flies=[e for e in effects if e['phase']['kind']=='fly' and not e['finished']]
impacts=[e for e in effects if e['phase']['kind']=='impact']
assert flies and impacts
indices=[path.index(e['position']) for e in flies]
assert indices==sorted(indices) and all(e['direction']==gate['effect']['direction'] for e in effects)
assert all(e['position']==new for e in impacts)
assert q['fake_sink_counts']=={'a':2,'b':2}
assert r['server_result']['authority_mutations']==0
out=run/('moving-qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
result=dict(status='independently_qualified_actual_C02_target_step_during_captured_cold_flight',
    admission_qualification=str(qp),admission_qualification_sha256=sha(qp),
    run=str(run),compile_profile=q['compile_profile'],product_sha256=q['product_sha256'],
    target_before=old,target_after=new,target_incarnation=actor['incarnation'],
    actual_C0A=sent,actual_remote_S12=received,wire_captured_destination=old,
    independently_evaluated_fixed16_path=path,observed_fly_positions=[e['position'] for e in flies],
    impact_anchor=new,observed_impact_samples=len(impacts),
    once_only_hurt_callbacks=q['once_only_hurt_callback_evidence'],
    controlled_presentation_pause=True,performance_acceptance=False,
    normal_product_registered=True,normal_EXE_runtime_inferred=False,source_manifest_sha256=q['source_manifest_sha256'],normal_sources_body_compiled=True,source_overrides_used=False,GUI_verified=False,physical_audio_verified=False,
    full_G06_verified=False,whole_goal_complete=False,
    evidence_sha256={f:sha(runtime/f) for f in ['b-flight-target-gate.json','a-target-move-sent.json','b-target-motion-observed.json','b-trace.json','server-trace.jsonl']})
(out/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert read(out/'qualified.json')==result
print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=q['compile_profile'],target_before=old,target_after=new,fly_samples=len(flies),impact_samples=len(impacts)),ensure_ascii=False))
