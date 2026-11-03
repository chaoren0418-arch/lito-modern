"""Build an internal portable ZIP from pinned files; never execute its payload."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_pe_imports import inspect
T=R/'維護工具';V=R/'驗證紀錄/current'
WINDOWS_DLLS={'kernel32.dll','oleaut32.dll','ntdll.dll','bcrypt.dll','bcryptprimitives.dll','advapi32.dll','imm32.dll','comctl32.dll','user32.dll','gdi32.dll','ws2_32.dll','winmm.dll'}
DOCS=['https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method?view=msvc-170','https://learn.microsoft.com/en-us/cpp/windows/determining-which-dlls-to-redistribute?view=msvc-170']

def publish(inspection_path):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'workload/freeze active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'],'tests/desktop remain deferred')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 inspection_path=owned(inspection_path.resolve(strict=True));sdk=read(inspection_path);runtime_sdk=read(inspection_path.parent/'runtime-inspection.json')
 source_ref=w['current_E149_skill_library_link_development'];require(sdk['source_qualification']==source_ref and sha(source_ref['qualification'])==source_ref['qualification_sha256'],'linked source qualification changed');source=read(source_ref['qualification']);check_pins(source['source_pins'])
 require(sha(sdk['dumpbin'])==sdk['dumpbin_sha256'] and all(r['exit_code']==0 for r in sdk['commands']) and runtime_sdk['exit_code']==0,'SDK bytecode inspection failed/changed')
 for r in sdk['commands']:check_pins(r['outputs'])
 check_pins(runtime_sdk['outputs']);runtime=Path(runtime_sdk['runtime'])
 require(runtime==Path('C:/Program Files/Microsoft Visual Studio/18/Insiders/VC/Redist/MSVC/14.51.36231/x64/Microsoft.VC145.CRT/vcruntime140.dll') and sha(runtime)==runtime_sdk['runtime_sha256']=='D1F4225DF2CD877DBF130D5668A021DCE3F94118455FF5EC952061C30AFC9CE7','runtime source differs')
 inventory={}
 for tag,path in [('Debug',Path(source['images']['Debug']['path'])),('Release',Path(source['images']['Release']['path'])),('vcruntime',runtime)]:
  row=inspect(path);log=inspection_path.parent/(tag+'.stdout.txt');sdk_names={s.decode('ascii').lower() for s in re.findall(rb'^\s*([A-Za-z0-9_.-]+[.]dll)\s*$',log.read_bytes(),re.M|re.I)}
  require({x['dll'].lower() for x in row['imports']+row['delay_imports']}==sdk_names,'raw PE and SDK dependency sets differ');inventory[tag]=row
 exe=inventory['Release'];dll=inventory['vcruntime'];require(exe['sha256']==source['images']['Release']['sha256'] and exe['PE_kind']=='EXE' and dll['PE_kind']=='DLL','EXE/DLL identity differs')
 require(dll['fixed_file_version']['file']==[14,51,36247,0] and dll['fixed_file_version']['file'][:2]>=exe['linker_version'],'runtime/linker family mismatch')
 imports=[s for r in exe['imports']+exe['delay_imports'] if r['dll'].lower()=='vcruntime140.dll' for s in r['symbols']];names={n for r in dll['exports'] for n in r['names']};ordinals={r['ordinal'] for r in dll['exports']}
 missing=[s for s in imports if s.get('name') not in names and s.get('ordinal') not in ordinals];require(len(imports)==9 and not missing,'runtime lacks an imported entry')
 def system_contract(name):return name in WINDOWS_DLLS or name.startswith(('api-ms-win-','ext-ms-win-'))
 required={r['dll'].lower() for row in [exe,dll] for r in row['imports']+row['delay_imports']};unknown=sorted(n for n in required if n!='vcruntime140.dll' and not system_contract(n));require(not unknown,'unreviewed non-Windows dependency')
 exports_used=[r for r in dll['exports'] if any(n in {s.get('name') for s in imports} for n in r['names']) or r['ordinal'] in {s.get('ordinal') for s in imports}]
 forwarders=[r['forwarder'] for r in exports_used if r['forwarder']];require(not forwarders,'used runtime export forwards require further inspection')
 payload_readme=('LiTo Modern：內部可攜驗證候選\n\n'
  '目前未經乾淨主機、完整library／Release／GUI驗證，未交人工或對外發佈。正常開發入口仍是重建版的啟動現代客戶端.ps1。\n'
  '包內Release主程式與vcruntime140.dll放在同一目錄；沒有安裝或registry步驟。未附遊戲資源、伺服器或個人設定。\n'
  '後續自動驗證需先確認系統與動態DLL相依，再核對首次設定視窗及既有資源目錄。設定檔名稱為LiToModern.native.json。\n'
  '此DLL來自本機Visual Studio 18/Insiders的Redist，版本14.51.36247.0；此包僅留本地後續隔離驗證，不主張正式外發資格。\n').encode('utf-8')
 manifest=dict(schema='LiTo-internal-portable-payload-v1',candidate_only=True,external_release_qualified=False,
  source_manifest_sha256=SOURCE,logical_input_manifest_sha256=source['projected_input_manifest_sha256'],source_qualification_sha256=source_ref['qualification_sha256'],formal_target_sha256=FORMAL,
  files={'LiToModern.exe':exe['sha256'],'vcruntime140.dll':dll['sha256'],'README.txt':hashlib.sha256(payload_readme).hexdigest().upper()},
  runtime_version=dll['fixed_file_version']['file'],runtime_source_family='Visual Studio 18/Insiders Redist x64 Microsoft.VC145.CRT',
  Windows_static_contracts=sorted(required-{'vcruntime140.dll'}),dynamic_dependency_closure_proven=False,minimum_Windows_version_verified=False,
  payload_executed=False,clean_host_verified=False,GUI_verified=False,whole_goal_complete=False,Microsoft_deployment_references=DOCS)
 manifest_blob=encoded(manifest);package_id=hashlib.sha256(manifest_blob).hexdigest().upper();products=owned(P/'target/current/verification/native-E149-portable-release'/package_id[:24]);products.mkdir(parents=True,exist_ok=True)
 payload={'LiToModern.exe':Path(exe['path']).read_bytes(),'vcruntime140.dll':runtime.read_bytes(),'README.txt':payload_readme,'payload-manifest.json':manifest_blob};expected={n:hashlib.sha256(b).hexdigest().upper() for n,b in payload.items()};archive=products/'LiToModern-portable-internal.zip'
 if not archive.exists():
  with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=9) as z:
   for n,blob in payload.items():info=zipfile.ZipInfo(n,(2026,1,1,0,0,0));info.create_system=0;info.external_attr=0x20;info.compress_type=zipfile.ZIP_DEFLATED;z.writestr(info,blob,compresslevel=9)
 with zipfile.ZipFile(archive) as z:require(set(z.namelist())==set(expected) and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in expected.items()),'portable payload readback differs')
 out=owned(V/'native-E149-portable-candidate-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 (out/'payload-manifest.json').write_bytes(manifest_blob);(out/'raw-imports-and-exports.json').write_bytes(encoded(inventory))
 tools=[T/n for n in ['native_E149_pe_imports.py','package-native-E149-portable-candidate.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 record=dict(status='internal_portable_zip_static_dependencies_and_payload_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_qualification=source_ref,archive=str(archive),archive_sha256=sha(archive),archive_bytes=archive.stat().st_size,payload_file_hashes=expected,
  payload_manifest=str(out/'payload-manifest.json'),payload_manifest_sha256=sha(out/'payload-manifest.json'),raw_PE_inventory=str(out/'raw-imports-and-exports.json'),raw_PE_inventory_sha256=sha(out/'raw-imports-and-exports.json'),
  source_PE=source['images']['Release'],runtime_source=str(runtime),runtime_sha256=dll['sha256'],runtime_version=dll['fixed_file_version']['file'],runtime_preview_toolchain=True,runtime_imported_entries=9,runtime_export_entries=len(dll['exports']),missing_imported_entries=missing,unreviewed_static_dependencies=unknown,
  SDK_inspection=str(inspection_path),SDK_inspection_sha256=sha(inspection_path),runtime_SDK_inspection=str(inspection_path.parent/'runtime-inspection.json'),runtime_SDK_inspection_sha256=sha(inspection_path.parent/'runtime-inspection.json'),
  source_pins={str(p):sha(p) for p in [*tools,T/'native-E149-portable-candidate.md',P/'src/main.rs',P/'src/modern_lito/native_startup.rs']},Microsoft_deployment_references=DOCS,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],candidate_registry_unchanged=w['current_isolated_development']['candidates'],
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_or_DLLs_executed=False,payload_extracted=False,VC_runtime_installed=False,registry_or_System32_modified=False,dynamic_dependency_closure_proven=False,minimum_Windows_version_verified=False,clean_host_verified=False,GUI_verified=False,desktop_inputs=0,external_release_qualified=False,whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 raw_files={'qualified-development.json':out/'qualified-development.json','payload-manifest.json':out/'payload-manifest.json','raw-imports-and-exports.json':out/'raw-imports-and-exports.json','inspection.json':inspection_path,'runtime-inspection.json':inspection_path.parent/'runtime-inspection.json'}
 raw_files.update({'source/'+Path(n).name:Path(n) for n in record['source_pins']})
 for row in sdk['commands']+[runtime_sdk]:
  for p in row['outputs']:raw_files['SDK/'+Path(p).name]=Path(p)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in raw_files.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw_files.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('Release可攜靜態缺口已處理為內部候選：SDK與raw PE依賴清單一致，AA766180需VCRUNTIME140.dll；九個具名匯入都在所選x64 runtime的71項exports。'
  'DLL取本機VS18/Insiders Redist，實際版本14.51.36247.0／SHA D1F4225D；僅留本地隔離驗證，不主張正式外發資格。'
  'ZIP只含Release EXE／runtime／說明／manifest，逐entry SHA讀回；不含資源／JAR／帳密／設定／測試EXE，未解壓／載入／執行／安裝或改registry。'
  '原入口已有程式旁設定與首次視窗，未另加重複流程。Windows靜態契約不等於實機解析，動態載入、最低Windows與clean host／GUI仍未驗。'
  '正常來源／所有PE、G06 binding／16案不變；測試與桌面暫緩維持。恢復後先元件／G06與當版library兩版、整合／Release／完整GUI，最後clean host可攜及逐要求稽核才交人工。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_portable_candidate_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='internal_portable_release_static_dependency_and_payload_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 baseline=read(w['current_development_typecheck_qualification']['qualification']);inputs=read(Path(baseline['run'])/'base-compiled-inputs.json');require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed');check_pins(baseline['normal_product_pins'])
 require(all(sha(source['images'][p]['path'])==source['images'][p]['sha256'] for p in ['Debug','Release']),'source images changed');require(sha(runtime)==dll['sha256'],'runtime changed')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()),'concurrent checkpoint writer')
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),archive=str(archive),archive_sha256=sha(archive),runtime_version=dll['fixed_file_version']['file'],runtime_entries_matched=9,tests_executed=0,payload_executed=False,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('inspection',type=Path);a=p.parse_args();publish(a.inspection)
