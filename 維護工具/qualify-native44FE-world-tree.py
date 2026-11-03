"""Read raw 75DA packets, owner audio events, inputs, PNGs and process identities."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native44FE-world-tree')
assert a.terminal_chunk not in {'pending','unknown',''},'actual observed terminal chunk required'
c=read(run/'receipt.json');assert c['status']=='executed_diagnostic_pending_independent_qualification' and c['actual_sessions']==4 and c['owned_processes']==[] and c['asset_metadata_unchanged']
runtime=Path(c['runtime']);build=Path(c['compile']);b=read(build/'receipt.json');inputs=read(build/'compiled-inputs.json')
assert sha(build/'receipt.json')==c['compile_receipt_sha256'] and sha(c['product'])==c['product_sha256']==b['product_sha256']
assert sha(c['reference_jar'])==c['reference_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(c['catalog'])==c['catalog_sha256']
assert c['source_manifest_sha256']==b['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0' and len(inputs)==1041
assert c['normal_source_bodies_unchanged'] and not c['source_overlays_used'] and not c['diagnostic_API_extension']
normal=read(Path(b['normal_qualification']));assert sha(Path(b['normal_qualification']))==b['normal_qualification_sha256'] and normal['passed']==23
assert normal['source_manifest_sha256']==c['source_manifest_sha256'] and sha(Path(normal['normal']['path']))==normal['normal']['sha256']
assert sha(ROOT/'LiTo381-Modern/tests/oracle/NativePeerHarness.java')==c['canonical_harness_sha256']

for n,h in inputs.items():assert sha(ROOT/n)==h,n
for row in b['libraries'].values():assert sha(row['path'])==row['sha256']
for n,h in c['harness_class_pins'].items():assert sha(n)==h,n
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
pins={};packets={};pictures={};owners={}
for stage in ['source']:
    result=read(runtime/stage/'result.json')
    assert result['actual_client_sessions']==4 and result['game_sessions']==2 and result['memberships_empty'] and result['authority_mutations']==0 and not result['sql_access']
    rows=[]
    for p in (runtime/stage/'captures').rglob('frames.jsonl'):
        pins[str(p)]=sha(p);rows.extend(json.loads(x) for x in p.read_text(encoding='utf-8-sig').splitlines())
    requests=[r for r in rows if r['direction']=='client_to_server' and r['opcode']=='0x14']
    effects=[r for r in rows if r['direction']=='server_to_client' and r['opcode']=='0x37']
    expected_heals=[bytes.fromhex('14 00 00')+own.to_bytes(4,'little') for own in [1,3]]
    tree=bytes.fromhex('14 10 02')
    assert sorted(bytes.fromhex(r['plain']) for r in requests)==sorted(expected_heals*2+[tree,tree])
    portal=[r for r in rows if r['direction']=='server_to_client' and r['opcode']=='0x5A']
    expected_portal=bytes([0x5a])+int(4).to_bytes(2,'little')+int(33051).to_bytes(2,'little')+int(32337).to_bytes(2,'little')
    assert len(portal)==2 and all(bytes.fromhex(r['plain'])==expected_portal for r in portal)
    assert sum(r['direction']=='client_to_server' and r['plain']=='08' for r in rows)==2
    for own in [1,3]:
        expected=bytes([0x37])+own.to_bytes(4,'little')+(11722).to_bytes(2,'little')
        assert sum(bytes.fromhex(r['plain'])==expected for r in effects)==4,(stage,own,effects)
    assert sum(r['plain']=='MASKED_C_LOGIN' for r in rows)==4
    packets[stage]=dict(actual_heal1_requests=4,actual_world_tree_requests=2,actual_S_PORTAL=len(portal),actual_C08_ack=2,actual_S37_heal_broadcasts=8,targets=[1,3],graphics=11722)
    pins[str(runtime/stage/'result.json')]=sha(runtime/stage/'result.json')
for role,own in [('a',1),('b',3)]:
    p=runtime/(role+'-result.json');j=read(p);pins[str(p)]=sha(p)
    before=j['before'];after=j['after'];walk=j['after_walk']
    assert j['old_epoch']==j['new_epoch'] and after['world']['generation']==before['world']['generation']+1
    assert before['world']['map_id']==564 and after['world']['map_id']==4 and after['world']['completed_portals']==1
    authority=[json.loads(line)['state'] for line in (runtime/'source/server-trace.jsonl').read_text('utf-8-sig').splitlines()]
    authority=[row for row in authority if row['object_id']==own and row['test_account']==('PeerFixture' if role=='a' else 'PeerFixture2')]
    first=next(row for row in authority if row['phase']=='BATTLE')
    arrival=next(row for row in authority if row['phase']=='WORLD' and row['classic_world_map']==4)
    last=next(row for row in reversed(authority) if row['phase']=='WORLD')
    assert first['session_identity']==arrival['session_identity']==last['session_identity']
    assert arrival['room']==last['room']==0 and [arrival['x'],arrival['y']]==[33051,32337]
    assert [last['x'],last['y']]==j['walk_target'] and first['character']==arrival['character']==last['character']
    assert before['selected_character']==after['selected_character'] and before['inventory']['owner']==after['inventory']['owner'] and before['inventory']['items']==after['inventory']['items']
    assert after['joined_room'] is None and after['channel'] is None and after['rooms']==[]
    assert j['old_cue_deliveries']==0 and j['old_surface_rejected'] and j['duplicate_tree_rejected'] and j['tree_request_hex']=='141002'
    assert j['held_cue_outcomes']==[dict(first='Stale',duplicate='AlreadyConsumed')]
    own_row=walk['world']['actors'][str(own)]
    assert [own_row['x'],own_row['y']]==j['walk_target'] and walk['world']['local_corrections']==after['world']['local_corrections']
    src=j['source_audio'];dst=j['destination_audio'];held=src['held'];event=held['row']['event']
    assert src['deferred'] and not dst['deferred'] and event['actor']==own and event['sound']==109 and event['action']==0
    assert src['own_deliveries_after']==src['own_deliveries_before']
    assert dst['own_deliveries_after']==dst['own_deliveries_before']+1
    for audio in [src,dst]:
        assert audio['request_hex']==(bytes.fromhex('14 00 00')+own.to_bytes(4,'little')).hex()
        before=audio['before'];immediate=audio['immediately_after_queue'];after=audio['after_ack']
        assert before['inventory']==immediate['inventory'] and before['world']['hp']==immediate['world']['hp'] and before['world']['mp']==immediate['world']['mp']
        assert after['world']['own_cast_packets']==before['world']['own_cast_packets']+1
        assert any(e['graphics']==11722 and e['target'][0]==own for e in after['world']['effects'])
    delivery=dst['own_delivery_events'][-1]
    assert delivery['row']['event']['actor']==own and delivery['row']['event']['sound']==109 and delivery['outcome'].startswith('Played(Started')
    assert delivery['epoch']==j['new_epoch']
    shutdown=read(runtime/(role+'-shutdown.json'));assert shutdown['client_quiescent'] and shutdown['view_quiescent'] and shutdown['error'] is None and shutdown['close_error'] is None
    for label in ['source','destination']:
        p=Path(j[label+'_picture']['png'])
        with Image.open(p) as im:
            im.load();assert im.size==(800,600) and im.mode=='RGBA';pixels=hashlib.sha256(im.tobytes()).hexdigest().upper()
        pictures[str(p)]=dict(sha256=sha(p),rgba_sha256=pixels)
    owners[role]=dict(held_event=event,first_submit='Stale',repeat='AlreadyConsumed',old_cue_deliveries=0,new_own_cue_deliveries=1)
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.CloseHandle.argtypes=[wintypes.HANDLE];k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
identities=[]
for j in [read(run/'runner-identity.json')]+c['process_history']:
    if 'exit_code' in j:assert j['exit_code']==0 and not j.get('cleanup_stopped_original_owned_handle')
    h=k.OpenProcess(0x100000|0x1000,False,j['pid'])
    if not h:assert ctypes.get_last_error()==87;observation='PID absent'
    else:
        try:
            ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t) for t in ts]);ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
            born=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
            reused=abs((born-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
            assert reused or k.WaitForSingleObject(h,0)==0;observation='PID reused' if reused else 'original signalled'
        finally:k.CloseHandle(h)
    identities.append(dict(pid=j['pid'],creation_utc=j['creation_utc'],original_terminal=True,observation=observation))
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'reader.py').write_bytes(Path(__file__).read_bytes())
result=dict(status='independently_qualified_same_connection_arena_WORLD_audio_and_authority',run=str(run),original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,profile=c['profile'],source_manifest_sha256=c['source_manifest_sha256'],product_sha256=c['product_sha256'],actual_sessions=4,packets=packets,owner_audio=owners,pictures=pictures,raw_pins=pins,recorded_identities=identities,GUI_verified=False,physical_audio_verified=False,C72_audio_handoff_verified=False,authority_continuity_verified=True,whole_G06_complete=False,whole_goal_complete=False)
raw_files={}
for label,folder in [('run',run),('runtime',runtime),('build',build)]:
    for p in folder.rglob('*'):
        if not p.is_file() or p.is_relative_to(out):continue
        assert p.resolve().is_relative_to(ROOT) and not p.is_symlink() and p.stat().st_size<32*1024**2
        raw_files[label+'/'+p.relative_to(folder).as_posix()]=p
hashes={n:sha(p)for n,p in raw_files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,p in raw_files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    assert set(z.namelist())==set(hashes)
    for n,h in hashes.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
result.update(reference_jar_sha256=c['reference_jar_sha256'],normal_sha256=normal['normal']['sha256'],
    normal_source_bodies_unchanged=True,source_overlays_used=False,diagnostic_API_extension=False,
    initial_repository_fixture=c['initial_repository_fixture'],passive_same_GAME_authority_and_walk_verified=True,
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,recorded_owned_all_terminal=True,
    normal_GUI_permission_inferred=False,normal_EXE_runtime_inferred=False)
(out/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=4,owners=2,profile=c['profile']),ensure_ascii=False))
