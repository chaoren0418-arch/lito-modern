"""Read actual wire, passive C02 authority and both normal-EXE presentations."""
from pathlib import Path
import argparse,hashlib,json,os,re,sys
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def local_path(value):
    original=Path(value).resolve(strict=True)
    text=str(original)
    if text.startswith('\\\\?\\UNC\\'):text='\\\\'+text[8:]
    elif text.startswith('\\\\?\\'):text=text[4:]
    normalized=Path(text).resolve(strict=True)
    assert os.path.samefile(original,normalized)
    return normalized
ap=argparse.ArgumentParser();ap.add_argument('receipt',type=Path);ap.add_argument('--normal-inputs',type=Path)
a=ap.parse_args()
p=a.receipt.resolve(strict=True)
assert p.is_relative_to(ROOT/'驗證紀錄/current/native-real-peers')
r=read(p);profile=r['action_profile'];positive=profile=='detection-invisibility'
assert profile in ['detection-invisibility','detection-refused','invisibility-refused','invisibility-missing']
assert r['status']=='verified_real_peer_action' and r['source_matches_snapshot']
assert r['jar_sha256']==r['source_sha256_at_end']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert r['resource_profile'] in ['lito-legacy','tw13081901-v1']
if r['resource_profile']=='tw13081901-v1':
    assert r['extended_item_details'] and r['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
server=r['server'];assert server['actual_client_sessions']==4 and server['game_sessions']==2
assert server['memberships_empty'] and server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
assert server['classic_world_members_after_disconnect']==server['active_hotel_leases_after_disconnect']==0
build_profile=r['profile'];assert build_profile in ['Debug','Release']
if a.normal_inputs is not None:
    run=a.normal_inputs.resolve(strict=True)
    assert run.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
    q=read(run/'qualified/qualified.json');stage=read(run/'cohort.json')
    assert q['status']=='independently_bound_existing_normal_for_visibility_runtime'
    assert q['actual_casts']==q['actual_test_executables_run_here']==q['actual_processes_launched_here']==0
    upstream=Path(q['normal_qualification']).resolve(strict=True)
    assert upstream.is_relative_to(ROOT/'驗證紀錄/current/native-presentation-source-owner')
    assert sha(upstream)==q['normal_qualification_sha256']
    normal=read(upstream)
    assert normal['observed_terminal_exit_code']==0 and normal['recorded_owned_processes_all_terminal']
    assert normal['source_manifest_sha256']==q['source_manifest_sha256']
    assert normal['profiles'][build_profile]['normal_sha256']==r['exe_sha256']
    assert sha(ROOT/'啟動現代客戶端.ps1')==normal['entry_sha256']
    broad=Path(q['broad_qualification']).resolve(strict=True)
    assert broad.is_relative_to(ROOT/'驗證紀錄/current/default-native-regression')
    assert sha(broad)==q['broad_qualification_sha256']
    assert read(broad)['source_manifest_sha256']==q['source_manifest_sha256']
else:
    app=ROOT/'驗證紀錄/current/native-detection-invisibility/application-20261017-015820-458055'
    stage=read(app/('release-stage.json' if build_profile=='Release' else 'current-stage.json'))
    run=Path(stage['run']);q=read(run/'qualified/qualified.json')
assert q.get('profile','Debug')==build_profile
assert q['observed_terminal_exit_code']==0 and q['normal_sha256']==r['exe_sha256']
assert q['source_manifest_sha256']==stage['source_manifest_sha256']
pins=read(run/'compiled-inputs.json');assert len(pins)==1025
for n,h in pins.items():assert sha(ROOT/n)==h,n
runtime=Path(r['runtime']).resolve(strict=True);assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current/native-real-peers')
assert sha(runtime/'qualified-client.exe')==r['exe_sha256']
for role in ['a','b']:
    launch=read(p.parent/f'{role}-launch-identity.json')
    assert launch['role']==role and launch['exe_sha256']==r['exe_sha256'] and launch['pid']>0 and launch['creation_utc']
launch=read(p.parent/'server-launch-identity.json');assert launch['exe_sha256']==r['exe_sha256'] and launch['jar_sha256']==r['jar_sha256']
files=list((runtime/'captures').glob('*/frames.jsonl'));assert len(files)==1
wire=[]
for line in files[0].read_text(encoding='utf-8-sig').splitlines():
    row=json.loads(line)
    if re.fullmatch('[0-9a-fA-F ]*',row.get('plain','')):wire.append((row,bytes.fromhex(row['plain'])))
requests=[(i,b) for i,(row,b) in enumerate(wire) if row['direction']=='client_to_server' and b[:1]==b'\x14']
expected=[60,13,60,13] if positive else [13 if profile=='detection-refused' else 60]
assert len(requests)==len(expected)
for (_,body),skill in zip(requests,expected):
    assert body[:3]==(bytes.fromhex('140104') if skill==13 else bytes.fromhex('140703'))
    assert len(body)==(3 if skill==13 else 7)
assert not [b for row,b in wire if row['direction']=='client_to_server' and b[:1] in [b'\x10',b'\x1a']]
effects=[b for row,b in wire if row['direction']=='server_to_client' and b[:1]==b'\x37']
assert all(len(b)==7 and int.from_bytes(b[5:7],'little')==749 for b in effects)
trace=[json.loads(l)['state'] for l in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
assert trace
checks=[]
for role,own_id in [('a',1),('b',3)]:
    c=r['clients'][role];probe=c['visibility_probe'];start=c['start']['world'];end=c['end']['world']
    assert c['status']=='real_peer_action_completed' and c['action_profile']==profile and c['worker_quiescent'] and c['shutdown']=='closed'
    assert start['status']['object_id']==end['status']['object_id']==own_id and start['generation']==end['generation']
    assert probe['completed_rounds']==(2 if positive else 1)
    actions=[a for a in probe['actions'] if 'skill' in a]
    action_expected=([13,60] if role=='a' else [60,13]) if positive else (expected if role=='b' else [])
    assert [a['skill'] for a in actions]==action_expected
    for a in actions:
        before=a['before'];immediate=a['immediately_after']
        assert before['world']['mp']==immediate['world']['mp'] and before['inventory']==immediate['inventory']
        assert a['self_intent'] and a['skill'] in before['world']['learned_skills']
    authority=[s for s in trace if s['object_id']==own_id and s['phase']=='BATTLE']
    assert authority
    if positive:
        assert any(s['invisible_active'] for s in authority) and any(not s['invisible_active'] for s in authority)
        assert authority[0]['magic_gems']==8 and any(s['magic_gems']==7 and not s['invisible_active'] for s in authority)
        assert start['mp'][0]>end['mp'][0]>0 and end['hp']==start['hp']
        assert any(s['mp']==end['mp'][0] and s['magic_gems']==7 and not s['invisible_active'] for s in authority)
        hidden=[a for a in probe['actions'] if a.get('observed_hidden')];assert len(hidden)==2
        assert sum(a['local_half'] for a in hidden)==sum(a['remote_target_and_input_retired'] for a in hidden)==1
        retired=[a for a in probe['actions'] if a.get('revealed_and_effects_retired')];assert len(retired)==2
        assert all(not a['draw_state']['invisible'] for a in end['actors'].values() if a['id'] in [1,3])
        before_items=actions[0]['before']['inventory']['items'];after_items=c['end']['inventory']['items']
        assert set(before_items)==set(after_items)
        changed=[(before_items[k],after_items[k]) for k in before_items if before_items[k]['count']!=after_items[k]['count']]
        assert len(changed)==1 and changed[0][0]['count']-changed[0][1]['count']==1
    else:
        assert not any(s['invisible_active'] for s in authority)
        assert len({s['magic_gems'] for s in authority})==1
        if role=='b':
            # These reviewed C02 helpers log and return on MP/reagent refusal;
            # they emit no S64 rejection code. Preserve the ordinary 2s gate.
            assert c['end']['skill_feedback']=='NoResponse'
            assert actions[0]['before']['world']['mp']==end['mp'] and actions[0]['before']['inventory']==c['end']['inventory']
            if profile.endswith('refused'):assert end['mp']==[2,2]
        assert not end['effects'] and end['own_cast_packets']==0
    observations=probe['observations'];assert observations and len(observations)<2048
    received=sum(e['graphics']==749 for e in end['effects'])
    assert observations[-1]['effects']['started']==received and observations[-1]['effects']['active']==0
    sound=[a for v in observations for a in v['audio'] if a['event']['sound']==108]
    assert len(sound)==received and all(a['prepared'] for a in sound)
    if positive:
        assert any(v['effects'].get('sources') and any(s['name']=='749-0.spr' and s['sha256'].upper()==
            '01D399D0D7F4ACC94AFD4A5A03E4373E101C57BDEF078EF8EAFD6492B792A8B7' for s in v['effects']['sources']) for v in observations)
    for image in probe['images']+[c['render'],c['start']['render']]:
        png=local_path(image['png']);assert png.is_relative_to(runtime)
        with Image.open(png) as img:
            assert img.size==(800,600) and img.mode=='RGBA'
            assert hashlib.sha256(img.tobytes()).hexdigest().upper()==image['frame']['rgba_sha256'].upper()
    checks.append(dict(role=role,actual_cast_requests=len(actions),received749=received,sound108_preparations=len(sound),images=len(probe['images'])))
if positive:
    for round,hider in [(0,3),(1,1)]:
        begin=requests[round*2][0];reveal=requests[round*2+1][0];stop=requests[round*2+2][0] if round==0 else len(wire)
        hid=[b for row,b in wire[begin:reveal] if row['direction']=='server_to_client']
        assert bytes([0x34])+hider.to_bytes(4,'little')+b'\x01' in hid
        assert bytes([0x15])+hider.to_bytes(4,'little') in hid
        after=[b for row,b in wire[reveal:stop] if row['direction']=='server_to_client']
        assert any(b[:1]==b'\x0b' and int.from_bytes(b[5:9],'little')==hider for b in after)
    assert len(effects)==sum(c['received749'] for c in checks)
else:assert not effects
out=dict(status='independently_compared_normal_EXE_visibility_wire_authority_and_frames',receipt_sha256=sha(p),
    profile=profile,source_manifest_sha256=q['source_manifest_sha256'],normal_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],
    actual_client_sessions=4,actual_cast_requests=len(requests),checks=checks,wire_sha256=sha(files[0]),
    runtime_authority_mutations=0,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(p.parent/'visibility-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(out,ensure_ascii=False))
