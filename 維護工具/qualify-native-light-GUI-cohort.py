"""Seal the five defined current light GUI cases, with explicit remaining scopes."""
from pathlib import Path
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser();parser.add_argument('cohort');args=parser.parse_args()
s=Path(args.cohort).resolve(strict=True)
assert s.is_relative_to(ROOT/'驗證紀錄/current/native-light-spell/live')
assert not (s/'qualified.json').exists()
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
pipeline=read(s/'pipeline.json')
assert pipeline['status']=='five_defined_GUI_cases_qualified_pending_aggregate_archive'
assert pipeline['active_case'] is None and len(pipeline['cases'])==5
normal_path=Path(pipeline['normal_qualification']);normal=read(normal_path)
assert sha(normal_path.parent/'source-inputs.json')==normal['source_manifest_sha256']==pipeline['source_manifest_sha256']
pins=read(normal_path.parent/'source-inputs.json');assert len(pins)==970
for n,h in pins.items():assert sha(ROOT/n)==h,n
for v in normal['profiles'].values():assert sha(Path(v['path']))==v['sha256']
assert sha(Path(normal['archive']['path']))==normal['archive']['sha256'] and normal['archive']['readback_verified']
for n,h in pipeline['tool_pins'].items():assert sha(ROOT/'維護工具'/n)==h,n
expected={('Release','success','tw13081901-v1'),('Release','recast','tw13081901-v1'),('Release','refused','tw13081901-v1'),('Release','success','lito-legacy'),('Debug','success','tw13081901-v1')}
observed=set();counts={'C14':0,'S37':0,'S1B':0,'S4A':0};sessions=world_pngs=0
files={'pipeline.json':s/'pipeline.json','qualifier.py':Path(__file__),'normal-qualification.json':normal_path,'source-inputs.json':normal_path.parent/'source-inputs.json'}
for case in pipeline['cases']:
    p=Path(case['path']);assert p.resolve().is_relative_to(ROOT/'驗證紀錄/current/native-light-gui')
    assert sha(p/'qualified.json')==case['qualification_sha256']
    q=read(p/'qualified.json');key=(q['profile'],q['scenario'],q['resource_profile']);assert key not in observed;observed.add(key)
    assert q['status']=='qualified_actual_native_light_GUI_defined_scenario'
    assert q['source_manifest_sha256']==normal['source_manifest_sha256'] and q['main_sha256']==normal['profiles'][q['profile']]['sha256']
    assert q['actual_sessions']==4 and q['LOGIN_sessions']==q['GAME_sessions']==2
    assert q['raw_archive_readback_verified'] and sha(p/'raw-evidence.zip')==q['raw_archive_sha256']
    prefix='/'.join(key)
    with zipfile.ZipFile(p/'raw-evidence.zip') as z:
        assert set(z.namelist())==set(q['raw_archive_files'])
        for n,h in q['raw_archive_files'].items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
    for n,h in q['raw_archive_files'].items():
        f=ROOT/'維護工具/qualify-native-light-GUI.py' if n=='qualifier.py' else p/n
        assert sha(f)==h,n
        files[prefix+'/'+n]=f
    files[prefix+'/qualified.json']=p/'qualified.json'
    for k in counts:counts[k]+=q['wire_counts'][k]
    sessions+=q['actual_sessions'];world_pngs+=q['world_PNGs']
assert observed==expected and sessions==20 and world_pngs==22
assert counts=={'C14':6,'S37':10,'S1B':10,'S4A':5}
for f in s.glob('*.log'):files['cohort-logs/'+f.name]=f
for f in s.glob('*-qualification.txt'):files['cohort-logs/'+f.name]=f
stage=ROOT/'驗證紀錄/current/native-light-spell'
for name in ['GUI-first-visual-review.json','passive-harness-derivation.json','qualified.json','completion-observation.json']:
    files['source-and-build/'+name]=stage/name
file_pins={n:sha(p) for n,p in files.items()};archive=s/'raw-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(file_pins)
    for n,h in file_pins.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
q={'status':'qualified_five_current_native_light_GUI_defined_cases','source_files':970,'source_manifest_sha256':normal['source_manifest_sha256'],
   'normal_mains':{k:v['sha256'] for k,v in normal['profiles'].items()},'cases':pipeline['cases'],
   'actual_sessions':20,'LOGIN_sessions':10,'GAME_sessions':10,'wire_counts':counts,'world_PNGs':22,
   'original_MP4_success_casts':5,'original_MP2_refused_casts':1,'GUI_retry_and_passive_light_refresh_verified':True,
   'raw_archive_sha256':sha(archive),'raw_archive_file_pins':file_pins,'raw_archive_readback_verified':True,
   'physical_input_verified':False,'multi_DPI_verified':False,'night_illumination_verified':False,'natural3600s_expiry_verified':False,
   'GUI_device_playback_event_count_verified':False,'physical_audio_verified':False,'full_world_pixel_oracle':False,
   'full_current_TCP_verified':False,'full_default90_verified':False,'full_library_verified':False,'current_TW14_verified':False,'whole_goal_complete':False}
(s/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:q[k] for k in ['status','actual_sessions','wire_counts','world_PNGs','raw_archive_readback_verified']}))
