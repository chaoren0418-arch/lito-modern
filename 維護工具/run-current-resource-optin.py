"""Opt in only five audited original-resource tests from the pinned current lib image."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,os,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def write(p,d):p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('build_run',type=Path);a=ap.parse_args();run=a.build_run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current')
q=read(run/'default-library-qualified.json');product=read(run/'product-retention.json');pins=read(run/'source-inputs.json');inventory=read(run/'inventory-audit/inventory.json')
assert q['source_files']==len(pins)==968 and q['default_full_library_selection_verified'] and q['failed']==0
assert sha(product['product'])==q['product_sha256'] and sha(product['archive'])==q['product_archive_sha256']
for name,h in pins.items():assert sha(ROOT/name)==h,name
names=['modern_lito::actor_assets::tests::real_explicit_sprite_bank_binds_eight_headings_and_unique_scope_rejects_collision','modern_lito::animation::tests::real_shared_attack_uses_source_fps_ticks_and_once_only_presentation_events','modern_lito::ammo_resource_reference_tests::actual_shared_bow_bodies_match_independent_pixels_and_launch_cues','modern_lito::ammo_resource_reference_tests::actual_shared_arrow_eight_banks_match_independent_reference','modern_lito::audio::tests::real_shared_sound_1539_matches_pcm_reference']
assert len(set(names))==5 and all(n in inventory['ignored_test_names'] for n in names)
original=Path('D:/lineage開發/0-客戶端').resolve(strict=True)
assert original!=ROOT and not original.is_relative_to(ROOT)
inputs=[original/'Sound/1539.wav']+sorted(original.glob('Sprite*.idx'))+[original/'Text.idx',original/'Text.pak']
input_pins={str(p):sha(p) for p in inputs}
# Whole multi-GiB sprite PAKs remain in their original location. Test bodies validate selected entry bytes.
pak_metadata={str(p):{'bytes':p.stat().st_size,'mtime_ns':p.stat().st_mtime_ns} for p in sorted(original.glob('Sprite*.pak'))}
legacy=PROJECT/'logs/continuation-20261003';historical={str(p.relative_to(legacy)):sha(p) for p in legacy.glob('*.json')}
out=ROOT/'驗證紀錄/current/native-resource-optin'/datetime.now().strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
write(out/'preflight.json',{'tests':names,'original_root':str(original),'read_only_resource_access':True,'input_file_pins':input_pins,'sprite_PAK_metadata':pak_metadata,'product_sha256':q['product_sha256'],'source_manifest_sha256':q['source_manifest_sha256'],'whole_goal_complete':False})
results=[];env=os.environ.copy();env['LITO_SHARED_RESOURCE_ROOT']=str(original);env['TEMP']=env['TMP']=str(PROJECT/'target/current/scratch')
for k in list(env):
 if k.startswith(('LITO_R','LITO_NATIVE_')):env.pop(k)
for i,name in enumerate(names):
 args=[product['product'],name,'--exact','--ignored','--nocapture','--test-threads=1'];started=datetime.now(timezone.utc).isoformat()
 p=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 try:stdout,stderr=p.communicate(timeout=180)
 except subprocess.TimeoutExpired:
  p.kill();stdout,stderr=p.communicate();(out/f'{i}-stdout.txt').write_bytes(stdout);(out/f'{i}-stderr.txt').write_bytes(stderr);write(out/'failure.json',{'status':'owned_direct_child_timeout','pid':p.pid,'test':name,'started_utc':started});raise
 (out/f'{i}-stdout.txt').write_bytes(stdout);(out/f'{i}-stderr.txt').write_bytes(stderr)
 text=stdout.decode('utf-8-sig');assert p.returncode==0 and f'test {name} ... ok' in text and '1 passed; 0 failed; 0 ignored; 0 measured; 2401 filtered out' in text,(name,p.returncode,text)
 results.append({'name':name,'pid':p.pid,'command':args,'started_utc':started,'finished_utc':datetime.now(timezone.utc).isoformat(),'exit_code':p.returncode,'passed':1,'stdout_sha256':sha(out/f'{i}-stdout.txt'),'stderr_sha256':sha(out/f'{i}-stderr.txt')})
 write(out/'executed.json',results)
for name,h in pins.items():assert sha(ROOT/name)==h,name
for name,h in input_pins.items():assert sha(name)==h,name
for name,s in pak_metadata.items():assert Path(name).stat().st_size==s['bytes'] and Path(name).stat().st_mtime_ns==s['mtime_ns'],name
assert historical=={str(p.relative_to(legacy)):sha(p) for p in legacy.glob('*.json')}
n=read(ROOT/'驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/qualified.json')
for x in n['profiles'].values():assert sha(x['path'])==x['sha256']
files={f.name:f for f in out.iterdir() if f.is_file()};files['runner.py']=Path(__file__);files['full-library-qualification.json']=run/'default-library-qualified.json';files['source-inputs.json']=run/'source-inputs.json'
for name in ['actor_assets.rs','animation.rs','ammo_resource_reference_tests.rs','audio.rs','resources.rs']:files['source/'+name]=PROJECT/'src/modern_lito'/name
file_pins={name:sha(p) for name,p in files.items()};archive=out/'raw-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
 for name,p in files.items():z.write(p,name)
with zipfile.ZipFile(archive) as z:
 assert set(z.namelist())==set(file_pins)
 for name,h in file_pins.items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==h,name
proof={'status':'qualified_five_explicit_current_original_resource_tests','source_files':968,'source_manifest_sha256':q['source_manifest_sha256'],'test_product_sha256':q['product_sha256'],'test_product_archive':product['archive'],'test_product_archive_sha256':q['product_archive_sha256'],'passed':5,'failed':0,'tests':results,'previously_ignored_default_tests_explicitly_executed':5,'remaining_original_ignored_tests_unexecuted':468,'default_library_run_ignored_count_unchanged':473,'historical_reports_unchanged':len(historical),'original_IDX_Text_WAV_SHA_unchanged':True,'original_sprite_PAK_metadata_unchanged':True,'source_and_normal_mains_unchanged':True,'raw_file_pins':file_pins,'raw_archive_sha256':sha(archive),'archive_readback_verified':True,'formal_protocol_sessions':0,'GUI_verified':False,'audio_device_verified':False,'all_resources_or_all_ignored_verified':False,'whole_goal_complete':False}
write(out/'qualified.json',proof);print(json.dumps({'output':str(out),'status':proof['status'],'passed':5,'failed':0},ensure_ascii=False))
