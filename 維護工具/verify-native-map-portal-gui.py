from pathlib import Path
import json, hashlib, sys, collections
from PIL import Image
p=Path(sys.argv[1]).resolve();read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'));sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='verified_original_map_portal_native_GUI_roundtrip' and r['inputs_unchanged'] and r['reference_is_current']
assert r['server']['actual_client_sessions']==4 and r['server']['memberships_empty'] and r['server']['authority_mutations']==0
assert not r['server']['sql_access'] and not r['server']['state_persistent']
for f,h in r['source_pins'].items():assert sha(p/'source-snapshot'/f)==h
runtime=Path(r['runtime']);cap=next((runtime/'captures').glob('*/frames.jsonl'));frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==4
packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
decode=lambda b:tuple(int.from_bytes(b[n:n+2],'little') for n in range(1,len(b),2))
requests=[b for x,b in packets if x['direction']=='client_to_server' and b[0]==0x73]
assert all(len(b)==5 for b in requests) and collections.Counter(decode(b) for b in requests)=={(32771,32824):2,(32788,32719):2}
reply=[b for x,b in packets if x['direction']=='server_to_client' and b[0]==0x5a]
assert all(len(b)==7 for b in reply) and collections.Counter(decode(b) for b in reply)=={(500,32788,32719):2,(500,32771,32824):2}
assert sum(x['direction']=='client_to_server' and b==b'\x08' for x,b in packets)==4
checks=[]
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
 rows=r[role+'-portals'];assert len(rows)==2
 for i,target in enumerate([[32788,32719],[32771,32824]]):
  before=rows[i]['before'];after=rows[i]['after'];assert before['test_account']==after['test_account']==account
  assert before['character']==after['character'] and before['session_identity']==after['session_identity'] and before['object_id']==after['object_id']
  assert [after['x'],after['y']]==target and after['rental_map']==500 and after['phase']=='BATTLE'
  assert before['rental_inventory']==after['rental_inventory']
 shots=[s for s in r['steps'] if s['role']==role and s['step'].startswith('after-entry-')];assert len(shots)==2
 for shot in shots:
  assert '入口：場景已更新' in shot['status'] and sha(Path(shot['png']))==shot['png_sha256']
  im=Image.open(shot['png']).convert('RGB');colored=0
  for gy in range(8):
   for gx in range(16):
    pixel=im.getpixel((round(im.width*(0.25+0.5*gx/15)),round(im.height*(0.25+0.25*gy/7))))
    colored+=max(pixel)>20 and max(pixel)-min(pixel)>15
  assert colored>=10 and shot['terrain_paint_probe'][-1]>=10

 walks=[s for s in r['steps'] if s['role']==role and s['step']=='GUI-key-walk'];assert len(walks)==4 and walks[-1]['after']==[32769,32822]
 checks.append({'role':role,'P_key_entries':2,'source_path_key_steps':4,'same_character_and_inventory':True})
out={'status':'verified_P_key_original_map_entrance_GUI_roundtrip','checks':checks,'formal_sessions':4,'exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'receipt_sha256':sha(p/'receipt.json'),'capture_sha256':sha(cap),'validator_sha256':sha(Path(__file__)),'physical_input_verified':False,'ordinary_WORLD_entry_verified':False,'full_product_verified':False}
(p/'map-portal-gui-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({'status':out['status'],'checks':checks}))
