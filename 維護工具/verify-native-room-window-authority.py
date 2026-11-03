"""Cross-check the owned native widget run against independent formal states."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]).resolve()
def read(name):return json.loads((p/name).read_text(encoding='utf-8-sig'))
def sha(name):return hashlib.sha256((p/name).read_bytes()).hexdigest().upper()
r=read('receipt.json');assert r['status']=='verified_native_room_widget_flow'
states=[row['state'] for line in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines() if 'state' in (row:=json.loads(line))]
selected=[]
def take(label,predicate):
    start=selected[-1]['index']+1 if selected else 0
    index=next(i for i in range(start,len(states)) if predicate(states[i]))
    selected.append({'step':label,'index':index,'state':states[index]})
def room(s,name,capacity,members):
    v=s.get('room_state')
    return s['object_id']==1 and v and v['name']==name and v['capacity']==capacity and v['password_required'] and len(v['members'])==members and v['owner']=='PeerFixture(測試乙)'
take('created',lambda s:room(s,'原生操作房',8,1))
take('peer joined',lambda s:room(s,'原生操作房',8,2))
take('edited',lambda s:room(s,'原生設定房',6,2))
take('observer on',lambda s:s['object_id']==3 and s['observer'])
take('observer off',lambda s:s['object_id']==3 and not s['observer'] and s['phase']=='ROOM')
take('kicked',lambda s:room(s,'原生設定房',6,1))
take('peer lobby',lambda s:s['object_id']==3 and s['phase']=='CHANNEL' and s['room']==0)
take('owner lobby',lambda s:s['object_id']==1 and s['phase']=='CHANNEL' and s['room']==0)
for step in r['steps']:
    image=Path(step['png']);assert image.parent==p and sha(image.name)==step['png_sha256']
result={'status':'verified_room_widget_authority','receipt_sha256':sha('receipt.json'),'trace_sha256':sha('server-trace.jsonl'),'exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'ordered_states':selected,'physical_input_verified':False,'ime_verified':False,'multi_dpi_verified':False,'full_product_verified':False}
(p/'authority-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified eight ordered formal states and screenshot hashes:',p)
