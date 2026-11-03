"""Cross-check completed two-peer evidence against observed formal authority."""
import hashlib
import json
import sys
from pathlib import Path

def load(p): return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest().upper()

receipt_path=Path(sys.argv[1]).resolve()
r=load(receipt_path)
events=[json.loads(s) for s in (receipt_path.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[]
for index in range(2):
    a=r['clients']['a']['rounds'][index]
    b=r['clients']['b']['rounds'][index]
    aw=a['end']['world'];bw=b['end']['world']
    assert aw['scores']==bw['scores'] and len(aw['scores'])==2
    assert aw['battle_result']==bw['battle_result']==index
    assert sorted((s['kills'],s['deaths']) for s in aw['scores'])==[(0,1),(1,0)]
    for role,id in [('a',1),('b',3)]:
        peer=r['clients'][role];row=peer['rounds'][index];w=row['end']['world']
        assert int(peer['transport']['epoch'])==2 and peer['final_lobby']['phase']=='lobby_ready'
        assert row['end']['selected_source']=='測試乙'
        assert row['start']['world']['status']['object_id']==id
        assert all(str(i) in row['start']['world']['actors'] for i in [1,3])
        actual=[];last_phase=None
        for e in events:
            s=e['state']
            if s['object_id']==id:
                if s['phase']=='RESULT' and last_phase!='RESULT': actual.append(s)
                last_phase=s['phase']
        assert len(actual)==2, (id,actual)
        server=actual[index];own=w['actors'][str(id)]
        assert server['hp']==w['hp'][0] and server['mp']==w['mp'][0]
        assert [server['x'],server['y']]==[own['x'],own['y']]
        assert server['room']==row['end']['joined_room']['id'] and server['source']=='測試乙'
        other_world=bw if role=='a' else aw
        remote=other_world['actors'][str(id)]
        assert [remote['x'],remote['y'],remote['dead']]==[own['x'],own['y'],own['dead']]
        score=next(s for s in w['scores'] if s['name']==row['end']['selected_character'])
        won=(role=='a')==(index==0)
        assert (score['kills'],score['deaths'])==((1,0) if won else (0,1))
        assert own['dead']==(not won)
        assert w['death_confirmations_sent']==(0 if won else 1)
        assert row['attacks_sent']==w['own_attack_packets'], 'every sent attack must be acknowledged'
        assert not won or row['moves_sent']>0
        assert row['start']['world']['generation']==index+1
        images=[]
        for image in [row['start']['render'],row['render']]:
            p=Path(image['png']);assert p.is_file()
            assert image['frame']['map_id']==w['map_id']==564
            images.append({'path':str(p),'sha256':sha(p)})
        checks.append(dict(role=role,round=index+1,actor=id,hp=w['hp'],mp=w['mp'],
                           moves=row['moves_sent'],attacks=row['attacks_sent'],winner=won,images=images))
report=dict(status='verified_cross_client_and_server_authority',jar_sha256=r['jar_sha256'],
    exe_sha256=r['exe_sha256'],receipt_sha256=sha(receipt_path),
    trace_sha256=sha(receipt_path.parent/'server-trace.jsonl'),checks=checks,
    gui_verified=False,full_gameplay_verified=False)
(receipt_path.parent/'authority-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified two clients against server: identities, force results, HP/MP, coordinates, death, scores, ACKs, reentry and 8 images.')
