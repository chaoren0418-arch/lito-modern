"""Qualify native class-0 source, scoped test images and two unique-entry mains."""
from pathlib import Path
from datetime import datetime
import hashlib,json,re,struct,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';S=R/'驗證紀錄/current/native-shared-higher-heals/peer-close-fix'
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



pipeline=read(S/'validation-pipeline.json');assert pipeline['status']=='completed_pending_independent_qualification' and pipeline['steps']==[]
pins=read(S/'source-inputs.json');before=read(S/'source-inputs-before.json');assert len(pins)==len(before)==968 and sha(S/'source-inputs.json')==pipeline['source_manifest_sha256']
assert {n for n in pins if pins[n]!=before[n]}=={'LiTo381-Modern/src/modern_lito/native_peer_actions.rs'}
for n,h in pins.items():assert sha(R/n)==h,n
normal=S/'normal';normal.mkdir();(normal/'source-inputs.json').write_bytes((S/'source-inputs.json').read_bytes());mains={};files={}
for profile,item in pipeline['normal_mains'].items():
 p=Path(item['path']);assert sha(p)==item['sha256'];raw=p.read_bytes();at=struct.unpack_from('<I',raw,60)[0];assert raw[at:at+4]==bytes([80,69,0,0]) and struct.unpack_from('<H',raw,at+4)[0]==0x8664 and struct.unpack_from('<H',raw,at+24)[0]==0x20b
 entry=S/(profile+'-entry-build.log');assert sha(entry)==item['entry_build_log_sha256'] and 'Finished `' in entry.read_text(encoding='utf-8-sig')
 mains[profile]={'path':str(p),'sha256':item['sha256'],'features':[],'unique_entry':True,'machine':'AMD64','optional_header':'PE32+','entry_build_log_sha256':sha(entry),'GUI_revalidated':False};files[profile+'/LiToModern.exe']=p
assert set(mains)=={'Debug','Release'}
retained=archive(normal/'normal-mains.zip',files)
prior=R/'驗證紀錄/current/native-shared-higher-heals/observer-fix/qualified.json'
n={'status':'qualified_unique_entry_normal_mains_after_peer_close_probe_fix_build_only','source_files':968,'source_manifest_sha256':sha(normal/'source-inputs.json'),'features':[],'profiles':mains,'archive':retained,'archive_readback_verified':True,'unit_tests_executed_this_build':0,'previous_higher_heal_scoped_qualification':str(prior),'previous_qualification_sha256':sha(prior),'compilation_input_delta':['LiTo381-Modern/src/modern_lito/native_peer_actions.rs'],'request_and_effect_loader_unchanged':True,'full_current_default90_verified':False,'full_library_regression_verified':False,'actual_higher_heal_TCP_verified':False,'GUI_verified':False,'whole_goal_complete':False}
save(normal/'qualified.json',n)
rawfiles={}
for p in S.iterdir():
 if p.is_file():rawfiles['phase/'+p.name]=p
rawfiles['current-source/native_peer_actions.rs']=P/'src/modern_lito/native_peer_actions.rs'
for name in ['qualify-native-higher-heal-peer-close-build.py','run-native-higher-heal-peer-close-build.ps1','run-native-higher-heal-live.ps1','verify-native-real-peers.ps1','verify-real-peer-higher-heal.py']:rawfiles['tools/'+name]=R/'維護工具'/name
raw=archive(S/'build-raw-evidence.zip',rawfiles)
for name,h in pins.items():assert sha(R/name)==h,name
q=dict(n);q['normal_qualification']=str(normal/'qualified.json');q['normal_qualification_sha256']=sha(normal/'qualified.json');q['raw_archive']=raw;save(S/'qualified.json',q)
print(json.dumps({'status':n['status'],'normal_mains':{k:v['sha256'] for k,v in mains.items()},'unit_tests_executed_this_build':0,'archive_readback_verified':True},ensure_ascii=False))
