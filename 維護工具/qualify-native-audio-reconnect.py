"""Read raw C02 packets, owner audio events, inputs, PNGs and process identities."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-audio-reconnect')
c=read(run/'receipt.json');assert c['status']=='executed_diagnostic_pending_independent_qualification' and c['actual_sessions']==8 and c['owned_processes']==[] and c['asset_metadata_unchanged']
runtime=Path(c['runtime']);build=Path(c['compile']);b=read(build/'receipt.json');inputs=read(build/'compiled-inputs.json')
assert sha(build/'receipt.json')==c['compile_receipt_sha256'] and sha(c['product'])==c['product_sha256']==b['product_sha256']
assert sha(c['reference_jar'])==c['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(c['catalog'])==c['catalog_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for row in b['libraries'].values():assert sha(row['path'])==row['sha256']
for n,h in c['harness_class_pins'].items():assert sha(n)==h,n
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
pins={};packets={};pictures={};owners={}
for stage in ['source','destination']:
    result=read(runtime/stage/'result.json')
    assert result['actual_client_sessions']==4 and result['game_sessions']==2 and result['memberships_empty'] and result['authority_mutations']==0 and not result['sql_access']
    rows=[]
    for p in (runtime/stage/'captures').rglob('frames.jsonl'):
        pins[str(p)]=sha(p);rows.extend(json.loads(x) for x in p.read_text(encoding='utf-8-sig').splitlines())
    requests=[r for r in rows if r['direction']=='client_to_server' and r['opcode']=='0x14']
    effects=[r for r in rows if r['direction']=='server_to_client' and r['opcode']=='0x37']
    assert sorted(bytes.fromhex(r['plain']) for r in requests)==[bytes.fromhex('14 00 00')+own.to_bytes(4,'little') for own in [1,3]]
    for own in [1,3]:
        expected=bytes([0x37])+own.to_bytes(4,'little')+(11722).to_bytes(2,'little')
        assert sum(bytes.fromhex(r['plain'])==expected for r in effects)==2,(stage,own,effects)
    packets[stage]=dict(actual_heal1_requests=len(requests),actual_S37_broadcasts=len(effects),targets=[1,3],graphics=11722)
    pins[str(runtime/stage/'result.json')]=sha(runtime/stage/'result.json')
for role,own in [('a',1),('b',3)]:
    p=runtime/(role+'-result.json');j=read(p);pins[str(p)]=sha(p)
    assert j['old_epoch']!=j['new_epoch'] and j['old_peer']!=j['new_peer'] and j['old_cue_deliveries']==0 and j['old_surface_rejected']
    assert j['held_cue_outcomes']==[dict(first='Stale',duplicate='AlreadyConsumed')]
    assert all(j['closed'][n] for n in ['frame_absent','client_quiescent','view_quiescent'])
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
result=dict(status='independently_qualified_explicit_battle_cancel_reconnect_audio_only',run=str(run),original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,profile=c['profile'],source_manifest_sha256=c['source_manifest_sha256'],product_sha256=c['product_sha256'],actual_sessions=8,packets=packets,owner_audio=owners,pictures=pictures,raw_pins=pins,recorded_identities=identities,GUI_verified=False,physical_audio_verified=False,C72_audio_handoff_verified=False,authority_continuity_verified=False,whole_G06_complete=False,whole_goal_complete=False)
(out/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=8,owners=2,profile=c['profile']),ensure_ascii=False))
