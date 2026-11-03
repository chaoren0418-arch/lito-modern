"""Verify actual owner map buttons, both visible room snapshots and C_D1 wire."""
from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]);read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='verified_native_teleport_GUI_cross_endpoint_shared_battle'
m=r['room_map_selection'];assert m==dict(owner='a',next=514,restored=564,previous_control=136,next_control=137,peer_controls_disabled=True,visible=True)
cap=next((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()];packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
requests=[b for x,b in packets if x['direction']=='client_to_server' and b[:2]==bytes([0xd1,3])];assert requests==[bytes([0xd1,3,2,2]),bytes([0xd1,3,0x34,2])]
# S_D5 type3: room u32 then map u16. Both windows must receive each change.
replies=[b for x,b in packets if x['direction']=='server_to_client' and b[:2]==bytes([0xd5,3])]
assert all(len(b)==8 for b in replies)
assert sum(int.from_bytes(b[-2:],'little')==514 for b in replies)==2
assert sum(int.from_bytes(b[-2:],'little')==564 for b in replies)>=2
for role in ['a','b']:
 for name,mapid in [('map-next',514),('map-restored',564)]:
  shot=next(s for s in r['steps'] if s['role']==role and s['step']==name);assert '地圖 '+str(mapid) in shot['status'];assert sha(Path(shot['png']))==shot['png_sha256']
out=dict(status='verified_native_GUI_owner_map_roundtrip',exe_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],C_D1_requests=2,authoritative_next_map=514,restored_before_battle=564,owner_buttons_visible=True,peer_buttons_disabled=True,private_desktop=True,physical_input_verified=False,receipt_sha256=sha(p/'receipt.json'),capture_sha256=sha(cap),validator_sha256=sha(Path(__file__)))
(p/'room-map-gui-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(out))
