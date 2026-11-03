"""Prepare exact current default93 and TW14 drivers without modifying old tools."""
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT=Path(__file__).resolve().parents[1]
RUN=ROOT/'驗證紀錄/current/native-game-timers/20261016-004010-201637'
STAGE=RUN/'current-regression';STAGE.mkdir(exist_ok=False)
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
normal=read(RUN/'normal/qualified.json');pins=read(RUN/'normal/source-inputs.json')
assert len(pins)==normal['source_files']==976
for n,h in pins.items():assert sha(ROOT/n)==h,n
for v in normal['profiles'].values():assert sha(Path(v['path']))==v['sha256']
tools=Path(r'C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin')
env=os.environ.copy();env.update(PYTHONDONTWRITEBYTECODE='1',RUSTC=str(tools/'rustc.exe'),RUSTDOC=str(tools/'rustdoc.exe'),TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'));env['PATH']=str(tools)+os.pathsep+env['PATH']
p=subprocess.run([str(tools/'cargo.exe'),'metadata','--no-deps','--offline','--locked','--format-version','1'],cwd=ROOT/'LiTo381-Modern',env=env,capture_output=True,check=True)
(STAGE/'metadata.json').write_bytes(p.stdout)
pkg=next(p for p in json.loads(p.stdout)['packages'] if p['name']=='lito-modern');targets=[t for t in pkg['targets'] if 'test' in t['kind']]
defaults=sorted(t['name'] for t in targets if not t.get('required-features'));excluded={t['name']:t['required-features'] for t in targets if t.get('required-features')}
assert len(defaults)==93 and {'native_light_spell','native_light_mask','native_game_timers'}<=set(defaults)
assert excluded=={'native_teleport_controller':['teleport-wire-test']}
old_names=['run-native-light-default-regression.py','qualify-native-light-default-regression.py','run-native-light-current-teleport-GUI.ps1',
    'qualify-native-light-teleport-current-case.py','qualify-native-light-teleport-current-cohort.py','run-native-light-default91-followup.ps1',
    'run-native-light-TW14-followup.ps1','seal-native-light-default-run.py']
records={}
for name in old_names:
    old=ROOT/'維護工具'/name;new_name=name.replace('native-light-','native-game-timer-').replace('default91','default93');new=old.with_name(new_name)
    assert not new.exists(),new_name
    s=old.read_text(encoding='utf-8-sig')
    s=s.replace('native-light-spell/current-regression','native-game-timers/20261016-004010-201637/current-regression')
    s=s.replace('native-light-spell/normal','native-game-timers/20261016-004010-201637/normal')
    s=s.replace('native-light-','native-game-timer-').replace('default91','default93')
    for a,b in [('970','976'),('92','94'),('91','93')]:
        s=re.sub(r'(?<![A-Za-z0-9_])'+a+r'(?![A-Za-z0-9_])',b,s)
    if new.suffix=='.py':ast.parse(s)
    new.write_text(s,encoding='utf-8');records[new_name]={'sha256':sha(new),'original':name,'original_sha256':sha(old)}
result={'status':'prepared_current976_default93_plus_main94_and_TW14_drivers','source_manifest_sha256':normal['source_manifest_sha256'],
    'normal_qualification':str(RUN/'normal/qualified.json'),'default_integration_targets':defaults,'integration_count':93,'expected_test_products_per_profile':94,
    'excluded_required_feature_targets':excluded,'prepared_tools':records,'validation_executed_by_preparation':False,'whole_goal_complete':False}
(STAGE/'preparation.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':result['status'],'integration_targets':93,'products_per_profile':94,'tools':len(records)},ensure_ascii=False))
