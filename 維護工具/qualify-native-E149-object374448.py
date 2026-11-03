"""Qualify compiler-only actor spells 37/44/48 source work and publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-object374448-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_shadow107_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==22 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==5,'merged source projection differs')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 controller='LiTo381-Modern/src/modern_lito/native_login.rs';prior=w['current_native_item_spells_development']['paired_runtime_sources'][controller];paired=c['paired_runtime_sources'][controller]
 require(paired['predecessor']==prior and paired['rewrites']==prior['rewrites'] and len(prior['rewrites'])==5,'controller nested lineage differs')
 require(sha(prior['source'])==prior['source_sha256']==candidate['base_files'][controller] and sha(prior['path'])==prior['sha256'],'original controller pair changed')
 for info in [prior,paired]:
  restored=Path(info['path']).read_text('utf-8-sig')
  for rewrite in info['rewrites']:
   require(restored.count(rewrite['compiled_attribute'])==1,'compiled nested module differs');restored=restored.replace(rewrite['compiled_attribute'],rewrite['original_attribute'])
  require(restored==Path(info['source']).read_text('utf-8-sig') and sha(info['source'])==info['source_sha256'] and sha(info['path'])==info['sha256'],'controller pair differs beyond nested paths')
 require(paired['source']==str(cp.parent/controller),'paired source is not this candidate')
 for n in candidate['changed_runtime_files']:
  oldpath=Path(prior['path']) if n==controller else Path(baseline['candidate_paths'][n]);newpath=Path(paired['path']) if n==controller else cp.parent/n
  require(modern.count(oldpath.as_posix())==1,'old module mapping differs');modern=modern.replace(oldpath.as_posix(),newpath.as_posix())
 require(modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==362,'full graph differs beyond two runtime paths')
 require(Path(old['generated_root']).read_text('utf-8-sig').replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig'),'library root differs')
 for kind,name in [('object374448','native_object374448.rs')]:
  source=cp.parent/'LiTo381-Modern/tests'/name;text=source.read_text('utf-8-sig');require(Path(c['paired_tests'][kind]).read_text('utf-8-sig')==text.replace('include_bytes!("fixtures/native-world-current.json")','include_bytes!(r#"'+(P/'tests/fixtures/native-world-current.json').as_posix()+'"#)'),'paired QA differs beyond fixture path')
  require(len(re.findall(r'#\[test\]\s*fn\s+\w+',text))==9,'QA test count changed')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'];require([(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,k) for p in ['Debug','Release'] for k in kinds],'compiler target order differs')
 libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-object374448-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 require(contract['costs_cures_cancellation_and_weapon_stats_server_owned'] and contract['tests_executed']==0,'server-owned state scope differs')
 reviewed={'LiTo381-Modern/src/modern_lito/native_skills.rs':'AE90C50F71445ACA431350687E8873FE27196E1DE58EB1CA86AB2A9C5DF761F7','LiTo381-Modern/src/modern_lito/native_skill_selection.rs':'7472F55F74BF2C3977745FFD1462C1280DBA8F0822A555B07E3798CFF6539AB3','LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs':'59F196F84670ACFFE5CECD21FD4D09D664FF60D0274454FFA0A4A5F5A22C88E8','LiTo381-Modern/src/modern_lito/native_login.rs':'1D25BB1F0C5A10F7C8FCA004C87EC0E85246A2A311705DF3894C950FE4E012C2','LiTo381-Modern/tests/native_object374448.rs':'86785E6C4779C7E59F3ABA9538AD79C4B54BAD3E75A7341A280C6EE9C8E024B1'}
 require(candidate['candidate_files']==reviewed,'reviewed source diff changed; re-review required');check_pins({str(cp.parent/n):h for n,h in reviewed.items()})
 api=read(V/'native-E149-object374448-contract/20261024-033519-068975/inspection.json');require(api['formal_target_sha256']==FORMAL and api['game_classes_instantiated']==0 and len(api['commands'])==6 and all(x['exit_code']==0 for x in api['commands']),'original API inspection differs');check_pins(api['outputs'])
 catalog=(V/'native-E149-slow29-advance79-contract/20261024-002909-204866/BattleSpellCatalog.stdout.txt').read_text('utf-8-sig')
 expected={37:('CurseRemovalSpell',[37,4,4,19,227,20,10,-1,1],'remove-curse'),44:('CancellationSpell',[44,5,3,19,870,40,500,3,280,40318,2],'cancellation'),48:('WeaponBuffSpell',[48,5,7,19,2176,20,10,1200,-1,2,2,2,0,79,161],'bless-weapon')}
 for skill,(kind,expected_values,key) in expected.items():
  blocks=re.findall(r'(?m)^ +\d+: new +#\d+ +// class local/lito381/server/BattleSpellCatalog\$'+kind+r'\n([\s\S]*?)(?=^ +\d+: invokespecial)',catalog)
  values=[]
  for block in blocks:
   if '// String '+key+'\n' not in block:continue
   for row in block.splitlines():
    match=re.search(r': (?:bi|si)push +(-?\d+)',row);icon=re.search(r': iconst_(m1|[0-5])',row);long=re.search(r'// long (\d+)l',row)
    if match:values.append(int(match[1]))
    elif icon:values.append(-1 if icon[1]=='m1' else int(icon[1]))
    elif long:values.append(int(long[1]))
  require(values==expected_values,'original catalog numeric definition differs: '+str(skill))
 require([(e['graphics'],e['bank'],e['sound'],len(e['frames'])) for e in contract['effects']]==[(227,227,122,7),(870,232,140,7),(2176,2176,800,17)],'original effect definition differs')
 require(sha(V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt')=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','original ClientSession version differs')
 for profile,row in w['current_E149_integrated_client_development']['images'].items():require(sha(row['path'])==row['sha256'],'previous nineteen-file GUI PE changed')
 identities=[c['pipeline'],*[r['original_process'] for r in c['compiler_runs']]];ids=','.join(str(r['pid']) for r in identities)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler still live')
 require(len(read(c['base_compiled_inputs']))==1049,'normal input count differs');check_pins({str(R/n):h for n,h in read(c['base_compiled_inputs']).items()})
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed')
 tools=[T/n for n in ['prepare-native-E149-object374448.py','compile-native-E149-object374448.py','qualify-native-E149-object374448.py']]
 for p in tools:compile(ast.parse(p.read_text('utf-8-sig'),filename=str(p)),str(p),'exec')
 out=owned(V/'native-E149-object374448-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw)
 record=dict(status='unselected_object374448_actor_requests_and_finite_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=5,effective_overlay_files=22,new_tests_written=9,retained_skill_tests_written=37,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),original_definitions=contract['skills'],original_effects=contract['effects'],paired_runtime_sources=c['paired_runtime_sources'],original_nested_paths_preserved=6,
  source_pins={str(p):sha(p) for p in [*tools,T/'templates/native_object374448.rs',T/'native-E149-object374448.md']},original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_shadow107_qualification=w['current_E149_shadow107_development'],
  normal_sources_unchanged=True,normal_PEs_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; no current E149 gameplay/resource/audio/GUI execution','actor request intent and MP/material/eligibility/cures/cancellation/weapon effects stay server-owned','twenty-two-file projection retains current G06 Java without recompiling or executing Java','older linked twenty-one-file PE and portable ZIP do not contain these changes'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
 archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json'}
 archive.update({'tools/'+Path(p).name:Path(p) for p in record['source_pins']});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',Path(paired['path']),*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record))
 work=('新增#37解除詛咒／#44相消術／#48祝福魔法武器：角色C14分別14 04 04／14 05 03／14 05 07接u32 ID，無選擇用本人；37／48沿原native screen，44三格／500ms。'
  'MP20／40／20、44寶石40318×2與解除／相消／48武器資格及1200秒效果由E149決定；79裝備回饋補#8／#48，161不造成功ACK。'
  '227／870嚴格finite各原7frame與Sound122／140，870明確bank232；2176原17frame／Sound800保留。'
  '五檔四Runtime差分共22檔投影，保留六nested module；兩版library／library-test／七skill整合共18metadata退出0，新9＋原37案全部未執行。'
  '未選入registry／套用／連結PE／執行Java／測試／播放／GUI；正常1049輸入／兩EXE及G06 binding／16案640次不變，舊21檔PE及ZIP不含本批。'
  '使用者恢復測試後先元件／G06，再驗三技能原書、對己對人、低MP／材料／武器、範圍／取消／期限及原音效，重建當版library兩版及GUI、Release完整回歸才交人工。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_object374448_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='object374448_actor_requests_original_effects_source',work=work,applies_to_normal_EXE=False,goal_complete=False)
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
 print(json.dumps(dict(evidence=str(out/'evidence.json'),metadata_compilers=18,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();publish(a.run,a.terminal_chunk)
