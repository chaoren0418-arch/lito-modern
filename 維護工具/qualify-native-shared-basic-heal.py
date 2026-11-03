"""Qualify native class-0 source, scoped test images and two unique-entry mains."""
from pathlib import Path
from datetime import datetime
import hashlib,json,re,struct,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';S=R/'驗證紀錄/current/native-shared-basic-heal'
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
debug=batch(S/'Debug-thirteen-tests.jsonl');release=batch(S/'Release-six-skill-tests.jsonl')
assert set(debug)==names and set(release)=={'native_skills'}
assert sum(v['passed'] for v in debug.values())==60 and sum(v['ignored'] for v in debug.values())==1
assert debug['native_skills']['passed']==release['native_skills']['passed']==6 and release['native_skills']['ignored']==0
first=read(S/'first-green-unit-qualified.json');assert first['passed']==1 and first['failed']==0 and first['filtered']==5 and sha(S/'first-green-unit-retained.zip')==first['archive_sha256']
for profile in ['Debug','Release']:
 rows=read(S/(profile+'-generated/native-shared-basic-heal-frames.json'));assert len(rows)==22
 for i,row in enumerate(rows):
  assert row['started']==1 and row['omitted']==0 and row['load_error'] is None
  f=row['effects'][0];assert (f['graphics'],f['sprite'],f['subframe'],f['callback_count'],f['native_clock_elapsed_ms'])==(11722,'11722-0.spr',20 if i>=20 else i,i+1,i*125)
  assert f['source_frame']==(i+1 if i<20 else 0) and f['drawn']==(1<=i<=12) and f['finished']==(i==21)
  assert f['next_callback_ms']==((i+1)*125 if i<21 else None) and row['active']==int(i<21)
  assert len(row['entered_events'])==int(i==0)
  if i==0:assert row['entered_events'][0]['sound']==109
  assert row['tw_catalog']['decoded_sha256']=='7ebe2b464c8b1ea9c58ac985dd8ff4c823784d8231894262eddcd7691ca94008'
  src=next(x for x in row['sources'] if x['name']=='11722-0.spr');assert src['bytes']==30637 and src['sha256']=='85f6872052fa335cab1ee565c3e184e2d4e718a6137c87d28850c88ecca6f8e5'
normal=S/'normal';normal.mkdir();(normal/'source-inputs.json').write_bytes((S/'final-source-inputs.json').read_bytes());mains={};files={}
for profile,item in pipeline['normal_mains'].items():
 p=Path(item['path']);assert sha(p)==item['sha256'];raw=p.read_bytes();at=struct.unpack_from('<I',raw,60)[0];assert raw[at:at+4]==bytes([80,69,0,0]) and struct.unpack_from('<H',raw,at+4)[0]==0x8664 and struct.unpack_from('<H',raw,at+24)[0]==0x20b
 entry=S/(profile+'-entry-build.log');assert sha(entry)==item['entry_build_log_sha256'] and 'Finished `' in entry.read_text(encoding='utf-8-sig')
 mains[profile]={'path':str(p),'sha256':item['sha256'],'features':[],'unique_entry':True,'machine':'AMD64','optional_header':'PE32+','entry_build_log_sha256':sha(entry),'GUI_revalidated':False};files[profile+'/LiToModern.exe']=p
assert set(mains)=={'Debug','Release'}
retained=archive(normal/'normal-mains.zip',files)
n={'status':'qualified_unique_entry_normal_mains_after_native_TW11722_scoped_checks','source_files':968,'source_manifest_sha256':sha(normal/'source-inputs.json'),'features':[],'profiles':mains,'archive':retained,'archive_readback_verified':True,'Debug_scoped_targets':13,'Debug_scoped_passed':60,'Debug_scoped_ignored':1,'Release_scoped_targets':1,'Release_scoped_passed':6,'Release_scoped_ignored':0,'full_current_default90_verified':False,'full_library_regression_verified':False,'actual_shared_profile_retest_verified':False,'GUI_verified':False,'whole_goal_complete':False}
save(normal/'qualified.json',n)
products={'Debug/'+name+'.exe':Path(x['product']) for name,x in debug.items()};products.update({'Release/'+name+'.exe':Path(x['product']) for name,x in release.items()});ptest=archive(S/'scoped-test-products.zip',products)
rawfiles={}
for p in S.rglob('*'):
 if p.is_file() and p.suffix in ['.json','.jsonl','.log','.txt','.rs','.py','.ps1','.png'] and p.name!='qualified.json':
  assert not p.is_symlink();rawfiles['phase/'+p.relative_to(S).as_posix()]=p
before=read(S/'source-inputs-before.json')
for name in pins:
 if pins[name]!=before[name]:rawfiles['current-source/'+name]=R/name
for name in ['qualify-native-shared-basic-heal.py','run-native-shared-basic-heal-validation.ps1','run-native-shared-profile-baseline.ps1','verify-native-real-peers.ps1']:rawfiles['tools/'+name]=R/'維護工具'/name
rawarchive=archive(S/'raw-evidence.zip',rawfiles)
for name,h in pins.items():assert sha(R/name)==h,name
proof={'status':'qualified_original_TW11722_basic_heal_scoped_assets_and_normal_builds','source_files':968,'source_manifest_sha256':sha(S/'final-source-inputs.json'),'normal_qualification_sha256':sha(normal/'qualified.json'),'normal_mains':{p:x['sha256'] for p,x in mains.items()},'profiles':{'Debug':{'targets':13,'passed':60,'failed':0,'ignored':1,'results':debug},'Release':{'targets':1,'passed':6,'failed':0,'ignored':0,'results':release}},'actual_original_slots':21,'visible_original_slots':12,'blank_original_slots':[0,*range(13,21)],'blank_marker':[192,120,0,0,65535,0],'sound109_events_once':True,'terminal_blank_dwell_retained':True,'world_authority_unchanged':True,'legacy_generation_guard_retained':True,'separate_explicit_TW_source_and_wire11722_SPR_used':True,'test_products':ptest,'raw_archive':rawarchive,'full_current_default90_verified':False,'full_library_regression_verified':False,'actual_shared_profile_retest_verified':False,'all_shared_resource_profile_support_verified':False,'GUI_verified':False,'physical_audio_verified':False,'whole_goal_complete':False,'qualifier_sha256':sha(__file__)}
save(S/'qualified.json',proof);print(json.dumps({'status':proof['status'],'normal_mains':proof['normal_mains'],'Debug_passed':60,'Debug_ignored':1,'Release_passed':6,'Release_ignored':0}))
