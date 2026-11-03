"""Audit body-linked actor layers against original archive bytes and scalar pixels."""
import datetime,hashlib,json,struct
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-actor-layers'
SOURCE=Path(r'D:\lineage開發\0-客戶端')
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def pin(p):return {'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)}
indexes={}
def index(archive):
 if archive not in indexes:
  b=(SOURCE/(archive+'.idx')).read_bytes();assert len(b)==4+struct.unpack_from('<I',b)[0]*28
  rows={}
  for at in range(4,len(b),28):
   name=b[at+4:at+24].split(b'\0',1)[0].decode('ascii').lower()
   assert name not in rows
   rows[name]=(struct.unpack_from('<I',b,at)[0],struct.unpack_from('<I',b,at+24)[0])
  indexes[archive]=rows
 return indexes[archive]
def selected(name):
 shard=f'Sprite{sum(name.encode("ascii"))&15:02}'
 return shard if name in index(shard) else 'Sprite'
def verify_source(s):
 name=s['member'];archive=selected(name);assert archive==s['archive']
 offset,size=index(archive)[name];assert size==s['bytes'] and 0<size<=8<<20
 with (SOURCE/(archive+'.pak')).open('rb') as f:f.seek(offset);b=f.read(size)
 assert len(b)==size and hashlib.sha256(b).hexdigest().upper()==s['sha256']
def main():
 exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';eh=sha(exe)
 ref=read('驗證紀錄/current/reference-snapshot.json');jh=ref['sha256'];assert jh==sha(Path(ref['source']))==sha(Path(ref['snapshot']))
 raw=read('驗證紀錄/current/native-actor-layers/native-submission-raw.json');assert sha(Path(raw['native_image']['path']))==raw['native_image']['sha256']
 r=read('LiTo381-Modern/target/current/verification/native-actor-layer-reference.json')
 coverage=read('LiTo381-Modern/target/current/verification/native-actor-layer-coverage.json')
 assert r['pixel_comparisons']==r['owner_surface_comparisons']==len(r['cases'])==6
 assert r['rgb555_reference'] and not r['native_rgb565_parity'] and not r['decoder_independent'] and not r['gui_tested']
 assert coverage['samples']>1000 and coverage['cache_bytes']<=64<<20 and coverage['members']<=1024
 pins={};pngs=[]
 for s in coverage['sources']+r['sources']:pins[(s['archive'],s['member'])]=s
 for name in coverage['missing_attachments']+coverage['missing_shadows']+coverage['missing_bodies']:
  assert name not in index(selected(name)),name
 assert coverage['missing_attachments']
 for c in r['cases']:
  p=Path(c['png']).resolve();p.relative_to(ROOT)
  with Image.open(p) as im:assert im.size==(320,240) and im.mode=='RGBA' and hashlib.sha256(im.tobytes()).hexdigest()==c['rgba_sha256']
  pngs.append(pin(p))
 old=read('驗證紀錄/20261012/native-world-layers-build-receipt.json')['native_world_layers']['reference']
 current=read('LiTo381-Modern/target/current/verification/native-layer-reference.json')
 assert [c['rgba_sha256'] for c in current['cases']]==[c['rgba_sha256'] for c in old['cases']]
 cases=[];layer_counts={'attachment':0,'shadow':0}
 for path in ['native-combat/receipt.json','native-item-actions/receipt.json','native-reagent-skills/receipt.json','native-ground-pickup/receipt.json','native-combat/skills-receipt.json','native-combat/attack-skills-receipt.json','native-combat/ground-skills-receipt.json']:
  x=read('驗證紀錄/current/'+path);assert x['exe_sha256']==eh and x['jar_sha256']==jh
  for c in x['cases']:
   combat=c['client'].get('combat')
   if combat is None:assert c['mode'] in ['inventory-actions','inventory-actions-refused'];continue
   frames=combat['frames'];assert frames
   for f in frames:
    assert f['fragment_composition']['body_linked_layers'] and not f['fragment_composition']['dynamic_draw_flags_complete']
    for a in f['actors']:
     if not a.get('rendered'):continue
     assert a['layer_error'] is None and not a['native_dynamic_draw_flags_complete']
     b=a['body_source'];assert b['member']==a['sprite_member'] and b['archive']==a['archive'];pins[(b['archive'],b['member'])]=b
     for l in a['layers']:
      assert l['subframe']==a['sprite_subframe'] and l['body_linked']
      s=l['source'];assert s['member'].rsplit('-',1)[1]==b['member'].rsplit('-',1)[1]
      assert l['mix'] in (['Shadow'] if l['kind']=='shadow' else ['Over','Screen'])
      pins[(s['archive'],s['member'])]=s;layer_counts[l['kind']]+=1
     for name in a['missing_attachments']+([a['missing_shadow']] if a['missing_shadow'] else []):assert name not in index(selected(name))
   cases.append({'mode':c['mode'],'frames':len(frames)})
 assert len(cases)==20 and all(layer_counts.values())
 for s in pins.values():verify_source(s)
 result={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':eh,'jar_sha256':jh,'independent_pixel_comparisons':6,'per_pixel_owner_comparisons':6,'unchanged_terrain_body_pixel_comparisons':8,'reference':r,'coverage':coverage,'source_pins':list(pins.values()),'pngs':pngs,'runtime_cases':cases,'runtime_layers':layer_counts,'native_evidence':pin(OUT/'native-submission-raw.json'),'gui_tested':False,'audio_device_tested':False,'full_native_render_equivalence':False,'limits':['Neutral actor flags only; original dynamic flags are not fully mapped.','RGB555 reference is not native RGB565 equivalence.','Shared SPR decoders; scalar composition is independent.','Missing attachment group is omitted while a valid shadow remains.','Native roof timer/pointer alias equivalence and native insertion history remain incomplete. Qualified effect interleaving has a separate audit.']}
 (OUT/'actor-layers-receipt.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print(json.dumps({'samples':coverage['samples'],'source_pins':len(pins),'layers':layer_counts,'pixel_comparisons':6}))
if __name__=='__main__':main()
