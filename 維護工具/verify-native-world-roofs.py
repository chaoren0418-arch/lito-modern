"""Independent S32 Layer5 lookup and roof clock replay for the current renderer."""
import datetime,hashlib,json,struct
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'驗證紀錄/current/native-world-roofs';SOURCE=Path(r'D:\lineage開發\0-客戶端\LiTo')
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def pin(p):return {'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)}
def metadata(plan):
 triggers={};groups=set();outside=0;records=0
 for s in plan['sectors']:
  p=SOURCE/'LTmap'/str(s['map'])/f'{s["x"]:04x}{s["y"]:04x}.s32';b=p.read_bytes();assert len(b)==s['bytes'] and sha(p)==s['sha256']
  at=32768;n=struct.unpack_from('<H',b,at)[0];at+=2+6*n+16384;ng=struct.unpack_from('<i',b,at)[0];at+=4;assert 0<=ng<=65536
  for _ in range(ng):gid,n=struct.unpack_from('<hH',b,at);at+=4+7*n
  n=struct.unpack_from('<i',b,at)[0] if at<len(b) else 0;at+=4;assert 0<=n<=262144
  ox=32768+(s['x']-32768)*64;oy=32768+(s['y']-32768)*64
  for _ in range(n):
   x,y,g,kind=struct.unpack_from('<BBHB',b,at);at+=5;records+=1
   if x>=128 or y>=64:outside+=1;continue
   assert g<ng and kind in (0,1)
   key=(s['x'],s['y'],g);groups.add(key);triggers.setdefault((2*ox-32768+x,oy+y),{})[key]='Half' if kind==0 else 'Hidden'
 return triggers,{'trigger_cells':len(triggers),'controlled_groups':len(groups),'ignored_outside_sector':outside},records
class Replay:
 def __init__(self,triggers):self.triggers=triggers;self.identity=None;self.groups={};self.last=None
 def apply(self,report):
  identity=report['identity'];now=report['elapsed_ns'];x,y=report['cell'];assert len(identity)==3 and all(identity)
  if self.identity!=identity:self.groups={};self.last=None;self.identity=identity
  assert self.last is None or now>=self.last;self.last=now
  assert not report['native_timer_queue_equivalent'] and not report['native_pointer_alias_equivalent']
  targets=self.triggers.get((2*x-32768,y),{})
  for g in targets:self.groups.setdefault(g,[['Native','Native',now],['Half','Half',now]])
  for g,ramps in self.groups.items():
   for base,r in enumerate(ramps):
    for _ in range(min(9,(now-r[2])//50_000_000)):
     if not isinstance(r[0],int):break
     w=r[0];destination={'Hidden':0,'Half':16,'Native':31}[r[1]]
     settle=(r[1]=='Half' and 13<=w<=20) or (r[1]=='Hidden' and w<4) or (r[1]=='Native' and w>27)
     r[0]=r[1] if settle else w+(4 if destination>w else -4);r[2]+=50_000_000
    target=targets.get(g,'Half' if base else 'Native')
    if target!=r[1]:
     r[1]=target
     if r[0]!=target:
      if r[0]=='Native':r[0]=31
      elif r[0]=='Half':r[0]=17 if target=='Native' else 16
      elif r[0]=='Hidden':r[0]=1
     r[2]=now
  self.groups={g:r for g,r in self.groups.items() if g in targets or r[0][0]!='Native' or r[1][0]!='Half'}
  def value(v):return {'Weight':v} if isinstance(v,int) else v
  expected=[{'sector':[g[0],g[1]],'ordinal':g[2],'normal':value(r[0][0]),'base_half':value(r[1][0])} for g,r in sorted(self.groups.items())]
  assert report['groups']==expected,(now,expected,report['groups'])
def main():
 exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';eh=sha(exe);ref=read('驗證紀錄/current/reference-snapshot.json');jh=ref['sha256'];assert jh==sha(Path(ref['source']))==sha(Path(ref['snapshot']))
 r=read('LiTo381-Modern/target/current/verification/native-roof-reference.json');h=read('LiTo381-Modern/target/current/verification/native-roof-hidden-reference.json')
 pngs=[];definitions=[]
 for sample,count in [(r,8),(h,6)]:
  triggers,definition,records=metadata(sample['plan']);assert definition==sample['definition'];definitions.append(definition)
  for a in sample['plan']['archives']:
   p=SOURCE/{'TileIndex':'LTtile.idx','TilePak':'LTtile.pak','SpriteIndex':'LTsprite.idx','SpritePak':'LTsprite.pak'}[a['role']];assert p.stat().st_size==a['bytes'] and sha(p)==a['sha256']
  assert sample['pixel_comparisons']==sample['owner_comparisons']==len(sample['cases'])==count and not sample['decoder_independent'] and not sample['rgb565_native_parity']
  replay=Replay(triggers)
  for c in sample['cases']:
   assert c['roof']['geometry_sha256'].upper()==sample['plan']['canonical_sha256'];replay.apply(c['roof']);p=Path(c['png']).resolve();p.relative_to(ROOT)
   with Image.open(p) as im:assert im.mode=='RGBA' and im.size==(320,240) and hashlib.sha256(im.tobytes()).hexdigest()==c['rgba_sha256']
   pngs.append(pin(p))
 triggers,definition,records=metadata(r['plan']);assert records==390
 cases=[];active=0;frames_total=0
 for path in ['native-combat/receipt.json','native-item-actions/receipt.json','native-reagent-skills/receipt.json','native-ground-pickup/receipt.json','native-combat/skills-receipt.json','native-combat/attack-skills-receipt.json','native-combat/ground-skills-receipt.json']:
  x=read('驗證紀錄/current/'+path);assert x['exe_sha256']==eh and x['jar_sha256']==jh
  for case in x['cases']:
   combat=case['client'].get('combat')
   if combat is None:assert case['mode'] in ['inventory-actions','inventory-actions-refused'];continue
   replay=Replay(triggers);frames=combat['frames'];assert frames
   for f in frames:
    assert f['map_id']==r['plan']['map_id'] and f['roof_definition']==definition and f['fragment_composition']['roof_runtime']
    assert f['roofs']['geometry_sha256']==f['geometry_sha256'] and f['geometry_sha256'].upper()==r['plan']['canonical_sha256']
    own=f['world']['actors'][str(f['local_object_id'])];roof=f['roofs']
    assert roof['identity']==[f['world_generation'],f['local_object_id'],own['incarnation']] and roof['cell']==[own['x'],own['y']] and roof['elapsed_ns']==f['elapsed_ns']
    replay.apply(roof);frames_total+=1;active+=bool(roof['groups'])
   cases.append({'mode':case['mode'],'frames':len(frames)})
 assert len(cases)==20
 evidence=read('驗證紀錄/current/native-world-roofs/native-roof-raw.json');assert sha(Path(evidence['native_image']['path']))==evidence['native_image']['sha256']
 result={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':eh,'jar_sha256':jh,'independent_pixel_comparisons':14,'per_pixel_owner_comparisons':14,'reference':{'half':r,'hidden':h},'definitions':definitions,'pngs':pngs,'definition':definition,'layer5_records':records,'runtime_cases':cases,'independent_clock_replayed_frames':frames_total,'runtime_active_roof_frames':active,'native_evidence':pin(OUT/'native-roof-raw.json'),'gui_tested':False,'audio_device_tested':False,'native_timer_queue_equivalent':False,'native_pointer_alias_equivalent':False,'full_native_render_equivalence':False}
 (OUT/'roofs-receipt.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({'frames':frames_total,'active_roof_frames':active,'definition':definition,'pixel_comparisons':14}))
if __name__=='__main__':main()
