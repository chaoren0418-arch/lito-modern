"""Qualify native class-0 source, scoped test images and two unique-entry mains."""
from pathlib import Path
from datetime import datetime
import hashlib,json,re,struct,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';S=R/'驗證紀錄/current/native-shared-higher-heals/observer-fix'
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
pins=read(S/'source-inputs.json');assert len(pins)==pipeline['source_files']==968 and sha(S/'source-inputs.json')==pipeline['source_manifest_sha256']
for n,h in pins.items():assert sha(R/n)==h,n
names={'native_skill_selection','native_effect_lifecycle','native_world_effects','native_skills'}
debug=batch(S/'Debug-four-tests.jsonl');release=batch(S/'Release-two-skill-targets.jsonl')
assert set(debug)==names and set(release)=={'native_skills','native_skill_selection'}
assert debug['native_skills']['passed']==release['native_skills']['passed']==10
assert debug['native_skill_selection']['passed']==release['native_skill_selection']['passed']>0
assert sum(v['ignored'] for v in debug.values())==0 and sum(v['ignored'] for v in release.values())==0
for profile in ['Debug','Release']:
 for name in ['native-shared-higher-heal-frames.json','native-legacy-higher-heal-frames.json']:
  groups=read(S/(profile+'-generated')/name);assert len(groups)==3
  for group in groups:
   rows=group['observations'];assert len(rows)==22
   for i,row in enumerate(rows):
    assert row['started']==1 and row['omitted']==0 and row['load_error'] is None
    f=row['effects'][0];assert f['callback_count']==i+1 and f['subframe']==(20 if i>=20 else i)
    assert f['source_frame']==(i+1 if i<20 else 0) and f['finished']==(i==21)
    assert f['next_callback_ms']==((i+1)*125 if i<21 else None) and row['active']==int(i<21)
   assert sum(len(row['entered_events']) for row in rows)==1
normal=S/'normal';normal.mkdir();(normal/'source-inputs.json').write_bytes((S/'source-inputs.json').read_bytes());mains={};files={}
for profile,item in pipeline['normal_mains'].items():
 p=Path(item['path']);assert sha(p)==item['sha256'];raw=p.read_bytes();at=struct.unpack_from('<I',raw,60)[0];assert raw[at:at+4]==bytes([80,69,0,0]) and struct.unpack_from('<H',raw,at+4)[0]==0x8664 and struct.unpack_from('<H',raw,at+24)[0]==0x20b
 entry=S/(profile+'-entry-build.log');assert sha(entry)==item['entry_build_log_sha256'] and 'Finished `' in entry.read_text(encoding='utf-8-sig')
 mains[profile]={'path':str(p),'sha256':item['sha256'],'features':[],'unique_entry':True,'machine':'AMD64','optional_header':'PE32+','entry_build_log_sha256':sha(entry),'GUI_revalidated':False};files[profile+'/LiToModern.exe']=p
assert set(mains)=={'Debug','Release'}
retained=archive(normal/'normal-mains.zip',files)
n={'status':'qualified_unique_entry_normal_mains_after_higher_heal_scoped_checks','source_files':968,'source_manifest_sha256':sha(normal/'source-inputs.json'),'features':[],'profiles':mains,'archive':retained,'archive_readback_verified':True,'Debug_scoped_targets':len(debug),'Debug_scoped_passed':sum(v['passed'] for v in debug.values()),'Debug_scoped_ignored':0,'Release_scoped_targets':len(release),'Release_scoped_passed':sum(v['passed'] for v in release.values()),'Release_scoped_ignored':0,'full_current_default90_verified':False,'full_library_regression_verified':False,'actual_higher_heal_TCP_verified':False,'GUI_verified':False,'whole_goal_complete':False}
save(normal/'qualified.json',n)
products={'Debug/'+name+'.exe':Path(x['product']) for name,x in debug.items()};products.update({'Release/'+name+'.exe':Path(x['product']) for name,x in release.items()});ptest=archive(S/'scoped-test-products.zip',products)
rawfiles={}
for p in S.iterdir():
 if p.is_file() and p.suffix in ['.json','.jsonl','.log','.txt']:rawfiles['phase/'+p.name]=p
for profile in ['Debug','Release']:
 for p in (S/(profile+'-generated')).iterdir():
  if p.is_file():rawfiles['generated/'+profile+'/'+p.name]=p
before=read(S/'source-inputs-before.json')
assert {name for name in pins if pins[name]!=before[name]}=={'LiTo381-Modern/src/modern_lito/native_peer_actions.rs'}
for name in pins:
 if pins[name]!=before[name]:rawfiles['current-source/'+name]=R/name
for name in ['qualify-native-higher-heal-observer-fix.py','run-native-higher-heal-observer-fix.ps1','verify-real-peer-higher-heal.py','verify-native-real-peers.ps1']:rawfiles['tools/'+name]=R/'維護工具'/name
rawarchive=archive(S/'scoped-raw-evidence.zip',rawfiles)
for name,h in pins.items():assert sha(R/name)==h,name
q={'status':'qualified_higher_heal_scoped_tests_and_normal_mains_only','source_files':968,'source_manifest_sha256':sha(S/'source-inputs.json'),'normal_qualification_sha256':sha(normal/'qualified.json'),'normal_mains':{k:v['sha256'] for k,v in mains.items()},'profiles':{'Debug':{'targets':len(debug),'passed':sum(v['passed'] for v in debug.values()),'failed':0,'ignored':0,'results':debug},'Release':{'targets':len(release),'passed':sum(v['passed'] for v in release.values()),'failed':0,'ignored':0,'results':release}},'test_products_archive':ptest,'raw_archive':rawarchive,'all_archives_readback_verified':True,'full_default90_verified':False,'full_library_verified':False,'actual_higher_heal_TCP_sessions':0,'GUI_verified':False,'whole_goal_complete':False}
q['previous_qualified_higher_heal']=str(R/'驗證紀錄/current/native-shared-higher-heals/qualified.json');q['previous_qualification_sha256']=sha(q['previous_qualified_higher_heal']);q['compilation_input_changes_since_previous_qualification']=['LiTo381-Modern/src/modern_lito/native_peer_actions.rs'];q['effect_loader_and_request_implementation_unchanged']=True
save(S/'qualified.json',q);print(json.dumps({'status':q['status'],'normal_mains':q['normal_mains'],'Debug_passed':q['profiles']['Debug']['passed'],'Release_passed':q['profiles']['Release']['passed']},ensure_ascii=False))
