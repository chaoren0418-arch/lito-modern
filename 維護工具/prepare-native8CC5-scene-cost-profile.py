"""Select actual source-world packet bytes for a resource-only cost measurement."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
runtime=P/'target/current/native-teleport-failure/20261018-120502-749'
capture=next((runtime/'captures').glob('*/frames.jsonl'));rows=[json.loads(line) for line in capture.read_text('utf-8-sig').splitlines()]
raw=[(i+1,bytes.fromhex(row['plain'])) for i,row in enumerate(rows) if row['direction']=='server_to_client' and re.fullmatch('[a-fA-F0-9 ]*',row.get('plain',''))]
roles={}
for role in ['a','b']:
    journal=runtime/(role+'-presentation-journal.json');j=read(journal);w=j['last_snapshot']['world'];own=w['status']['object_id']
    chosen=[]
    def select(label,predicate):
        candidates=[(line,body) for line,body in raw if predicate(body)];assert candidates,(role,label)
        line,body=candidates[-1];chosen.append(dict(label=label,capture_line=line,hex=body.hex()));return body
    select('world',lambda b:len(b)==4 and b[0]==0x28 and int.from_bytes(b[1:3],'little')==w['map_id'] and b[3]==w['underwater'])
    select('start',lambda b:b==b'\xd2')
    select('status',lambda b:len(b)>5 and b[0]==0x0c and int.from_bytes(b[1:5],'little')==own)
    for identity,actor in w['actors'].items():
        select('actor-'+identity,lambda b,a=actor:len(b)>=11 and b[0]==0x0b and int.from_bytes(b[5:9],'little')==a['id'] and int.from_bytes(b[1:3],'little')==a['x'] and int.from_bytes(b[3:5],'little')==a['y'] and int.from_bytes(b[9:11],'little')==a['graphics'])
    for opcode,key in [(0x0d,'hp'),(0x4d,'mp')]:
        value=w[key]
        if value is not None:
            payload=bytes([opcode])+b''.join(n.to_bytes(2,'little') for n in value)
            select(key,lambda b,payload=payload:b==payload)
    def skills(b):return sorted(i*8+bit+1 for i,mask in enumerate(b[2:]) for bit in range(8) if mask&(1<<bit))
    select('book',lambda b:len(b)>=2 and b[0]==0x1e and len(b)==b[1]+2 and skills(b)==w['learned_skills'])
    if w['game_time'] is not None:select('time',lambda b:len(b)==5 and b[0]==0x21 and int.from_bytes(b[1:],'little')==w['game_time'])
    selected_keys=['id','x','y','graphics','heading','walk_action','light','speed','brave','name','title','status','draw_state']
    roles[role]=dict(packets=chosen,expected_actors={identity:{key:actor[key] for key in selected_keys} for identity,actor in w['actors'].items()},
        expected_own_id=own,expected_map=w['map_id'],expected_hp=w['hp'],expected_mp=w['mp'],expected_minimap=w['minimap'],
        map_mask=j['last_snapshot']['environment']['map_mask_argument'],expected_actor_count=len(w['actors']),
        origin=dict(journal=str(journal),journal_sha256=sha(journal)))
minimap=sorted({body.hex() for _,body in raw if body[:1]==b'\xe8'})
normal=V/'native-legacy-audio-normal/validation-20261018-104256-823814';c=read(normal/'cohort.json')
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
out=V/'native8CC5-scene-cost'/('input-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
record=dict(status='prepared_uncompiled_resource_cost_input',source_manifest_sha256=c['source_manifest_sha256'],
    capture=dict(path=str(capture),sha256=sha(capture)),roles=roles,minimap_candidates=minimap,
    scope='Captured final source actor layouts (A15/B14), map/status/book bytes; fixed elapsed0 and full-bright map binding. This measures Model load/draw costs, not the original live scheduling or environment timeline.',
    acceptance_ready_bound_seconds=8,normal_product_source_modified=False,new_sessions=0,new_requests=0,physical_audio_verified=False,whole_goal_complete=False)
(out/'input.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(input=str(out/'input.json'),actors={role:v['expected_actor_count'] for role,v in roles.items()},minimap_candidates=len(minimap),compiled=False),ensure_ascii=False))
