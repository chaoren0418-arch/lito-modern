"""Scan actual Code instructions for timer writers/callers; execute no game API."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,traceback,zipfile
from native_E149_recovery_contract import R,P,FORMAL,SOURCE,read,sha,encoded,owned,require
from native_E149_classfile_scan import parse,instructions,member
T=R/'維護工具';V=R/'驗證紀錄/current'
FIELDS={'_magicStoneExpireFuture','_equipmentTimerFuture','_activeEquipmentTimer','_pendingEquipmentExpiry',
 '_magicStoneExpirySubmissionInFlight','_magicStoneExpiryRetryScheduled','_pendingMagicStoneExpiryGeneration'}
NAMES={'cancelMagicStoneExpiryTimer','stopEquipmentTimerSerialized','cancelActiveEquipmentTimerLocked',
 'terminalizeEquipmentLocked','applyMagicStoneExpiryIfNeeded','beginMagicStoneMutationLocked','clearMagicStoneFieldsOnly','clearMagicStone'}

def main():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'offline development scope required')
 jar=owned(w['current_formal_reference_identity']['cached_reference']);require(sha(jar)==FORMAL==sha(w['current_formal_reference_identity']['formal']),'formal changed')
 out=owned(V/'native-E149-item-timer-routes'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 for n in ['inspect-native-E149-item-timer-routes.py','native_E149_classfile_scan.py']:(out/n).write_bytes((T/n).read_bytes())
 facts=dict(field_writes=[],lifetime_calls=[],schedule_calls=[],reflective_writes=[],method_handles=[],target_name_occurrences=[],class_inventory=[]);commands=[];last=None
 try:
  with zipfile.ZipFile(jar) as z:
   entries=[e for e in z.infolist() if e.filename.startswith('local/lito381/server/') and e.filename.endswith('.class')];require(len({e.filename for e in entries})==len(entries),'duplicate server class entries')
   for entry in entries:
    last=dict(class_name=entry.filename,method=None);data=z.read(entry);c=parse(data);require(c['class_name']+'.class'==entry.filename,'classfile/JAR identity differs');cp=c['cp'];instruction_count=0;methods_with_code=0
    occurrence=sorted({v[1] for v in cp if v and v[0]==1 and v[1] in FIELDS})
    if occurrence:facts['target_name_occurrences'].append(dict(class_name=c['class_name'],names=occurrence))
    for index,v in enumerate(cp):
     if v and v[0]==15:
      ref=member(cp,v[2])
      if ref['name'] in FIELDS|NAMES or 'EquipmentTimerLease' in ref['owner']:facts['method_handles'].append(dict(class_name=c['class_name'],cp_index=index,reference_kind=v[1],member=ref))
    for method in c['methods']:
     last['method']=method['name']+method['descriptor']
     if method['code'] is None:continue
     methods_with_code+=1;rows=instructions(method['code']['bytes'],cp);instruction_count+=len(rows)
     for index,row in enumerate(rows):
      if 'member' not in row:continue
      ref=row['member'];base=dict(class_name=c['class_name'],method=method['name'],descriptor=method['descriptor'],code_sha256=hashlib.sha256(method['code']['bytes']).hexdigest().upper(),offset=row['offset'],opcode=row['opcode'],member=ref,context=rows[max(0,index-5):index+1])
      if row['opcode'] in [0xb3,0xb5] and ref['name'] in FIELDS:
       previous=rows[index-1] if index else {};base['immediately_preceding_constant']='constant' in previous
       if 'constant' in previous:base['constant_value']=previous['constant']
       facts['field_writes'].append(base)
      if 0xb6<=row['opcode']<=0xb9:
       if ref['name'] in NAMES or ref['name']=='<init>' and 'EquipmentTimerLease' in ref['owner']:facts['lifetime_calls'].append(base)
       if ref['name'] in ['schedule','scheduleAtFixedRate','scheduleWithFixedDelay']:facts['schedule_calls'].append(base)
       if ref['owner']=='java/lang/reflect/Field' and ref['name'] in ['set','setBoolean','setInt','setLong','setByte','setShort','setChar','setFloat','setDouble'] or 'VarHandle' in ref['owner'] and ref['name'].startswith(('set','compareAnd','getAndSet')) or 'Unsafe' in ref['owner'] and ref['name'].startswith('put') or 'MethodHandles$Lookup' in ref['owner'] and ref['name'] in ['findSetter','findStaticSetter','unreflectSetter','findVarHandle','findStaticVarHandle']:
        facts['reflective_writes'].append(base)
    facts['class_inventory'].append(dict(path=entry.filename,class_sha256=hashlib.sha256(data).hexdigest().upper(),methods=len(c['methods']),methods_with_code=methods_with_code,instructions=instruction_count))
  require(len(facts['class_inventory'])==len(entries),'server class scan incomplete')
  for name,value in facts.items():(out/(name+'.json')).write_bytes(encoded(value))
  selected={x['class_name'] for x in facts['field_writes']}
  selected|={'local/lito381/server/NativeDeWorldKinds$Item$EquipmentTimerLease','local/lito381/server/NativeDeItemMagicStone','local/lito381/server/NativeDeItemInventory'}
  require(len(selected)<=10,'new writer class scope needs review before javap')
  javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe');require(sha(javap)=='09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A','javap changed')
  for name in sorted(selected):
   command=[str(javap),'-J-Dfile.encoding=UTF-8','-p','-c','-constants','-classpath',str(jar),name.replace('/','.')];result=subprocess.run(command,capture_output=True,cwd=out,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS);logs={}
   for kind,blob in [('stdout',result.stdout),('stderr',result.stderr)]:
    p=out/(name.rsplit('/',1)[-1]+'.'+kind+'.txt');p.write_bytes(blob);logs[str(p)]=sha(p)
   commands.append(dict(class_name=name,command=command,exit_code=result.returncode,logs=logs));(out/'commands.json').write_bytes(encoded(commands));require(result.returncode==0,'original javap failed')
  require(sha(jar)==FORMAL,'formal cache changed')
  record=dict(status='decoded_original_Code_timer_writers_and_candidate_routes_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,jar=str(jar),scanned_server_classes=len(entries),scanned_methods_with_code=sum(x['methods_with_code'] for x in facts['class_inventory']),decoded_instructions=sum(x['instructions'] for x in facts['class_inventory']),target_fields=sorted(FIELDS),field_write_instructions=len(facts['field_writes']),lifetime_call_instructions=len(facts['lifetime_calls']),schedule_call_instructions=len(facts['schedule_calls']),reflective_write_instructions=len(facts['reflective_writes']),matching_method_handles=len(facts['method_handles']),commands=commands,outputs={str(p):sha(p) for p in out.iterdir() if p.is_file()},tests_executed=0,classes_instantiated=0,timer_or_game_methods_invoked=False,normal_GAME_timer_reachability_proven=False,desktop_inputs=0,whole_goal_complete=False,
   limitations=['Explicit Code writes/calls are decoded; constant-pool names alone are not counted as executed instructions','Immediate preceding constants are local instruction facts, not a full stack/dataflow proof','Generic reflection, method handles, dynamic loading and external callers remain separate unresolved paths','No actual normal GAME route, timer, copy or natural expiry ran'])
  (out/'inspection.json').write_bytes(encoded(record));print(json.dumps({k:record[k] for k in ['scanned_server_classes','scanned_methods_with_code','decoded_instructions','field_write_instructions','lifetime_call_instructions','schedule_call_instructions','tests_executed']}|dict(inspection=str(out/'inspection.json')),ensure_ascii=False))
 except BaseException:
  (out/'original-failure.json').write_bytes(encoded(dict(last=last,error=traceback.format_exc(),tests_executed=0)));raise

if __name__=='__main__':main()
