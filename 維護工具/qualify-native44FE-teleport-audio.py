"""Read actual wire events, scoped delivery, full item bytes, pixels and terminal identities."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,itertools,json,re,struct,subprocess,time,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern'
def read(p):
    for attempt in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def inventory(body):
    assert body[0]==0x7c;at=2;records={}
    def take(n):
        nonlocal at
        assert 0<=n<=len(body)-at;v=body[at:at+n];at+=n;return v
    def number(n):return int.from_bytes(take(n),'little')
    for _ in range(body[1]):
        object_id=number(4);assert object_id>0 and str(object_id) not in records
        use_type,charge,icon,blessing,count,identified=[number(n) for n in [1,1,2,1,4,1]]
        assert count>0 and blessing<=3;end=body.find(b'\0',at,at+256);assert end>=at
        name=take(end-at).decode('cp950');assert take(1)==b'\0';detail=list(take(number(1)))
        records[str(object_id)]=dict(id=object_id,use_type=use_type,charge=charge,icon=icon,blessing=blessing,count=count,identified=identified,name=name,detail=detail)
    assert at==len(body);return records
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
assert a.session_id>0 and a.terminal_exit_code==0 and re.fullmatch('[0-9a-f]+',a.terminal_chunk)
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native44FE-teleport-audio')
r=read(run/'receipt.json');assert r['status']=='executed_current_normal_body_nonempty_audio_case' and r['owned_process'] is None and r['asset_metadata_unchanged']
assert len(r['process_history'])==2 and all(j['exit_code']==0 for j in r['process_history'])
kind=r['kind'];assert kind==('success' if r['scenario']=='cross-endpoint' else 'failure')
expected_sessions=6 if kind=='success' else 10 if r['scenario']=='wrong-service' else 8;assert r['actual_sessions']==expected_sessions
build=Path(r['compile']);b=read(build/'receipt.json');assert b['status']=='compiled_current_normal_body_audio_diagnostics' and sha(build/'receipt.json')==r['compile_receipt_sha256']
assert b['source_manifest_sha256']==r['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0' and b['profile']==r['profile']
assert not b['source_overlays_used'] and not r['source_overlays_used'] and b['normal_source_bodies_unchanged'] and r['normal_source_bodies_unchanged']
assert b['diagnostic_request_API_extension'] and not b['normal_GUI_API_permission_inferred'] and not r['normal_EXE_runtime_inferred']
assert sha(r['product'])==r['product_sha256']==b['products'][kind]['sha256']
assert sha(run/'probe.rs')==sha(build/(kind+'.rs'))==b['candidate_inputs'][kind+'.rs']['sha256']
with Path(r['product']).open('rb') as f:
    assert f.read(2)==b'MZ';f.seek(60);off=struct.unpack('<I',f.read(4))[0];f.seek(off);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041 and len(read(build/'source-inputs.json'))==1040 and sha(build/'source-inputs.json')==r['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for n,h in r['pins'].items():assert sha(n)==h,n
for pin in [*b['libraries'].values(),*b['candidate_inputs'].values()]:assert sha(pin['path'])==pin['sha256']
assert sha(b['diagnostic_library'])==b['diagnostic_library_sha256'] and sha(build/'diagnostic-inputs.json')==b['diagnostic_input_manifest_sha256']==r['diagnostic_input_manifest_sha256']
original=(build/'native_login.original.rs').read_bytes();api=(build/'diagnostic_wave_api.rs').read_bytes()
assert original==(P/'src/modern_lito/native_login.rs').read_bytes() and (build/'native_login.rs').read_bytes()==original+b'\n'+api
assert len(original)==b['native_login_original_bytes'] and sha(build/'native_login.original.rs')==b['native_login_original_sha256']
assert len(api)+1==b['appended_API_bytes'] and hashlib.sha256(b'\n'+api).hexdigest().upper()==b['appended_API_sha256'] and sha(build/'native_login.rs')==b['diagnostic_native_login_sha256']
assert len(b['process_history'])==3 and [j['label'] for j in b['process_history']]==['library','success','failure'] and all(j['exit_code']==0 for j in b['process_history'])
assert '--test' not in b['process_history'][0]['arguments']
for job in b['process_history'][1:]:assert '--test' in job['arguments'] and 'lito_modern='+b['diagnostic_library'] in job['arguments'] and job['arguments'][1]==str(build/(job['label']+'.rs'))
assert tuple(b['release_configuration'][key] for key in ['opt-level','lto','codegen-units','strip'])==('z','thin',16,'symbols')
expected_options={'opt-level':'z','lto':'thin','codegen-units':'16','strip':'symbols'} if r['profile']=='Release' else {'opt-level':'0','debuginfo':'1','debug-assertions':'yes','overflow-checks':'yes'}
for job in b['process_history']:
    args=job['arguments'];options=[args[i+1] for i,arg in enumerate(args[:-1]) if arg=='-C']
    for key,value in expected_options.items():assert [option for option in options if option.startswith(key+'=')]==[key+'='+value]
terminal=read(build/'compile-terminal-observation.json');assert terminal['observed_terminal_exit_code']==0 and terminal['compile_receipt_sha256']==sha(build/'receipt.json') and terminal['original_session_id']>0
expected=[];previous=False;overrides=0;root=P/'src/modern_lito'
assert(build/'mod.rs').read_bytes()==(root/'mod.rs').read_bytes()
for line in (build/'mod.rs').read_text('utf-8-sig').splitlines():
    path=re.match(r'\s*#\[path\s*=\s*"([^"\n]+)"\]\s*$',line)
    if path:expected.append('#[path = '+json.dumps((root/path.group(1)).resolve(strict=True).as_posix(),ensure_ascii=False)+']');previous=True;continue
    decl=re.match(r'^(?:(?:pub(?:\([^)]*\))?)\s+)?mod\s+(\w+);\s*$',line)
    if decl:
        if not previous:
            name=decl.group(1)+'.rs';resolved=build/name if name=='native_login.rs' else root/name
            if name=='native_login.rs':overrides+=1
            if not resolved.exists():resolved=root/decl.group(1)/'mod.rs'
            expected.append('#[path = '+json.dumps(resolved.as_posix(),ensure_ascii=False)+']')
        previous=False
    expected.append(line)
assert overrides==1 and(build/'resolved_mod.rs').read_text('utf-8-sig')=='\n'.join(expected)+'\n'
wrapper='#[path = '+json.dumps((build/'resolved_mod.rs').as_posix(),ensure_ascii=False)+']\npub mod modern_lito;\n'
wrapper+='#[path = '+json.dumps((P/'src/shared_v1/morph_auth.rs').as_posix(),ensure_ascii=False)+']\npub mod morph_auth;\n';assert(build/'lib_source.rs').read_text('utf-8-sig')==wrapper
nq=read(b['normal_qualification']);assert sha(b['normal_qualification'])==b['normal_qualification_sha256'] and nq['source_manifest_sha256']==r['source_manifest_sha256'] and(nq['passed'],nq['failed'],nq['ignored'])==(23,0,0)
normal=read(Path(b['normal_cohort'])/'cohort.json')
for info in normal['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
g06=Path(r['prerequisite_G06']);gq=read(g06/'qualified/qualified.json');assert sha(g06/'qualified/qualified.json')==b['g06_qualification_sha256'] and gq['actual_formal_sessions']==96 and gq['source_manifest_sha256']==r['source_manifest_sha256']
assert sha(b['formal_C1F_binding']['path'])==b['formal_C1F_binding']['sha256'];binding=read(b['formal_C1F_binding']['path']);assert binding['request_opcode']==31 and binding['request_action']==68
case=Path(r['case']);c=read(case/'receipt.json');runtime=Path(c['runtime'])
comparison_name={'cross-endpoint':'teleport-handoff-comparison.json','timeout':'teleport-failure-comparison.json','refused':'teleport-refused-comparison.json','wrong-service':'teleport-wrong-service-comparison.json','cancel-retry':'teleport-cancel-retry-comparison.json','cancel-expiry':'teleport-cancel-expiry-comparison.json'}[r['scenario']]
comparison=read(case/comparison_name);assert comparison['receipt_sha256']==sha(case/'receipt.json') and comparison['test_binary_sha256']==r['product_sha256'] and len(comparison['checks'])==2 and not comparison['gui_verified']
assert c['inputs_unchanged'] and c['test_exit_code']==c['server_exit_code']==0 and c['test_binary_sha256']==sha(c['immutable_test_binary'])==r['product_sha256']
server=c['server'];assert server['actual_client_sessions']==expected_sessions and server['memberships_empty'] and server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
assert c['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'==sha(ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')
for n,h in c['source_pins'].items():assert sha(case/'source-snapshot'/n)==h
if kind=='success':assert c['harness_source_sha256']==r['pins'][c['harness_source']]
else:
    harness_key='LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'
    assert c['source_pins'][harness_key]==inputs[harness_key]
assert sha(case/'server-trace.jsonl')==comparison['server_trace_sha256']
server_error=(case/'server.stderr.txt').read_bytes()
try:server_text=server_error.decode('utf-8-sig')
except UnicodeDecodeError:
    server_text=server_error.decode('cp950');assert server_text.encode('cp950')==server_error
assert 'Picked up JAVA_TOOL_OPTIONS: '+r['JAVA_TOOL_OPTIONS_for_owned_children_only'] in server_text
assert '-Dlito.resourceProfile=tw13081901-v1 -Dlito.extendedItemDetails=true' in r['JAVA_TOOL_OPTIONS_for_owned_children_only']
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
for n,v in read(run/'asset-metadata-after.json').items():assert(Path(n).stat().st_size,Path(n).stat().st_mtime_ns)==(v['bytes'],v['mtime_ns'])
classes={p.relative_to(runtime).as_posix():sha(p) for p in(runtime/'oracle').rglob('*.class')};assert classes
captures=list((runtime/'captures').glob('*/frames.jsonl'));assert len(captures)==1 and sha(captures[0])==comparison['capture_sha256']
frames=[json.loads(line) for line in captures[0].read_text('utf-8-sig').splitlines()]
assert sum(f['plain']=='MASKED_C_LOGIN' for f in frames)==expected_sessions
raw=[(i+1,f,bytes.fromhex(f['plain'])) for i,f in enumerate(frames) if re.fullmatch('[a-fA-F0-9 ]*',f.get('plain',''))]
requests=[n for n,f,data in raw if f['direction']=='client_to_server' and data==b'\x1f\x44'];assert len(requests)==(2 if kind=='success' else 4)
batches=[(n,inventory(data)) for n,f,data in raw if f['direction']=='server_to_client' and data[:1]==b'\x7c'];assert batches and any(item['detail'] for _,batch in batches for item in batch.values())
wav_sha={0:'6D4ED3DE2AC408F2CA81F252C6840282FCB044DC6CC50CC354B848E5B5A1F8E5',61:'4F46F1E4D31E9D4802B36A42BB9A073BFDB32690FAE9EF926D2A0ABE73D7257B'}
def wave_proof(meta,index,epoch,generation):
    world=meta['before']['world'];own=world['status']['object_id'];graphics=world['actors'][str(own)]['graphics'];assert graphics in [0,61];sound={0:1534,61:1564}[graphics]
    assert meta['actual_action_request'] and meta['diagnostic_request_API_extension'] and not meta['normal_GUI_action_request_permission_inferred']
    assert meta['request_hex']=='1f44' and meta['action']==68 and meta['sound']==sound and meta['deferred']
    assert meta['before']['world']==meta['immediately_after_queue']['world'] and meta['before']['inventory']==meta['immediately_after_queue']['inventory']
    held=meta['held'];row=held['row'];event=row['event'];assert held['epoch']==epoch and held['generation']==generation
    assert row['prepared'] and row['delivery_kind']=='legacy_scoped' and row['sha256'].upper()==wav_sha[graphics]
    wav=ROOT.parent.parent/'0-客戶端'/('Sound/'+str(sound)+'.wav');assert sha(wav)==wav_sha[graphics] and row['bytes']==wav.stat().st_size
    assert event['actor']==own and event['action']==68 and event['sound']==sound and event['frame']==2 and event['event']==f'Sound({sound})'
    actions=[x for x in meta['after_ack']['world']['actors'][str(own)]['actions'] if x['sequence']==event['sequence'] and x['action']==68 and x['source']=='action'];assert len(actions)==1
    request=requests[index];upper=requests[index+1] if index+1<len(requests) else len(frames)+1
    actors=[data for n,f,data in raw if n<request and f['direction']=='server_to_client' and len(data)>=11 and data[0]==11 and int.from_bytes(data[5:9],'little')==own]
    assert actors and int.from_bytes(actors[-1][9:11],'little')==graphics
    wire=b'\x20'+own.to_bytes(4,'little')+b'\x44';received=[n for n,f,data in raw if request<n<upper and f['direction']=='server_to_client' and data==wire];assert received
    return dict(object_id=own,graphics=graphics,sound=sound,request_line=request,S20_lines=received,upper_line=upper,event_sequence=event['sequence'],epoch=epoch,generation=generation,wav_sha256=wav_sha[graphics])
reports={role:read(runtime/(role+('-handoff.json' if kind=='success' else '-failure.json'))) for role in ['a','b']}
proofs=[];pngs=[];inventory_proofs=[]
for role,v in reports.items():
    name=role+('-handoff.json' if kind=='success' else '-failure.json');assert sha(runtime/name)==sha(case/name)
    assert v['actual_Presentation_executed'] and not v['GUI_acceptance'] and not v['physical_audio_verified']
    before=v['before'];retired=v['destination_retirement' if kind=='success' else 'failure_retirement'];assert retired['view_quiescent'] and retired['frame_absent'] and retired['old_surface_rejected'] and retired['retained_source_cues']==1
    assert retired['audio_deliveries_before']==retired['audio_deliveries_after']
    source_index=(0 if role=='a' else 1) if kind=='success' else (0 if role=='a' else 2)
    source=wave_proof(v['source_audio'],source_index,v['old_epoch'],before['world']['generation'])
    action=b'\x27'+(1812987996).to_bytes(4,'little')+v['action'].encode('ascii')+bytes(2)
    actions=[n for n,f,data in raw if f['direction']=='client_to_server' and data==action];assert len(actions)==1
    assert source['request_line']<min(source['S20_lines'])<actions[0]
    transfers=[n for n,f,data in raw if actions[0]<n<source['upper_line'] and f['direction']=='server_to_client' and data[:1]==b'\xf4'];assert transfers
    if kind=='success':
        assert v['new_epoch']==v['old_epoch']+1 and v['old_peer']!=v['new_peer'] and v['old_frame_rejected'] and not v['same_endpoint']
        destination=v['destination'];recovered=v['battle'];assert before['selected_character']==destination['selected_character']==recovered['selected_character']
        assert not destination['world']['actors'] and destination['joined_room'] is None and destination['services'] is None and destination['rental_set'] is None
        assert recovered['world']['generation']>before['world']['generation'] and recovered['world']['map_id']==564
        assert retired['cue_outcomes']==['Stale'] and retired['repeated_cue_outcomes']==['AlreadyConsumed'] and retired['held_source_meta']==v['source_audio']['held'] and retired['old_cue_device_deliveries']==0
        audio=v['destination_audio'];own=audio['before']['world']['status']['object_id'];packet=b'\x14\x00\x00'+own.to_bytes(4,'little')
        assert audio['request_hex']==packet.hex() and not audio['deferred'] and audio['own_deliveries_after']==audio['own_deliveries_before']+1
        heal=[n for n,f,data in raw if max(transfers)<n and f['direction']=='client_to_server' and data==packet];assert heal
        effects=[n for n,f,data in raw if min(heal)<n and f['direction']=='server_to_client' and data==b'\x37'+own.to_bytes(4,'little')+(11722).to_bytes(2,'little')];assert effects
        delivery=audio['own_delivery_events'][-1];assert delivery['epoch']==v['new_epoch'] and delivery['generation']==recovered['world']['generation'] and delivery['row']['event']['actor']==own and delivery['row']['event']['sound']==109
        new_audio=dict(kind='native_heal1_sound109',C14_lines=heal,S37_lines=effects,delivered_once=True)
        continued=destination;stages=[('source',before),('battle',recovered)]
    else:
        assert [v['failed_epoch'],v['recovery_epoch'],v['reentered_epoch']]==[v['old_epoch']+n for n in [1,2,3]]
        failed=v['failed'];continued=v['continued'];recovered=v['presentation_recovered_snapshot']
        assert not failed['world']['actors'] and failed['inventory']['owner'] is None and failed['selected_character'] is None
        assert before['selected_character']==continued['selected_character']==recovered['selected_character'] and recovered['world']['generation']>before['world']['generation'] and recovered['world']['map_id']==500 and v['old_surface_rejected_after_reentry']
        assert retired['retirement_ms']<3000 and retired['cue_outcomes']==[dict(first='Stale',repeat='AlreadyConsumed')]
        partial=read(runtime/(role+'-source-audio-retirement.json'));assert partial==dict(source_audio=v['source_audio'],retirement=retired,whole_case_qualified=False)
        fresh=v['recovery_audio'];new_audio=wave_proof(fresh['request'],source_index+1,v['reentered_epoch'],recovered['world']['generation'])
        assert min(transfers)<new_audio['request_line'] and fresh['first']==f"Played(Started {{ sound_id: {new_audio['sound']}, slot: 0 }})" and fresh['repeat']=='AlreadyConsumed' and fresh['deliveries_after']==fresh['deliveries_before']+1
        own=recovered['world']['actors'][str(recovered['world']['status']['object_id'])];assert [own['x'],own['y']]==v['recovery_move']['after']
        if r['scenario']=='cancel-expiry':assert v['manual_selection'] and v['closed_samples']>0 and v['expiry_wait_ms']>=33000
        if r['scenario']=='cancel-retry':assert not v['manual_selection'] and v['closed_samples']>0 and 600<=v['expiry_wait_ms']<30000
        stages=[('source',before),('recovered',recovered)]
    original=[(i,j) for i,j in itertools.combinations(range(len(batches)),2) if batches[i][0]<actions[0] and batches[j][0]<actions[0] and not(set(batches[i][1])&set(batches[j][1])) and batches[i][1]|batches[j][1]==before['inventory']['items']]
    continued_batches=[i for i,(n,batch) in enumerate(batches) if n>actions[0] and batch==continued['inventory']['items']];assert len(original)==len(continued_batches)==1
    inventory_proofs.append(dict(role=role,original_batches=original[0],continued_batch=continued_batches[0],all_item_fields_exact=True))
    shutdown=read(runtime/(role+'-presentation-shutdown.json'));assert shutdown['client_quiescent'] and shutdown['Presentation_quiescent'] and shutdown['shutdown_error'] is None
    journal=read(runtime/(role+'-presentation-journal.json'));assert journal['error'] is None and not journal['GUI_acceptance']
    for row in v['presentation_trace']:
        state=row['state'];frame=state['frame']
        if frame is not None:assert frame['generation']==state['world_generation'] and state['world_ready'] and state['phase'] in ['battle_pending','world_ready']
    for label,snapshot in stages:
        picture=v[label+'_picture'];path=Path(picture['png']);receipt=picture['receipt'];assert not picture['GUI_acceptance']
        assert receipt['world_generation']==snapshot['world']['generation'] and receipt['map_id']==snapshot['world']['map_id'] and receipt['local_object_id']==snapshot['world']['status']['object_id']
        with Image.open(path) as im:assert im.size==(800,600) and im.mode=='RGBA' and hashlib.sha256(im.tobytes()).hexdigest().lower()==receipt['rgba_sha256'].lower()
        pngs.append(dict(role=role,stage=label,path=str(path),sha256=sha(path),rgba_sha256=receipt['rgba_sha256'],map_id=receipt['map_id'],generation=receipt['world_generation']))
    proofs.append(dict(role=role,source=source,F4_lines=transfers,old_first='Stale',old_repeat='AlreadyConsumed',old_deliveries=0,new_audio=new_audio,FakePlayer_only=True))
jobs=r['process_history']+[read(run/'runner-identity.json'),b['original_runner']]+b['process_history']+[dict(pid=j['pid'],creation_utc=j['start_utc']) for j in c['owned_processes']]
ids=','.join(str(j['pid']) for j in jobs);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
output=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(output.decode('utf-8-sig')) if output.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for j in jobs:
        if row['ProcessId']==j['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
assert not list(run.glob('qualified-*'));out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'recorded-process-CIM.json').write_bytes(output)
files={}
for family,folder in [('driver',run),('case',case),('runtime',runtime),('compile',build)]:
    for path in folder.rglob('*'):
        if path.is_file() and not path.is_relative_to(out) and path.suffix.lower() not in {'.exe','.class','.jar','.zip'}:files[family+'/'+path.relative_to(folder).as_posix()]=path
pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_current_normal_bodies_nonempty_audio_branch',run=str(run),profile=r['profile'],scenario=r['scenario'],
    source_manifest_sha256=r['source_manifest_sha256'],diagnostic_input_manifest_sha256=b['diagnostic_input_manifest_sha256'],actual_product_sha256=r['product_sha256'],
    original_session_id=a.session_id,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,actual_sessions=expected_sessions,actual_Presentation_executed=True,
    source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,normal_GUI_API_permission_inferred=False,
    retained_source_cues=2,unplayed_source_audio_rejection_verified=True,new_scene_own_sound_events_delivered_once_each=True,wire_audio_proofs=proofs,
    inventory_all_fields=inventory_proofs,diagnostic_PNGs=pngs,actual_harness_class_pins=classes,
    recorded_owned_identities=len({(j['pid'],j['creation_utc']) for j in jobs}),recorded_owned_all_terminal=True,
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,normal_product_source_modified=False,normal_EXE_runtime_inferred=False,
    GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'qualified.json',q);print('QUALIFICATION='+str(out/'qualified.json'));print(json.dumps(dict(profile=r['profile'],scenario=r['scenario'],actual_sessions=expected_sessions,old_cues_rejected=2,new_cues_delivered_once=2),ensure_ascii=False))
