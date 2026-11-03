import hashlib
"""Compare room operations with passive observations of the formal GameHub."""
import json, sys
from pathlib import Path
p=Path(sys.argv[1]); r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='room-admin' and r['source_matches_snapshot']
trace=[json.loads(line)['state'] for line in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
clients=r['clients']; checked=[]
for role,c in clients.items():
    assert c['status']=='real_peer_room_admin_completed' and c['worker_quiescent'] and c['shutdown']=='closed'
    stages={row['stage']:row['state'] for row in c['stages']}
    expected={'joined','edited','observer-on','observer-off','owner-transfer','rejoined','kicked','finished'}
    if role=='a':expected|={'created','kick-rejoin-rejected'}
    else:expected|={'wrong-password-rejected','ready-before-edit'}
    assert expected<=stages.keys(), (role,stages.keys())
    for label,s in stages.items():
        room=s['joined_room']
        if room is None:continue
        # Stage barriers keep accepted snapshots stable long enough for passive sampling.
        peers=sorted(m['name'] for m in room['members'])
        matching=[t for t in trace if t['object_id']==(1 if role=='a' else 3) and t.get('room_state') and
            all(t['room_state'][k]==room[k] for k in ['id','name','capacity','password_required','owner']) and
            sorted(t['room_state']['members'])==peers and t['observer']==s['observer']]
        assert matching,(role,label,'no corresponding formal room snapshot')
        checked.append({'role':role,'stage':label,'room':room['id'],'authority_samples':len(matching)})
    assert stages['joined']['joined_room']['password_required']
    edit=stages['edited']['joined_room'];assert edit['name']=='整合修改房' and edit['capacity']==8 and not edit['password_required']
    assert all(m['state']==0 for m in edit['members'])
    assert stages['observer-on']['observer']==(role=='b') and not stages['observer-off']['observer']
    assert stages['owner-transfer']['joined_room']['owner']==clients['b']['character'] if role=='b' else stages['owner-transfer']['joined_room'] is None
    if role=='a':assert stages['kick-rejoin-rejected']['room_result'] not in (None,0)
    else:
        assert stages['wrong-password-rejected']['room_result'] not in (None,0)
        assert stages['kicked']['room_kick_result'] is True
    assert stages['finished']['joined_room'] is None
assert r['server']['memberships_empty'] and r['server']['authority_mutations']==0
result={'status':'verified','scope':'room-service subset','checks':checked,'gui_verified':False,'ordinary_world_verified':False}
result['receipt_sha256']=hashlib.sha256(p.read_bytes()).hexdigest()
(p.parent/'room-admin-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':'verified','room_snapshots':len(checked)}))
