import json,sys,hashlib
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='rental-return' and r['source_matches_snapshot']
states=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
frames=[];pins={}
for f in sorted((Path(r['runtime'])/'captures').rglob('frames.jsonl')):
    frames.extend(json.loads(s) for s in f.read_text(encoding='utf-8-sig').splitlines());pins[str(f)]=hashlib.sha256(f.read_bytes()).hexdigest()
starts=[f for f in frames if f['direction']=='client_to_server' and f['plain']=='10']
assert len(starts)==2
start=min(f['time'] for f in starts);after=[f for f in frames if f['time']>=start]
assert len([f for f in after if f['direction']=='server_to_client' and f['plain']=='65'])==2
assert len([f for f in after if f['direction']=='client_to_server' and f['plain']=='D9'])==2
rows=[]
for role,c in r['clients'].items():
    own=1 if role=='a' else 3;s=c['return_observation'];a=s['after']
    assert c['worker_quiescent'] and c['shutdown']=='closed' and a['rental_restart_acks']==1
    if s['characters_ready']:
        assert a['phase']=='characters_ready' and a['joined_room'] is None and a['world']['map_id'] is None
        assert any(t['object_id']==own and not t['rental_town'] and t['phase']=='LOBBY' for t in states)
        outcome='returned_to_character_directory_reentry_pending'
    else:
        assert s['observed_ms']>=5900 and a['phase']=='returning_characters' and a['joined_room']['id']==60001 and a['world']['map_id']==500
        assert a['inventory']['items']==s['before']['inventory']['items']
        assert any(t['object_id']==own and t['phase']=='BATTLE' and t['rental_town'] and t['room_state'] is None and t['pending_transition']=='RESTART_AWAIT_QUIT_FIGHT' for t in states)
        outcome='restart_acknowledged_no_return_within_observation'
    rows.append({'role':role,'outcome':outcome,'observed_ms':s['observed_ms']})
if all(not c['return_observation']['characters_ready'] for c in r['clients'].values()):
    assert not any(f['direction']=='server_to_client' and f['opcode']=='0xF5' for f in after)
assert r['server']['memberships_empty'] and r['server']['authority_mutations']==0
report={'status':'observed_rental_return_boundary','rows':rows,'receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'capture_sha256':pins,'normal_return_verified':False,'reentry_verified':False,'frames_after_restart':after}
(p.parent/'rental-return-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':report['status'],'rows':rows}))
