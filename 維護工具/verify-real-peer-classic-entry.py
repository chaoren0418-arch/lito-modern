import hashlib
"""Record C05 reachability as an observation, without equating a reply to a scene."""
import json,sys
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='classic-entry' and r['source_matches_snapshot']
states=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
rows=[]
for role,c in r['clients'].items():
    assert c['worker_quiescent'] and c['shutdown']=='closed' and c['observation_ms']>=2000
    check=c['boundary']['classic_entry_result'];assert check is not None
    name=c['directory'][0]['name'];world=c['boundary']['world']
    assert not world['actors'] and world['map_id'] is None
    assert c['return_directory']==c['directory']
    # C05 compares CharacterSummary.name directly; native display aliases are separate identities.
    candidates=[s for s in states if s['phase']=='LOBBY' and s['source']=='測試乙' and s['available_characters']]
    assert candidates
    names=sorted({n for s in candidates for n in s['available_characters']})
    if not check['accepted']:
        assert name.casefold() not in [n.casefold() for n in names]
        assert c['boundary']['selected_character'] is None
    rows.append({'role':role,'received_identity':name,'formal_character_names':names,'reply':check,'world_received':False})
assert r['server']['memberships_empty'] and r['server']['authority_mutations']==0
result={'status':'observed_not_world_entry','rows':rows,'ordinary_world_verified':False,
 'limitation':'The native directory display identity is not the raw CharacterSummary identity expected by C05. No implicit alias rewriting was applied.'}
result['receipt_sha256']=hashlib.sha256(p.read_bytes()).hexdigest()
(p.parent/'classic-entry-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':result['status'],'peers':len(rows)}))
