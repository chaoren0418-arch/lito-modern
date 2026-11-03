import json,hashlib
from pathlib import Path
from PIL import Image
from html.parser import HTMLParser
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r474-offline-map-flow'
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
d=json.loads((L/'r474-offline-map-details.json').read_text(encoding='utf-8'));s=json.loads((O/'player-session/session.json').read_text(encoding='utf-8'))
assert [r['command'] for r in s['history']]==['list','select 1 526','select 1 564','enter','pan 240 120','back','enter','back','quit']
assert s['history'][1]['error'] and s['history'][-1]['page']=='Menu';assert s['GUI']==0 and not s['renderer_LOADED'] and s['native_callbacks']==0
outputs=[r['image'] for r in s['history'] if r.get('image')];assert len(outputs)==3
comparisons=[]
for bmp,png in zip(outputs,d['images']):
 with Image.open(bmp) as b, Image.open(png['path']) as p:
  assert b.size==p.size==(800,600);assert b.convert('RGB').tobytes()==p.convert('RGB').tobytes()
 comparisons.append(dict(player_bmp=pin(bmp),focused_png=pin(png['path']),all_480000_RGB_pixels_identical=True))
class Review(HTMLParser):
 def __init__(self):super().__init__();self.options=[];self.ids=[];self.images=[]
 def handle_starttag(self,tag,attrs):
  a=dict(attrs)
  if tag=='option':self.options.append(a)
  if 'id' in a:self.ids.append(a['id'])
  if tag=='img':self.images.append(a['src'])
r=Review();r.feed((O/'offline-map-preview.html').read_text(encoding='utf-8'));assert len(r.options)==40;enabled=[o for o in r.options if 'disabled' not in o];assert len(enabled)==1 and enabled[0]['value']=='564';assert len(r.ids)==len(set(r.ids));assert (O/r.images[0]).exists()
result=dict(state='PLAYER_TOOL_AND_FOCUSED_OUTPUTS_PIXEL_IDENTICAL_STATIC_REVIEW_REFERENCES_VERIFIED',player_session=pin(O/'player-session/session.json'),comparisons=comparisons,review=pin(O/'offline-map-preview.html'),options=40,enabled_preview_options=1,directly_viewed_pngs=['01-entry-battle-ground.png','02-pan-battle-ground.png'],hash_and_pixels_only=['03-reentry-battle-ground.png'],HTML_browser_executed=False,GUI_acceptance=0,renderer_LOADED=False)
p=L/'r474-player-output-verification.json';assert not p.exists();p.write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(dict(paired_pixel_comparisons=3,pixels_each=480000,menu_options=40,enabled=1)))
