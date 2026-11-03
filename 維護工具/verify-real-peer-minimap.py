"""Current E8 rendering compared to independently decoded formal TCP captures."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]);p=p/'receipt.json' if p.is_dir() else p
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
r=read(p);assert r['status']=='verified_real_peer_action'
profile=r['clients']['a']['action_profile'];assert profile in ['movement','rental-shop']
wire=set();capture_pins=[]
for f in Path(r['runtime']).glob('captures/*/frames.jsonl'):
 capture_pins.append({'path':str(f),'sha256':sha(f)})
 for line in f.read_text(encoding='utf-8-sig').splitlines():
  event=json.loads(line)
  if event['direction']!='server_to_client':continue
  b=bytes.fromhex(event['plain'])
  if b[0]!=0xe8:continue
  assert len(b)==13 and b[5:7]==b'\0\0' and b[11]==1 and b[12] in [0,1]
  wire.add((int.from_bytes(b[1:5],'little'),int.from_bytes(b[7:9],'little'),int.from_bytes(b[9:11],'little'),bool(b[12])))
assert len(wire)>=2
checks=[];images=[]
for role,c in r['clients'].items():
 own=1 if role=='a' else 3;other=3 if role=='a' else 1
 if profile=='movement':
  rows=[(f'stage-{i}',row['world'],row['minimap_frame']) for i,row in enumerate(c['world_probe']['stages'])]
  assert len(rows)==35
 else:rows=[('end',c['end']['world'],c['render']['frame']['minimap'])]
 for name,w,mini in rows:
  assert mini['visible'] and mini['map_id']==w['map_id'] and mini['generation']==w['generation']
  assert mini['rect']==[610,10,180,180] and mini['geometry']=='original LTmap directional edges'
  assert mini['terrain_cells']>100 and not mini['click_through'] and not mini['dynamic_obstacles_shown']
  assert mini['own_id']==own and mini['center']==[w['actors'][str(own)]['x'],w['actors'][str(own)]['y']]
  assert 232 not in w['unhandled_opcodes'] and 0<len(w['minimap']['markers'])<=2048
  for m in w['minimap']['markers'].values():assert (m['id'],m['x'],m['y'],m['allied']) in wire
  for m in mini['markers']:
   assert m['id']!=own and (m['id'],*m['world'],m['allied']) in wire
   assert m['allied']==(profile=='rental-shop')
   dx=m['world'][0]-mini['center'][0];dy=m['world'][1]-mini['center'][1]
   assert m['pixel']==[700+(dx-dy)*2,101+dx+dy]
  if profile=='movement':
   row=c['world_probe']['stages'][int(name.split('-')[1])]
   if row['intent']['subject']==other:
    marker=next(m for m in mini['markers'] if m['id']==other)
    assert marker['world']==row['intent']['position']
  checks.append({'role':role,'stage':name,'markers':mini['markers'],'center':mini['center']})
 if profile=='movement':
  assert any(x['markers'] for x in checks if x['role']==role)
  for row in c['world_probe']['stages']:
   if 'png' in row['render']:
    f=Path(row['render']['png']);assert f.exists();images.append({'path':str(f),'sha256':sha(f)})
 else:
  assert any(m['id']==other and m['allied'] for m in mini['markers'])
  f=Path(c['render']['png']);images.append({'path':str(f),'sha256':sha(f)})
report={'status':'verified_native_minimap_wire_and_render_subset','profile':profile,'receipt_sha256':sha(p),
 'exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'capture_pins':capture_pins,
 'unique_wire_markers':len(wire),'checks':checks,'images':images,'gui_physical_input_verified':False,
 'hidden_delete_portal_coverage':'separate deterministic Rust tests; not live invisibility gameplay'}
(p.parent/'minimap-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified minimap:',profile,len(checks),'snapshots,',len(wire),'independent wire markers')
