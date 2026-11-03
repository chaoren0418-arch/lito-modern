"""Qualify the shared finite callback consumers, scoped images and unique-entry mains."""
from pathlib import Path
from datetime import datetime
import hashlib,json,re,struct,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';S=R/'驗證紀錄/current/native-effect-native-clock/common-finite-followup'
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
names={'native_effect_lifecycle','native_holy_weapon','native_poison_cure','native_travel','native_turn_undead','native_reagent_skills','native_item_effects','native_world_effects','native_chill_touch','native_vampiric_touch','native_skills','native_lightning','native_stalac_layers'}
profiles={};products={}
for profile in ['Debug','Release']:
 result=batch(S/(profile+'-thirteen-tests.jsonl'));assert set(result)==names
 assert result['native_chill_touch']['passed']==4 and result['native_lightning']['passed']==5 and result['native_vampiric_touch']['passed']==4
 profiles[profile]={'passed':sum(x['passed'] for x in result.values()),'failed':0,'ignored':sum(x['ignored'] for x in result.values()),'targets':13,'results':result}
 assert profiles[profile]['ignored']==1 and profiles[profile]['passed']==59
 for n,x in result.items():products[profile+'/'+n+'.exe']=Path(x['product'])
 for trace,expected_sources,clocks,callbacks,deadlines in [
  ('native-common-finite-class1-clock.json',[1,2,3,4,5,0,0],[0,41,400,525,691,732,857],[1,2,3,4,5,6,7],[41,124,525,691,732,857,None]),
  ('native-common-finite-many-clock.json',[1,2,3,4,5,6,7,8,9,0,0],[0,41,1000,1083,1166,1249,1332,1415,1498,1581,1664],list(range(1,12)),[41,124,1083,1166,1249,1332,1415,1498,1581,1664,None])]:
  rows=read(S/(profile+'-generated')/trace);assert len(rows)==len(clocks)
  for row,source,clock,count,due in zip(rows,expected_sources,clocks,callbacks,deadlines):
   frame=row['effects'][0];assert (frame['source_frame'],frame['native_clock_elapsed_ms'],frame['callback_count'],frame['next_callback_ms'])==(source,clock,count,due)
   assert row['load_error'] is None and row['omitted']==0
  assert rows[-1]['active']==0 and rows[-1]['hits']==1
  assert sum(len(row['entered_events']) for row in rows)==1
# Re-read the original PE and every saved disassembly byte; no original execution.
import pefile
pepath=R.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
contract=read(S.parent/'common-finite-constructor-proof.json');assert sha(pepath)==contract['original_pe_sha256'] and contract['static_only']
pe=pefile.PE(str(pepath),fast_load=True)
allrows={row['va']:row for region in contract['ranges'].values() for row in region['rows']}
for row in allrows.values():
 raw=bytes.fromhex(row['bytes']);assert pe.get_data(int(row['va'],16)-pe.OPTIONAL_HEADER.ImageBase,len(raw))==raw
for va,raw in [('0x44ee12','c706c0844f00'),('0x44ef17','c706c0844f00'),('0x450023','e858efffff')]:assert allrows[va]['bytes']==raw
failure=read(S/'failed-scope.json');assert (failure['passed'],failure['failed'],failure['ignored'],failure['targets_executed'])==(372,5,19,91)
assert sha(S/'failed-default90-raw.zip')==failure['raw_archive_sha256']
with zipfile.ZipFile(S/'failed-default90-raw.zip') as z:
 for name,h in failure['raw_file_pins'].items():
  with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,name
red=read(S/'red-common-clock-retained.json');assert red['failed']==1 and red['passed']==0 and red['archive_readback']
assert sha(S/'red-common-clock-products.zip')==red['archive_sha256']
normal=S/'normal';normal.mkdir();(normal/'source-inputs.json').write_bytes((S/'final-source-inputs.json').read_bytes());mainfiles={};mains={}
for profile,m in pipeline['normal_mains'].items():
 path=Path(m['path']);assert sha(path)==m['sha256'];raw=path.read_bytes();at=struct.unpack_from('<I',raw,60)[0];assert raw[at:at+4]==bytes([80,69,0,0])
 assert struct.unpack_from('<H',raw,at+4)[0]==0x8664 and struct.unpack_from('<H',raw,at+24)[0]==0x20b
 entry=S/(profile+'-entry-build.log');assert sha(entry)==m['entry_build_log_sha256'] and 'Finished `' in entry.read_text(encoding='utf-8-sig')
 mains[profile]={'path':str(path),'sha256':m['sha256'],'features':[],'unique_entry':True,'machine':'AMD64','optional_header':'PE32+','GUI_revalidated':False,'entry_build_log_sha256':sha(entry)};mainfiles[profile+'/LiToModern.exe']=path
assert set(mains)=={'Debug','Release'}
mainarchive=archive(normal/'normal-mains.zip',mainfiles)
normalq={'status':'qualified_unique_entry_normal_mains_after_common_finite_clock_fix','source_files':968,'source_manifest_sha256':sha(normal/'source-inputs.json'),'features':[],'profiles':mains,'archive':mainarchive,'archive_readback_verified':True,'full_library_regression_verified':False,'full_current_TCP_regression_verified':False,'whole_goal_complete':False}
save(normal/'qualified.json',normalq)
retention=archive(S/'scoped-test-products.zip',products)
files={}
for p in S.rglob('*'):
 if p.is_file() and p.suffix in ['.json','.jsonl','.txt','.log','.rs','.py','.ps1','.png','.diff'] and p.name!='qualified.json':
  assert not p.is_symlink();files['scope/'+p.relative_to(S).as_posix()]=p
beforepins=read(S/'source-inputs-before.json')
for n in pins:
 if pins[n]!=beforepins[n]:files['current-source/'+n]=R/n
files['source-contract.json']=S.parent/'common-finite-constructor-proof.json'
for n in ['qualify-native-common-finite-clock.py','run-native-common-finite-clock-validation.ps1','run-native-common-clock-default90-followup.ps1','run-native-common-clock-current14-followup.ps1','qualify-native-teleport-current-cohort.py']:files['tools/'+n]=R/'維護工具'/n
rawarchive=archive(S/'raw-evidence.zip',files)
for n,h in pins.items():assert sha(R/n)==h,n
q={'status':'qualified_common_finite_class1_S39_birth_phase_and_callback_tests','source_files':968,'source_manifest_sha256':sha(S/'final-source-inputs.json'),'profiles':profiles,'normal_qualification':str(normal/'qualified.json'),'normal_mains':{p:x['sha256'] for p,x in mains.items()},'normal_qualification_sha256':sha(normal/'qualified.json'),'original_common_finite_constructor_and_updater_bytes_verified':True,'native_submillisecond_birth_and_late_callback_class1_S39_verified':True,'world_authority_unchanged_by_tests':True,'prior_default90_Debug_five_failures_retained':True,'red_test_retained':read(S/'red-common-clock-retained.json'),'test_products':retention,'raw_archive':rawarchive,'native_client_executed':False,'global_cross_object_timer_verified':False,'native_timer_wrap_verified':False,'full_default90_regression_verified':False,'full_library_regression_verified':False,'new_GUI_verified':False,'full_TCP_verified':False,'whole_goal_complete':False,'qualifier_sha256':sha(__file__)}
save(S/'qualified.json',q);print(json.dumps({'status':q['status'],'counts':{p:{k:v[k] for k in ['passed','failed','ignored','targets']} for p,v in profiles.items()},'mains':q['normal_mains']}))
