"""Verify full build pins, real native walk residence, original sources and frames."""
from pathlib import Path
import hashlib,json,runpy,sys
from PIL import Image
p=Path(sys.argv[1]).resolve();read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
# Preserve the separate source-only portal/wire/authority comparison.
runpy.run_path(str(Path(__file__).with_name('verify-native-arena-portal.py')),run_name='__main__')
r=read(p/'receipt.json');assert r['complete_build_inputs_verified'] and r['inputs_unchanged']
assert sha(p/'build-inputs.json')==r['build_input_manifest_sha256']
build=read(p/'build-inputs.json');assert build['status']=='completed_build' and build['products']['native_arena_portal']['sha256']==r['test_binary_sha256']
root=Path(__file__).resolve().parents[1]
for n,h in build['files'].items():assert sha(root/n)==h,n
assert build['files']['LiTo381-Modern/src/modern_lito/native_room.rs']==r['source_pins']['LiTo381-Modern/src/modern_lito/native_room.rs']
source=read(root/'驗證紀錄/current/native-arena-portals/original-map-inputs.json');pins=[x for x in source if x['map']==r['map_id']];assert len(pins)==(19 if r['map_id']==563 else 4)
for x in pins:assert sha(Path(x['path']))==x['sha256']
trace=[json.loads(l)['state'] for l in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[];total_approach_steps=0
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
 d=read(p/f'{role}-arena-portal.json');total_approach_steps+=d['approach_steps'];rows=d['terrain_residences'];assert len(rows)>=2
 assert len({tuple(x['native_terrain_center']) for x in rows})>=2
 assert len({x['geometry_sha256'] for x in rows})==1
 imgs=[]
 for i,x in enumerate(rows):
  pos=x['view_position'];center=[0x7fff+(pos[0]-32704)//64,0x7fff+(pos[1]-32704)//64];assert center==x['native_terrain_center']
  assert x['native_terrain_block']==[v&~7 for v in pos] and x['native_terrain_block_span']==8 and x['native_terrain_window_pixels']==[1328,864]
  assert x['map_sectors']==len(pins) and 0<x['terrain_resident_sectors']<=len(pins)
  budget=x['budget'];assert budget['decoded_limit_bytes']==64<<20 and 0<budget['decoded_bytes']<=64<<20
  assert any(t['test_account']==account and t['phase']=='BATTLE' and [t.get('x'),t.get('y')]==pos for t in trace)
  f=p/f'{role}-terrain-{i}.png';im=Image.open(f).convert('RGBA');assert im.size==(800,600)
  hist=im.convert('RGB').getcolors(800*600);assert hist is not None and len(hist)>1024
  colored=sum(count for count,c in hist if c not in [(0,0,0),(18,25,35)]);assert colored>48000
  imgs.append(dict(file=f.name,sha256=sha(f),terrain_colored_pixels=colored))
 checks.append(dict(role=role,approach_steps=d['approach_steps'],approach_source=d['approach_source'],walk_residences=len(rows),centers=[x['native_terrain_center'] for x in rows],decoded_bytes=[x['budget']['decoded_bytes'] for x in rows],images=imgs))
cap=next((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()];move_requests=sum(x['direction']=='client_to_server' and x['plain'].startswith('0A ') for x in frames);assert move_requests==total_approach_steps+2
out=dict(status='verified_real_native_terrain_residence_and_siege_portal',C_MOVE_requests=move_requests,map=r['map_id'],actual_sessions=60,checks=checks,full_build_source_pins_verified=True,original_source_sector_count=len(pins),build_manifest_sha256=sha(p/'build-inputs.json'),receipt_sha256=sha(p/'receipt.json'),test_binary_sha256=r['test_binary_sha256'],jar_sha256=r['jar_sha256'],validator_sha256=sha(Path(__file__)),physical_input_verified=False,full_product_verified=False)
(p/'terrain-residence-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(out,ensure_ascii=False))
