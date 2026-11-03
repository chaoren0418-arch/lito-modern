"""Compare controller citizenship against passive formal-session observations."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='rental-citizen' and r['source_matches_snapshot']
assert r['server']['authority_mutations']==0 and r['server']['memberships_empty']
trace=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[];gem='1811939330'
for role,c in r['clients'].items():
    own=1 if role=='a' else 3;q=c['citizen'];states=[s for s in trace if s['object_id']==own and s['rental_town']]
    assert not q['server_authority_injection'] and not q['gui_verified']
    assert c['worker_quiescent'] and c['shutdown']=='closed'
    original=q['before']['inventory']['items'];assert original[gem]['count']==20
    assert [s['npc'] for s in q['stages']]==[1812987905,1812987906,1812987907,1812987908,1812987904]
    held=set();cursor=0
    for i,s in enumerate(q['stages']):
        before=s['inventory_before']['items'];after=s['granted']['inventory']['items']
        assert after==s['duplicate']['inventory']['items']
        if i<4:
            added=set(after)-set(before);assert len(added)==1
            item=added.pop();assert after[item]['count']==1 and before=={k:v for k,v in after.items() if k!=item}
            held.add(item);assert after[gem]['count']==20
        else:
            assert set(before)-set(after)==held and set(after)==set(original)
            assert after[gem]['count']==21
        for step in s['steps']:
            assert max(abs(a-b) for a,b in zip(step['before'],step['after']))==1
            match=next((j for j in range(cursor,len(states)) if [states[j]['x'],states[j]['y']]==step['after']),None)
            assert match is not None,(role,i,step,'formal movement absent');cursor=match
        matches=[j for j in range(cursor,len(states)) if states[j]['rental_citizen']==(i==4) and len(states[j]['rental_emblems'])==(i+1 if i<4 else 0) and states[j]['rental_gems']==(20 if i<4 else 21)]
        assert matches,(role,i,'formal reward absent');cursor=matches[0]
        image=Path(s['render']['png']);assert image.is_file()
        checks.append({'role':role,'npc':s['npc'],'walk_steps':len(s['steps']),'formal_index':cursor,'image_sha256':hashlib.sha256(image.read_bytes()).hexdigest()})
    assert q['end']['inventory']['items']==q['stages'][-1]['granted']['inventory']['items']
report={'status':'verified_original_pages_citizenship','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'checks':checks,'gui_verified':False,'all_npc_services_verified':False,'duplicate_rewards_rejected':True}
(p.parent/'rental-citizen-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':report['status'],'stages':len(checks),'steps':sum(x['walk_steps'] for x in checks)}))
