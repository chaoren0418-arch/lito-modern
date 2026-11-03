"""Qualify compiler-only #107 source work and publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-shadow107-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='sixteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_magical73_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==21 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==5,'merged source projection differs')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for n in candidate['changed_runtime_files']:require(modern.count(Path(baseline['candidate_paths'][n]).as_posix())==1,'old module mapping differs');modern=modern.replace(Path(baseline['candidate_paths'][n]).as_posix(),(cp.parent/n).as_posix())
 require(modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==362,'full graph differs beyond two runtime paths')
 require(Path(old['generated_root']).read_text('utf-8-sig').replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig'),'library root differs')
 for kind,name in [('item_spells','native_item_spells.rs'),('shadow107','native_shadow107.rs')]:
  source=cp.parent/'LiTo381-Modern/tests'/name;text=source.read_text('utf-8-sig');require(Path(c['paired_tests'][kind]).read_text('utf-8-sig')==text.replace('include_bytes!("fixtures/native-world-current.json")','include_bytes!(r#"'+(P/'tests/fixtures/native-world-current.json').as_posix()+'"#)'),'paired QA differs beyond fixture path')
  require(len(re.findall(r'#\[test\]\s*fn\s+\w+',text))==(6 if kind=='shadow107' else 8),'QA test count changed')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107'];require([(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,k) for p in ['Debug','Release'] for k in kinds],'compiler target order differs')
 libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-shadow107-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 require(tuple(contract[k] for k in ['skill','row','column','MP_base','reuse_millis','server_duration_seconds','material_id','material_count','invalid_item_message','success_message','expiration_message'])==(107,13,2,30,0,192,40321,1,79,0,308),'formal item spell values differ')
 require(contract['material_name']=='二級黑魔石' and len(contract['frames'])==13 and contract['audio_sound']==1510 and contract['attrs']==[[102,[48]],[104,[56]]],'original sprite/material contract differs')
 bank='LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs';oldbank=Path(baseline['candidate_paths'][bank]).read_text('utf-8-sig')
 def nums(values):return '['+','.join(str(n) for n in values)+']'
 frames=','.join('('+','.join([str(a),str(b),str(t),'&'+nums(sounds)])+')' for a,b,t,sounds in contract['frames'])
 spec='Spec{sprite:2951,bank:2951,header:&'+nums(contract['original_header'])+',attrs:&[(102,&[48]),(104,&[56])],frames:&['+frames+']},\n'
 require(oldbank.replace('6342,752,747,748];','6342,752,747,748,2951];').replace('const SPECS:&[Spec]=&[\n','const SPECS:&[Spec]=&[\n'+spec)==(cp.parent/bank).read_text('utf-8-sig'),'finite bank differs beyond exact2951 spec')
 api=read(V/'native-E149-item-spell-contract/20261024-014418-307290/inspection.json');require(api['formal_target_sha256']==FORMAL and all(r['exit_code']==0 for r in api['commands']),'original action/catalog API inspection failed');check_pins(api['outputs'])
 require(sha(V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt')=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','original full ClientSession version differs')
 for profile,row in w['current_E149_integrated_client_development']['images'].items():require(sha(row['path'])==row['sha256'],'previous nineteen-file GUI PE changed')
 identities=[c['pipeline'],*[r['original_process'] for r in c['compiler_runs']]];ids=','.join(str(r['pid']) for r in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler still live')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['prepare-native-E149-shadow107.py','compile-native-E149-shadow107.py','qualify-native-E149-shadow107.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 out=owned(V/'native-E149-shadow107-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw)
 record=dict(status='unselected_shadow107_inventory_target_and_2951_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*16,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=5,effective_overlay_files=21,new_tests_written=6,retained_skill_tests_written=31,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),wire_prefix=[20,13,2],base_MP=30,material40321_count=1,material_name='二級黑魔石',server_duration_seconds=192,reuse_millis=0,effect_graphics=2951,effect_frame_count=13,original_sound=1510,
  source_pins={str(p):sha(p) for p in [*tools,T/'templates/native_shadow107.rs',T/'native-E149-shadow107.md']},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_magical73_qualification=w['current_E149_magical73_development'],
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; no current E149 gameplay/resource/audio/GUI execution','inventory intent and MP/material/item eligibility/effect stay server-owned','twenty-one-file projection retains current G06 Java sources without recompiling or executing Java','original two preparation failures preserved; neither started rustc'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'tools/'+Path(p).name:Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 preparation=V/'native-E149-shadow107-preparation-failures'
 for folder in preparation.iterdir():
  for p in folder.iterdir():
   if p.is_file():archive['preparation-failures/'+folder.name+'/'+p.name]=p
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('補#107暗影之牙五檔差分：三Runtime、原item QA及新增六案；C14位元組14 0D 02接unsigned背包物品ID，沿typed ItemTarget／明確選取／原controller gate。'
  '原MP30基值、40321二級黑魔石×1、192秒及武器資格／+5效果均由E149決定，reuse0，原79拒絕沿物品Pending且不造成功ACK。'
  '2951嚴格finite接原13影格／可變ticks、102.type(0)與104.attr(8)、Sound1510及既有音效／換代退役。'
  '保留#73並換入目前G06三Java來源後共21檔來源投影；兩版library／library-test／六skill整合共16metadata退出0，新6＋原31案全未跑。'
  '兩次準備來源引用錯誤完整保留且未啟動Rust；新來源未選入／套用／連結PE／施法／播放，正常1049輸入／所有PE及G06 binding／16案640次不變。'
  '測試與桌面暫緩維持；恢復後元件／G06，再驗#107材料／低MP／舊物品／取消／原192秒／2951及GUI，完成當版完整library兩版、Release與完整GUI後才交人工。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_shadow107_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='shadow107_inventory_target_and_original_2951_source',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'progress document format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'checkpoint before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()),'concurrent checkpoint writer');check_pins(c['normal_product_pins'])
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
 print(json.dumps(dict(evidence=str(out/'evidence.json'),metadata_compilers=16,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();publish(a.run,a.terminal_chunk)
