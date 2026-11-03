"""Independent raw C14, C02 authority and production Presentation diagnostic reader."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,time,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-live')
r=read(run/'receipt.json');assert r['status']=='executed_owned_native77_controller_presentation' and r['all_started_processes_terminal'] and r['asset_metadata_unchanged']
runtime=Path(r['runtime']);build=Path(r['compile']);b=read(build/'receipt.json');assert sha(build/'receipt.json')==r['compile_receipt_sha256'] and b['status']=='compiled_current_production_library_wrapper'
assert sha(r['product'])==r['product_sha256']==b['product_sha256'] and sha(run/'probe.rs')==sha(build/'probe.rs')==b['probe_sha256']
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
for v in b['libraries'].values():assert sha(v['path'])==v['sha256']
assert sha(r['reference_jar'])==r['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(r['common_catalog'])==r['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
for n,h in r['harness_class_pins'].items():assert sha(n)==h
assert sha(r['fixture_qualification'])==r['fixture_qualification_sha256']
for p in ['Debug','Release']:
    q=read(next(Path(r['normal_cohort']).glob(p+'-qualified-*/qualified.json')));assert q['passed']==30 and q['source_manifest_sha256']==r['source_manifest_sha256']
server=read(runtime/'result.json');assert server['actual_client_sessions']==4 and server['game_sessions']==2 and server['memberships_empty'] and server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
config=(runtime/'server.properties').read_text(encoding='utf-8-sig')
for s in ['database.mode=memory','database.read.only=true','state.persistence.enabled=false','battle.ai.enabled=false']:assert s+'\n' in config
fixture=read(runtime/'initial-disintegrate-fixture.json');assert fixture['before_all_sessions'] and fixture['runtime_mutations']==0 and fixture['natural_special_critical'] and fixture['profile']==r['profile']
clients={p:read(runtime/(p+'-client.json')) for p in ['a','b']};caster=clients['b'];positive=r['profile']=='disintegrate'
assert all(v['workers_quiescent'] and not v['GUI_acceptance'] and not v['real_audio_device_opened'] for v in clients.values())
raw=[]
for f in sorted((runtime/'captures').rglob('frames.jsonl')):
    for i,s in enumerate(f.read_text(encoding='utf-8-sig').splitlines(),1):
        p=json.loads(s)
        if re.fullmatch('[a-fA-F0-9 ]*',p.get('plain','')):raw.append((str(f),i,p,bytes.fromhex(p['plain'])))
requests=[(i,v) for i,v in enumerate(raw) if v[2]['direction']=='client_to_server' and v[3][:1]==b'\x14'];assert len(requests)==len(caster['completed']) and requests
trace=[json.loads(s)['state'] for s in (runtime/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()];authority={id:[v for v in trace if v['object_id']==id and v['phase']=='BATTLE'] for id in [1,3]};assert all(authority.values())
initial_mp=5000 if r['profile']!='disintegrate-refused' else 2
assert caster['initial']['state']['world']['mp']==[initial_mp,initial_mp]
assert all(v['lawful']==(499 if r['profile']=='disintegrate-lawful' else 1000) for v in authority[3])
graphics=set();per=[]
for n,((start,request),response) in enumerate(zip(requests,caster['completed'])):
    wire=request[3];before=response['before'];immediate=response['immediately_after_queue'];after=response['after'];target=before['world']['actors']['1'];own=before['world']['actors']['3']
    expected=b'\x14\x09\x04'+(1).to_bytes(4,'little')+target['x'].to_bytes(2,'little')+target['y'].to_bytes(2,'little');assert wire==expected and response['request_hex']==wire.hex()
    assert max(abs(own['x']-target['x']),abs(own['y']-target['y']))<=10
    for key in ['hp','mp']:assert before['world'][key]==immediate['world'][key]
    assert before['inventory']==immediate['inventory'] and response['pending_duplicate_error']
    end=requests[n+1][0] if n+1<len(requests) else len(raw);window=[v for v in raw[start+1:end] if v[2]['direction']=='server_to_client']
    attacks=[v[3] for v in window if len(v[3])==29 and v[3][0]==0x23 and v[3][1]==18 and int.from_bytes(v[3][2:6],'little')==3 and int.from_bytes(v[3][6:10],'little')==1]
    mana=[before['world']['mp'][0]]+[int.from_bytes(v[3][1:3],'little') for v in window if len(v[3])==5 and v[3][0]==0x4d and int.from_bytes(v[3][3:5],'little')==initial_mp]
    rejections=[int.from_bytes(v[3][1:3],'little') for v in window if len(v[3])==4 and v[3][0]==0x10]
    if positive:
        assert response['feedback']=='CastAction' and after['world']['own_cast_packets']==before['world']['own_cast_packets']+1
        assert attacks and len(set(attacks))==1 and any(x-y==70 for x,y in zip(mana,mana[1:]))
        gfx=int.from_bytes(attacks[0][16:18],'little');assert gfx in {11746,11748} and response['graphics']==gfx;graphics.add(gfx)
        assert response['active_seen'] and response['hurt_seen'] and response['final_receipt']['active']==0 and response['final_receipt']['hit_entries']>=n+1
        cooldown=response['post_ack_cooldown'];assert cooldown['authority_unchanged'] and cooldown['body_ready'] and cooldown['elapsed_ms']<5000 and cooldown['error']=='等待施法動作完成'
    else:
        code=response['feedback']['Rejected'];assert code in rejections and (r['profile']!='disintegrate-refused' or code==278)
        assert not attacks and after['world']['own_cast_packets']==before['world']['own_cast_packets'] and before['world']['mp']==after['world']['mp'] and not response['active_seen']
        assert all(x<=y for x,y in zip(mana,mana[1:]))
    assert after['world']['mp'][0] in mana and any(v['mp']==after['world']['mp'][0] for v in authority[3])
    per.append(dict(request_hex=wire.hex(),capture_line=request[1],feedback=response['feedback'],wire_MP=mana,attack_graphics=response['graphics']))
accepted=sum(v['feedback']=='CastAction' for v in caster['completed']);assert accepted==(len(requests) if positive else 0)
assert caster['played'].count(717)==accepted and set(caster['graphics_observed'])==graphics
for role,client in clients.items():
    image=client['initial']['render'];path=Path(image['png'])
    with Image.open(path) as im:assert im.size==(800,600) and im.mode=='RGBA' and hashlib.sha256(im.tobytes()).hexdigest().upper()==image['receipt']['rgba_sha256'].upper()
jobs=r['process_history']+[read(run/'original-runner-process.json')];assert len(r['process_history'])==3 and all(j['exit_code']==0 for j in r['process_history'])
ids=','.join(str(j['pid']) for j in jobs);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
output=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(output.decode('utf-8-sig')) if output.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for j in jobs:
        if row['ProcessId']==j['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());files={p.relative_to(runtime).as_posix():p for p in runtime.rglob('*') if p.is_file()};files.update({'driver/'+p.name:p for p in [run/'receipt.json',run/'runner.py',run/'probe.rs',run/'NativeSelectorHarness.java']});files['build/receipt.json']=build/'receipt.json';pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_native77_C02_controller_presentation_diagnostic_scope',run=str(run),source_manifest_sha256=r['source_manifest_sha256'],profile=r['profile'],original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,product_sha256=r['product_sha256'],actual_sessions=4,actual_requests=len(requests),actual_accepted_casts=accepted,graphics_observed=sorted(graphics),both_critical_variants_observed=graphics=={11746,11748},per_request=per,recorded_owned_identities=4,recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),legacy_runner_completed_responses_mislabeled_as_casts=(r['actual_casts']!=accepted),GUI_verified=False,physical_audio_verified=False,full_G06_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q);print(json.dumps({k:q[k] for k in ['status','profile','actual_requests','actual_accepted_casts','graphics_observed','both_critical_variants_observed']},ensure_ascii=False));print('QUALIFICATION='+str(out/'qualified.json'))
