"""Invoke only after the agent actually views the specified normal-client image."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,sys
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'object-gate-tests-20261016-132623-487623/normal-qualified'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
profile=sys.argv[1];assert profile in ['Debug','Release']
case=Path(sys.argv[2]).resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-gui')
n=read(NORMAL/'qualified.json');q=read(case/'current-lighting-source-qualification.json')
assert q['profile']==profile and q['source_files']==1019 and q['scenario']=='cross-endpoint'
assert q['main_sha256']==n['profiles'][profile]['sha256']
png=case/'a-new-battle.png';assert png.read_bytes().startswith(b'\x89PNG\r\n\x1a\n')
note={'status':'agent_visually_inspected_current1019_normal_cross_endpoint_battle_frame',
    'observed_utc':datetime.now(timezone.utc).isoformat(),'profile':profile,'source_files':1019,
    'case':str(case),'image':str(png),'image_sha256':sha(png),'case_qualification_sha256':sha(case/'current-lighting-source-qualification.json'),
    'main_sha256':q['main_sha256'],'map_label_visible':564,'terrain_visible':True,'actors_visible':True,
    'minimap_visible':True,'Chinese_HUD_visible':True,'scope':'One captured private-desktop normal-client battle frame; visual inspection only.',
    'full_framebuffer_oracle_verified':False,'map303_specific_GUI_verified':False,'physical_input_verified':False,
    'whole_goal_complete':False}
dest=NORMAL/('GUI-visual-'+profile+'.json');assert not dest.exists()
dest.write_text(json.dumps(note,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');assert read(dest)==note
print(json.dumps({'status':note['status'],'profile':profile,'image_sha256':note['image_sha256']},ensure_ascii=False))
