"""Inspect original class references and bytecode; never load server classes.

A constant-pool reference is only a candidate caller. It is not proof of a
reachable packet route, normal gameplay mutation, or a successful scenario.
"""
from pathlib import Path
from datetime import datetime,timezone
import json,struct,subprocess,zipfile
from native_E149_recovery_contract import R,P,FORMAL,SOURCE,read,sha,encoded,owned
w=read(R/'工作佇列.json')
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
jar=owned(w['current_formal_reference_identity']['cached_reference']);assert sha(jar)==FORMAL
out=R/'驗證紀錄/current/native-E149-item-gameplay-reachability'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
actor='local/lito381/server/NativeDeWorldKinds$Item'
targets={'setAttrEnchantKind','setAttrEnchantLevel','setGreater','setInlayCards','setAwakenPowerIds',
 'addAwakenPowerId','clearAwakenPowerIds','setMagicStone','clearMagicStone','setDarkItem'}
fields={'_attrEnchantKind','_attrEnchantLevel','_greater','_inlayCards','_awakenPowerIds',
 '_magicStoneItemId','_magicStoneLevel','_magicStoneTime'}

def constant_pool(blob):
 assert blob[:4]==b'\xca\xfe\xba\xbe'
 size=struct.unpack_from('>H',blob,8)[0];cp=[None]*size;at=10;i=1
 while i<size:
  tag=blob[at];at+=1
  if tag==1:
   length=struct.unpack_from('>H',blob,at)[0];at+=2
   # ASCII method/class identities are unaffected by classfile modified UTF8.
   cp[i]=(tag,blob[at:at+length].decode('utf-8','replace'));at+=length
  elif tag in [3,4]:at+=4
  elif tag in [5,6]:at+=8;i+=1
  elif tag in [7,8,16,19,20]:cp[i]=(tag,struct.unpack_from('>H',blob,at)[0]);at+=2
  elif tag in [9,10,11,12,17,18]:cp[i]=(tag,*struct.unpack_from('>HH',blob,at));at+=4
  elif tag==15:at+=3
  else:raise RuntimeError('unknown classfile CP tag '+str(tag))
  i+=1
 return cp

refs=[];classes=[];field_refs=[];reflective_names=[]
with zipfile.ZipFile(jar) as z:
 for entry in z.infolist():
  if not entry.filename.startswith('local/lito381/server/') or not entry.filename.endswith('.class'):continue
  cp=constant_pool(z.read(entry));found=[]
  for value in cp:
   if value and value[0] in [10,11]:
    owner=cp[cp[value[1]][1]][1];nt=cp[value[2]];name,descriptor=cp[nt[1]][1],cp[nt[2]][1]
    # Inherited calls may encode the subclass as owner. Match names across
    # owners and retain the descriptor/owner for subsequent bytecode review.
    if name in targets:found.append(dict(owner=owner,name=name,descriptor=descriptor))
   elif value and value[0]==9:
    owner=cp[cp[value[1]][1]][1];nt=cp[value[2]];name=cp[nt[1]][1]
    if name in fields:field_refs.append(dict(class_name=entry.filename[:-6].replace('/','.'),owner=owner,field=name))
  names=sorted({v[1] for v in cp if v and v[0]==1 and v[1] in fields})
  if names:reflective_names.append(dict(class_name=entry.filename[:-6].replace('/','.'),names=names))
  if found:refs.append(dict(class_name=entry.filename[:-6].replace('/','.'),references=found))
  classes.append(entry.filename)
selected={row['class_name'] for row in refs+field_refs+reflective_names}
selected|={'local.lito381.server.NativeDeItemStateCopy','local.lito381.server.NativeDeItemInventory',
 'local.lito381.server.NativeDeItemApplication$Owner','local.lito381.server.NativeDeWorldKinds$Item'}
assert len(selected)<=24,'inspect the new caller set before broad disassembly'
commands=[];javap=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe')
for name in sorted(selected):
 command=[str(javap),'-J-Dfile.encoding=UTF-8','-p','-c','-constants','-classpath',str(jar),name]
 result=subprocess.run(command,capture_output=True,cwd=out,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
 prefix=name.rsplit('.',1)[-1]
 logs={}
 for suffix,blob in [('stdout',result.stdout),('stderr',result.stderr)]:
  path=out/(prefix+'.'+suffix+'.txt');path.write_bytes(blob);logs[str(path)]=sha(path)
 commands.append(dict(class_name=name,command=command,exit_code=result.returncode,logs=logs))
 assert result.returncode==0,name
record=dict(status='constant_pool_candidate_callers_and_original_bytecode_only',source_manifest_sha256=SOURCE,
 formal_target_sha256=FORMAL,jar=str(jar),javap=str(javap),javap_sha256=sha(javap),
 scanned_server_classes=len(classes),target_actor=actor,target_method_names=sorted(targets),
 constant_pool_candidate_callers=refs,target_field_references=field_refs,field_name_occurrences=reflective_names,
 disassembly=commands,classes_instantiated=0,
 original_initializer_invoked=False,tests_executed=0,database_started=False,SQL_executed=False,desktop_inputs=0,
 normal_gameplay_reachability_proven=False,full_attribute_matrix_qualified=False,whole_goal_complete=False,
 limitations=['Class references do not establish method-level reachability or packet authorization',
  'Original setters used by copy/bootstrap are not synthetic gameplay fixtures',
  'No server class, gameplay mutation, factory, SQL, client or case was executed'],recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'inspection.json').write_bytes(encoded(record));(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
assert sha(jar)==FORMAL
print(json.dumps(dict(inspection=str(out/'inspection.json'),scanned_classes=len(classes),candidate_caller_classes=len(refs),
 disassembled_classes=len(commands),tests_executed=0,classes_instantiated=0),ensure_ascii=False))
