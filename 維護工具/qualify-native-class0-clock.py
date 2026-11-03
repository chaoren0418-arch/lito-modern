"""Qualify native class-0 source, scoped test images and two unique-entry mains."""
from pathlib import Path
from datetime import datetime
import hashlib,json,re,struct,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';S=R/'驗證紀錄/current/native-effect-native-clock'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,d):p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def archive(path,files):
 assert not path.exists();pins={n:sha(p) for n,p in files.items()}
 with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
  for n,p in files.items():z.write(p,n)
 with zipfile.ZipFile(path) as z:
  assert set(z.namelist())==set(pins)
  for n,h in pins.items():
   with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
 return {'path':str(path),'sha256':sha(path),'file_pins':pins,'readback_verified':True}
def batch(path):
 text=path.read_text(encoding='utf-8-sig');messages=[json.loads(l) for l in text.splitlines() if l.startswith('{')]
 assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
 products={m['target']['name']:m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['target']['kind']==['test']}
 runs=list(re.finditer(r'Running tests[\\/]([A-Za-z0-9_]+)\.rs\s+\(',text));result={}
 for i,m in enumerate(runs):
  name=m[1];body=text[m.end():runs[i+1].start() if i+1<len(runs) else len(text)]
  terminal=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',body);assert len(terminal)==1,(name,terminal)
  status,passed,failed,ignored,measured,filtered=terminal[0];assert status=='ok' and failed==measured==filtered=='0'
  entries=re.findall(r'^test (.+) \.\.\. (ok|ignored)(?:,.*)?$',body,re.M);assert len(entries)==int(passed)+int(ignored) and len({n for n,_ in entries})==len(entries)
  assert sum(s=='ok' for _,s in entries)==int(passed) and sum(s=='ignored' for _,s in entries)==int(ignored)
  product=products[name];assert product['features']==[]
  result[name]={'passed':int(passed),'failed':0,'ignored':int(ignored),'test_names':[n for n,_ in entries],'product':product['executable'],'product_sha256':sha(product['executable']),'features':[]}
 assert set(result)==set(products)
 return result
pipeline=read(S/'validation-pipeline.json');assert pipeline['status']=='completed_pending_independent_qualification'
pins=read(S/'final-source-inputs.json');assert len(pins)==pipeline['source_files']==968 and sha(S/'final-source-inputs.json')==pipeline['source_manifest_sha256']
for n,h in pins.items():assert sha(R/n)==h,n
initial=read(S/'Debug-initial-eight-qualified.json');assert initial['passed']==35 and initial['failed']==0 and initial['ignored']==1 and initial['archive_readback_verified'];assert sha(S/'Debug-initial-eight-raw.zip')==initial['raw_archive_sha256']
prior=read(S/'green-source-inputs.json');changed=[n for n in prior if prior[n]!=pins[n]];assert changed==['LiTo381-Modern/tests/native_effect_lifecycle.rs']
debug=batch(S/'Debug-final-effect-tests.jsonl');release=batch(S/'Release-final-eight-tests.jsonl');assert set(debug)=={'native_effect_lifecycle'} and debug['native_effect_lifecycle']['passed']==7
names={'native_effect_lifecycle','native_holy_weapon','native_poison_cure','native_travel','native_turn_undead','native_reagent_skills','native_item_effects','native_world_effects'};assert set(release)==set(initial['targets'])==names
# The seven retained Debug images exclude the one subsequently extended test file.
for name,p in initial['targets'].items():
 if name=='native_effect_lifecycle':continue
 assert sha(p['product'])==p['product_sha256'];dep=Path(p['product']).with_suffix('.d');assert sha(dep)==p['dependency_sha256'] and 'native_effect_lifecycle.rs' not in dep.read_text(encoding='utf-8')
debug_passed=initial['passed']-6+debug['native_effect_lifecycle']['passed'];assert debug_passed==36 and sum(x['passed'] for x in release.values())==36 and sum(x['ignored'] for x in release.values())==1
samples=read(P/'target/current/verification/native-class0-clock-samples.json');assert len(samples)==7
for row,source,now,callbacks,due in zip(samples,[1,1,2,3,4,0,0],[0,82,83,1000,1083,1166,1249],[1,1,2,3,4,5,6],[83,83,166,1083,1166,1249,None]):
 e=row['effects'][0];assert e['source_frame']==source and e['native_clock_elapsed_ms']==now and e['callback_count']==callbacks and e['next_callback_ms']==due
 assert e['drawn']==(now!=1249) and e['finished']==(now==1249) and row['omitted']==0 and row['load_error'] is None
assert [len(x['entered_events']) for x in samples]==[1,0,0,0,0,0,0]
contract=read(S/'native-delay-vectors.json');assert contract['status']=='verified_PE_parser_slice_integer_delay_and_inferred_timer_vectors' and not contract['original_native_client_executed']
normal=S/'normal';normal.mkdir();(normal/'source-inputs.json').write_bytes((S/'final-source-inputs.json').read_bytes());mainfiles={};profiles={}
for profile,m in pipeline['normal_mains'].items():
 path=Path(m['path']);assert sha(path)==m['sha256'];raw=path.read_bytes();at=struct.unpack_from('<I',raw,60)[0];assert raw[at:at+4]==b'PE\0\0' and struct.unpack_from('<H',raw,at+4)[0]==0x8664 and struct.unpack_from('<H',raw,at+24)[0]==0x20b
 entry=S/(profile+'-entry-build.log');assert sha(entry)==m['entry_build_log_sha256'] and 'Finished `' in entry.read_text(encoding='utf-8-sig')
 profiles[profile]={'path':str(path),'sha256':m['sha256'],'features':[],'unique_entry':True,'machine':'AMD64','optional_header':'PE32+','GUI_revalidated':False,'entry_build_log_sha256':sha(entry)};mainfiles[profile+'/LiToModern.exe']=path
assert set(profiles)=={'Debug','Release'}
mainarchive=archive(normal/'normal-mains.zip',mainfiles)
normalq={'status':'qualified_unique_entry_normal_mains_after_native_class0_clock_fix','source_files':968,'source_manifest_sha256':sha(normal/'source-inputs.json'),'features':[],'profiles':profiles,'archive':mainarchive,'archive_readback_verified':True,'full_library_regression_verified':False,'full_current_TCP_regression_verified':False,'whole_goal_complete':False}
save(normal/'qualified.json',normalq)
products={'Debug/native_effect_lifecycle.exe':Path(debug['native_effect_lifecycle']['product'])}
for name,x in release.items():products['Release/'+name+'.exe']=Path(x['product'])
retention=archive(S/'final-scoped-test-products.zip',products)
files={}
for f in S.rglob('*'):
 if f.is_file() and f.suffix in ['.json','.jsonl','.txt','.log','.rs','.py','.ps1'] and f.name not in ['qualified.json']:
  assert f.resolve().is_relative_to(S) and not f.is_symlink();files['scope/'+f.relative_to(S).as_posix()]=f
for n in ['src/modern_lito/native_effect_timing.rs','src/modern_lito/native_skill_effects.rs','tests/native_effect_lifecycle.rs','tests/native_holy_weapon.rs','tests/native_poison_cure.rs','tests/native_travel.rs','tests/native_turn_undead.rs']:
 files['source-current/'+n]=P/n
step=next(x for x in pipeline['steps'] if x['profile']=='Release');start=datetime.fromisoformat(step['started_utc']).timestamp();end=datetime.fromisoformat(step['finished_utc']).timestamp();pngs=0
for p in (P/'target/current/verification').iterdir():
 if p.is_file() and start<=p.stat().st_mtime<=end+1 and p.suffix in ['.json','.png']:
  assert p.stat().st_size<32*1024**2;files['Release-generated/'+p.name]=p;pngs+=p.suffix=='.png'
for n in ['verify-native-class0-clock-source.py','run-native-class0-clock-validation.ps1']:
 files['validation-sources/'+n]=R/'維護工具'/n
files['qualifier.py']=Path(__file__);files['unique-entry.ps1']=R/'啟動現代客戶端.ps1'
rawarchive=archive(S/'native-clock-raw-evidence.zip',files)
for n,h in pins.items():assert sha(R/n)==h
proof={'status':'qualified_native_class0_integer_frame_delays_and_callback_relative_dwell','source_files':968,'source_manifest_sha256':sha(S/'final-source-inputs.json'),'profiles':{'Debug':{'passed':36,'failed':0,'ignored':1,'targets':8,'initial_eight_receipt_sha256':sha(S/'Debug-initial-eight-qualified.json'),'final_effect_target':debug},'Release':{'passed':36,'failed':0,'ignored':1,'targets':8,'results':release}},'normal_qualification':str(normal/'qualified.json'),'normal_qualification_sha256':sha(normal/'qualified.json'),'normal_mains':{p:x['sha256'] for p,x in profiles.items()},'new_tests_in_default_integration_inventory':2,'original_opcode_delay_vectors_verified':True,'native_common_clock_submillisecond_birth_verified':True,'native_late_callback_positive_dwell_no_catchup_verified':True,'terminal_and_zero_delay_cue_order_verified':True,'explicit_Life_Stream_action3_timeline_retained':True,'world_authority_unchanged_by_presentation_tests':True,'Release_generated_PNGs_retained':pngs,'test_product_retention':retention,'raw_evidence_retention':rawarchive,'Debug_initial_raw_archive_sha256':initial['raw_archive_sha256'],'native_global_cross_object_timer_order_verified':False,'native_timer_wrap_verified':False,'native_client_executed':False,'new_GUI_verified':False,'full_default90_regression_verified':False,'full_current_library_default_selection_verified':False,'full_current_TCP_verified':False,'whole_goal_complete':False,'qualifier_sha256':sha(__file__)}
save(S/'qualified.json',proof);print(json.dumps({'status':proof['status'],'Debug_passed':36,'Release_passed':36,'normal_mains':proof['normal_mains'],'Release_generated_PNGs_retained':pngs},ensure_ascii=False))
