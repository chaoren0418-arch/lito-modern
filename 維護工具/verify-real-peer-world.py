"""Compare both native views with independent formal positions and door state."""
import hashlib,json,sys
from pathlib import Path
def load(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
p=Path(sys.argv[1]).resolve();r=load(p);profile=r['action_profile'];assert profile in ['doors','movement']
events=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
total=14 if profile=='doors' else 35
clients=r['clients'];checks=[];images=[]
for role,id in [('a',1),('b',3)]:
    c=clients[role];probe=c['world_probe'];assert c['worker_quiescent'] and c['shutdown']=='closed'
    assert len(probe['stages'])==total and int(c['transport']['epoch'])==2
    assert c['start']['world']['status']['object_id']==c['end']['world']['status']['object_id']==id
    assert probe['moves_sent']==c['end']['world']['predicted_moves']
    assert c['end']['world']['local_corrections']==0 and c['end']['world']['own_attack_packets']==0
    assert probe['toggles_sent']==(2 if profile=='doors' else 0)
    assert probe['blocked_requests']==(2 if profile=='doors' else 0)
    for image in [c['start']['render'],c['render']]+[s['render'] for s in probe['stages'] if 'png' in s['render']]:
        path=Path(image['png']);assert path.is_file();images.append(dict(path=str(path),sha256=sha(path)))
for index in range(total):
    a=clients['a']['world_probe']['stages'][index];b=clients['b']['world_probe']['stages'][index]
    assert a['intent']==b['intent'];intent=a['intent'];assert intent['stage']==index
    for role,id,row,other in [('a',1,a,b),('b',3,b,a)]:
        w=row['world'];own=w['actors'][str(id)];remote=other['world']['actors'][str(id)]
        assert [own[k] for k in ['x','y','heading','dead']]==[remote[k] for k in ['x','y','heading','dead']]
        source=[s for s in events if s['object_id']==id and s['phase']=='BATTLE' and s['source']=='測試乙'
            and [s['x'],s['y'],s['heading']]==[own['x'],own['y'],own['heading']]]
        assert source,(profile,index,role,'unmatched server position')
        subject=w['actors'][str(intent['subject'])];pose=row['subject_frame']
        assert [subject['x'],subject['y']]==intent['position']==pose['world'] and pose['rendered']
        if intent['kind']=='toggle':
            assert pose['phase']=='object_action' and pose['action_complete'] and pose['action']==intent['expected_action']
            assert subject['walk_action']==intent['expected_action']
            assert any(s.get('door') and s['door']['id']==intent['subject']
                and s['door']['action']==intent['expected_action'] and s['door']['passable']==intent['expected_passable'] for s in source)
        else:assert pose['phase']=='idle' and pose['heading']==intent['heading']
    if profile=='movement' and index<32:
        base=(index%16)//2;expected=base if index%2==0 else (base+4)%8
        assert intent['heading']==expected and intent['active']==('a' if index<16 else 'b')
    checks.append(dict(stage=index,kind=intent['kind'],active=intent['active'],subject=intent['subject'],position=intent['position']))
overlap=None
if profile=='movement':
    row=clients['a']['world_probe']['stages'][33];w=row['world'];x=w['actors']['1'];y=w['actors']['3']
    assert [x['x'],x['y']]==[y['x'],y['y']];overlap=True
report=dict(status='verified_real_peer_world_authority',action_profile=profile,jar_sha256=r['jar_sha256'],exe_sha256=r['exe_sha256'],
    receipt_sha256=sha(p),trace_sha256=sha(p.parent/'server-trace.jsonl'),checks=checks,images=images,
    player_overlap_authoritatively_allowed=overlap,gui_verified=False,all_world_lifecycles_verified=False)
# Independently decode actual server output; bootstrap companion IDs are allocated
# at runtime and must not be guessed from the player ID or a display name.
masters={};bounds=set();capture_pins=[]
for capture in Path(r['runtime']).glob('captures/*/frames.jsonl'):
    capture_pins.append(dict(path=str(capture),sha256=sha(capture)))
    for line in capture.read_text(encoding='utf-8-sig').splitlines():
        event=json.loads(line)
        if event['direction']!='server_to_client':continue
        body=bytes.fromhex(event['plain'])
        if body[0]==0x4f:
            assert body[-1]==0
            id=int.from_bytes(body[1:5],'little');name=body[5:-1].decode('cp950')
            assert id not in masters or masters[id]==name
            masters[id]=name
        elif body[0]==0xe2 and len(body)==9:
            bounds.add(tuple(int.from_bytes(body[i:i+2],'little') for i in [1,3,5,7]))
assert len(masters)==2 and len(bounds)==1
expected=next(iter(bounds))
for c in clients.values():
    w=c['end']['world'];area=w['observer_bounds']
    assert tuple(area['min']+area['max_exclusive'])==expected
    assert not set([0x4f,0xe2,0x22]).intersection(w['unhandled_opcodes'])
    for id,name in masters.items():assert w['actors'][str(id)]['master']==name
report['world_metadata']=dict(masters=masters,observer_bounds=expected,capture_pins=capture_pins,observer_controls_verified=False)
(p.parent/'authority-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified two real clients and formal world authority:',profile,total,'stages')
