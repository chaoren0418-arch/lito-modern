"""Read back static catalog facts against original javap and publish progress only."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,traceback,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_classfile_scan import parse,instructions
T=R/'維護工具';V=R/'驗證紀錄/current';CAT='local/lito381/server/BattleSpellCatalog'

def publish(inspection,tool_chunk):
 inspection=owned(inspection.resolve(strict=True),V/'native-E149-skill-catalog-audit');q=read(inspection);w=read(R/'工作佇列.json')
 require(q['status']=='E149_catalog_literal_maps_and_source_recognition_static_only' and q['source_manifest_sha256']==SOURCE and q['formal_target_sha256']==FORMAL,'static audit identity differs');check_pins(q['source_pins'])
 require(q['source_qualification']==w['current_E149_object374448_development'] and q['tests_executed']==0 and not q['Java_classes_loaded'] and not q['Java_methods_invoked'] and not q['per_skill_dispatch_or_packet_contract_verified'],'audit source/execution scope differs')
 require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'workload/user deferral changed')
 require(sha(q['jar'])==FORMAL==sha(w['current_formal_reference_identity']['formal']) and sha(q['facts'])==q['facts_sha256'],'formal/facts changed');facts=read(q['facts'])
 require((q['catalog_literal_definitions'],q['catalog_literal_unique_ids'],q['candidate_missing_catalog_count'],q['factory_count'],q['ClientSession_factory_call_sites'],q['classfiles_read'])==(180,180,132,122,363,124),'current static count differs');require(not q['catalog_skill_maps_with_unmapped_objects'] and len(facts['unmapped_entries'])==q['opaque_non_skill_entries']==31,'unmapped scope differs')
 normal=q['normal_request']['all_request_ids'];candidate=q['candidate_request']['all_request_ids'];require(len(normal)==38 and len(candidate)==48 and set(normal)<set(candidate)<=set(q['catalog_literal_ids']),'recognition set differs');require(q['candidate_missing_catalog_ids']==sorted(set(q['catalog_literal_ids'])-set(candidate)),'missing set differs')
 jar=Path(q['jar'])
 with zipfile.ZipFile(jar) as z:
  for name,h in facts['class_pins'].items():require(hashlib.sha256(z.read(name+'.class')).hexdigest().upper()==h,'original class bytes changed')
  c=parse(z.read(CAT+'.class'));clinit=next(m for m in c['methods'] if m['name']=='<clinit>');rows=instructions(clinit['code']['bytes'],c['cp'])
 require(hashlib.sha256(clinit['code']['bytes']).hexdigest().upper()==facts['initializer_code_sha256'] and len(rows)==facts['initializer_instructions']==5757,'initializer differs')
 primary=V/'native-E149-slow29-advance79-contract/20261024-002909-204866/BattleSpellCatalog.stdout.txt';require(sha(primary)=='158171905884F2794357B7EE5DCF556B92B349D372F8267BE19858E424029DC7','original javap changed');text=primary.read_text('utf-8-sig');require(text.count('  static {};')==1,'initializer javap header differs');block=text.split('  static {};',1)[1];observed=[]
 for line in block.splitlines():
  match=re.match(r'^\s+(\d+):\s+(\w+)(.*)$',line)
  if match:observed.append((int(match[1]),match[2],match[3].strip()))
 names={0x02:'iconst_m1',**{n:'iconst_'+str(n-3) for n in range(3,9)},0x09:'lconst_0',0x0f:'dconst_1',0x10:'bipush',0x11:'sipush',0x13:'ldc_w',0x14:'ldc2_w',0x53:'aastore',0x59:'dup',0xb1:'return',0xb2:'getstatic',0xb3:'putstatic',0xb7:'invokespecial',0xb8:'invokestatic',0xbb:'new',0xbd:'anewarray'}
 require(len(observed)==len(rows),'javap instruction count differs');comparisons=[]
 for row,(offset,name,operand) in zip(rows,observed):
  require(row['offset']==offset and names[row['opcode']]==name,'javap offset/opcode differs')
  if 'cp_index' in row:require(re.match(r'^#'+str(row['cp_index'])+r'(?:\s|,|$)',operand),'javap constant-pool reference differs')
  if row['opcode'] in [0x10,0x11]:require(int(operand)==row['constant'],'javap immediate constant differs')
  comparisons.append(dict(offset=offset,opcode=name,cp_index=row.get('cp_index')))
 base=w['current_development_typecheck_qualification'];require(sha(base['qualification'])==base['qualification_sha256'],'normal baseline changed');baseline=read(base['qualification']);normal_inputs=read(Path(baseline['run'])/'base-compiled-inputs.json');require(len(normal_inputs)==1049,'normal input count differs');check_pins({str(R/n):h for n,h in normal_inputs.items()});check_pins(baseline['normal_product_pins'])
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/'audit-native-E149-skill-catalog.py',T/'qualify-native-E149-skill-catalog.py',T/'native_E149_classfile_scan.py'];
 for path in tools:compile(ast.parse(path.read_text('utf-8-sig'),filename=str(path)),str(path),'exec')
 out=owned(V/'native-E149-skill-catalog-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'javap-readback.json').write_bytes(encoded(dict(primary=str(primary),primary_sha256=sha(primary),compared_instructions=comparisons,scope='offset/opcode/constant-pool-reference and immediate push comparison only')))
 record=dict(status='current_E149_literal_skill_catalog_and_request_gap_static_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,inspection=str(inspection),inspection_sha256=sha(inspection),facts=q['facts'],facts_sha256=q['facts_sha256'],source_qualification=q['source_qualification'],catalog_literal_definitions=180,catalog_literal_unique_ids=180,candidate_request_recognized_count=48,normal_request_recognized_count=38,candidate_missing_request_ids=q['candidate_missing_catalog_ids'],candidate_missing_request_count=132,normal_missing_request_count=142,factory_count=122,ClientSession_factory_call_sites=363,original_classfiles_read=124,initializer_javap_instructions_compared=5757,initializer_object_allocations=facts['allocated_objects'],initializer_static_fields=facts['initialized_static_fields'],opaque_non_skill_summon_entries=31,unmapped_skill_objects=0,observed_original_tool_chunk=tool_chunk,observed_actual_exit_code=0,javap_readback=str(out/'javap-readback.json'),javap_readback_sha256=sha(out/'javap-readback.json'),source_pins={str(path):sha(path) for path in [*tools,T/'native-E149-skill-catalog-audit.md',primary]},normal_product_pins=baseline['normal_product_pins'],normal_source_input_count=1049,current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],next_skill_source_review=[22,25,30,53],tests_executed=0,Java_classes_loaded=False,Java_methods_invoked=False,learned_book_verified=False,per_skill_dispatch_or_packet_contract_verified=False,normal_sources_unchanged=True,normal_PEs_unchanged=True,PEs_linked=False,outputs_executed=False,production_applied=False,GUI_verified=False,WAV_playbacks=0,desktop_inputs=0,whole_goal_complete=False,limitations=q['limitations'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archives={'qualified-development.json':out/'qualified-development.json','javap-readback.json':out/'javap-readback.json'}
 for folder in sorted((V/'native-E149-skill-catalog-audit').iterdir()):
  require(folder.is_dir() and not folder.is_symlink(),'unexpected audit archive entry')
  for file in folder.iterdir():
   require(file.is_file() and not file.is_symlink(),'unexpected audit source entry');archives['audit-runs/'+folder.name+'/'+file.name]=file
 archives.update({'tools/'+path.name:path for path in [*tools,T/'native-E149-skill-catalog-audit.md']})
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name,path in archives.items():z.write(owned(path),name)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in archives.items()),'raw archive readback differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('更新當版E149技能缺口，直接classfile／Code解析127個static欄、231個物件與180個技能Map字面定義，原javap5757指令offset／opcode／CP及立即值核對。'
  '正常請求識別38、最新22檔候選48，當版目錄仍有132個ID無候選請求入口；非技能召喚31個helper保留opaque，122factory／363ClientSession呼叫僅方法層索引，不宣稱branch或參數資料流。'
  '初次單Map假設失敗與中間162項未展開AttackSpell委派結果完整保留；最新source已處理兩種same-class委派與原預設值。'
  '此清單取代歷史75之37個缺口作為當版開發依據，仍無E149技能書／玩法／GUI資格；下一玩法群先核對#22／#25／#30／#53之真正Protocol reader、目標／範圍、reuse及原資源，再實作。'
  'G06仍為主線；正常1049輸入／所有PE、22檔候選、原registry與binding／16案640次不變，測試／桌面／Java／fixture／DB／播放均未執行；使用者恢復測試後依元件／G06、當版library兩版、Release與完整GUI順序，確認才交人工。')
 staged={}
 for name in before:
  path=R/name
  if path.suffix=='.json':
   value=read(path);value['current_E149_skill_catalog_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='current_E149_skill_catalog_and_missing_request_source',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if name.endswith('next-validation-action.json'):value['next_action']=work
   staged[name]=encoded(value)
  else:
   _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs');staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name in before:z.write(R/name,name)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items()),'before archive differs')
 require(all(sha(R/name)==h for name,h in before.items()),'concurrent checkpoint writer');check_pins(record['source_pins']);check_pins(baseline['normal_product_pins'])
 for name,blob in staged.items():(R/name).write_bytes(blob)
 require(all((R/name).read_bytes()==blob for name,blob in staged.items()),'eight-file readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={name:sha(R/name) for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),catalog_ids=180,candidate_recognized=48,missing_candidate_requests=132,javap_instructions_compared=5757,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('inspection',type=Path);parser.add_argument('--tool-chunk',required=True);args=parser.parse_args();publish(args.inspection,args.tool_chunk)
