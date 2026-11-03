"""Inspect exact original world PNG RGB pixels for actual scene presentation."""
from pathlib import Path
from PIL import Image
import argparse,json,hashlib,shutil
root=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);a=ap.parse_args();case=a.case.resolve(strict=True);assert case.is_relative_to(root/'驗證紀錄/current/native-teleport-gui')
r=read(case/'receipt.json');assert r['status'].startswith('verified_native_teleport_GUI_') and r['inputs_unchanged']
names=['town','new-battle'] if r['scenario']=='cross-endpoint' else ['town','reentered','recovery-moved'];shots=[x for x in r['steps'] if x['step'] in names];assert len(shots)==2*len(names)
checks=[]
for shot in shots:
 p=Path(shot['png']);assert sha(p)==shot['png_sha256'];im=Image.open(p).convert('RGB');w,h=im.size;region=im.crop((round(w*.2),round(h*.2),round(w*.8),round(h*.65)));assert region.getcolors(1024) is None,('world image lacks terrain diversity',p)
 colored=0
 for gy in range(8):
  for gx in range(16):
   c=im.getpixel((round(w*(.25+.5*gx/15)),round(h*(.25+.25*gy/7))));colored+=max(c)>20 and max(c)-min(c)>15
 assert colored>=10,('world RGB probe below bound',p,colored)
 if shot.get('terrain_paint_probe'):assert shot['terrain_paint_probe'][-1]>=10
 checks.append({'role':shot['role'],'step':shot['step'],'PNG':p.name,'sha256':sha(p),'world_region_over1024_colors':True,'colored_probes':colored,'capture_paint_probes':shot.get('terrain_paint_probe'),'capture_settle_ms':shot.get('scene_settle_ms')})
out={'status':'verified_actual_original_world_scene_RGB_pixels','main_sha256':r['exe_sha256'],'scenario':r['scenario'],'receipt_sha256':sha(case/'receipt.json'),'checks':checks,'images':len(checks),'validator_sha256':sha(Path(__file__)),'capture_waits_are_diagnostic_not_G11':True,'physical_input_verified':False,'full_game_complete':False};(case/'world-scene-pixel-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');dest=case/'validation-sources'/(sha(Path(__file__))[:16]+'-'+Path(__file__).name);dest.parent.mkdir(exist_ok=True)
if not dest.exists():shutil.copy2(Path(__file__),dest)
else:assert sha(dest)==sha(Path(__file__))
print(json.dumps({'status':out['status'],'scenario':r['scenario'],'images':len(checks)}))