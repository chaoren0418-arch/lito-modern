"""Read back #23/#36 source and 18 metadata commands, then publish eight progress files."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def publish(run,terminal_chunk):
 run=owned(run.resolve(strict=True),V/'native-E149-reveal23-tame36-metadata');c=read(run/'cohort.json');cp=Path(c['candidate']);candidate=read(cp);w=read(R/'工作佇列.json')
 require(c['status']=='eighteen_metadata_compilers_complete_no_tests' and c['owned_process'] is None and sha(cp)==c['candidate_sha256'],'metadata build incomplete or changed')
 require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'compiler-only scope changed')
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'source identity differs')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency changed')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'formal/workload changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'tests must remain deferred')
 require(candidate['previous_skill_qualification']==w['current_E149_curse33_drain39_development'] and candidate['current_fixture_binding']==w['current_E149_fixture_binding'],'base skill/current fixture context changed')
 old=read(c['original_metadata_cohort']);require(sha(c['original_metadata_cohort'])==c['original_metadata_cohort_sha256'],'original compiler graph changed')
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed');baseline=read(candidate['base_overlay']);expected_files=dict(baseline['candidate_files']);expected_paths=dict(baseline['candidate_paths'])
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(baseline['candidate_files'][n]==h and baseline['candidate_paths'][n]==binding['current_effective_overlay_paths'][n],'current Java source projection differs')
 for n,h in candidate['candidate_files'].items():expected_files[n]=h;expected_paths[n]=str(cp.parent/n)
 require(len(expected_files)==29 and candidate['effective_overlay_files']==expected_files and candidate['effective_overlay_paths']==expected_paths and len(candidate['candidate_files'])==4,'merged source projection differs')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for n in candidate['changed_runtime_files']:
  base=Path(baseline['candidate_paths'].get(n,str(R/n)));needles=list(dict.fromkeys([str(base),base.as_posix()]));found=[needle for needle in needles if modern.count(needle)==1]
  require(candidate['base_files'][n]==sha(base),'raw module predecessor changed')
  if not found:
   alias=c['module_predecessor_aliases'].get(n);require(alias is not None and alias['raw_source']==str(base) and alias['raw_source_sha256']==sha(base),'generated predecessor alias absent');actual=owned(alias['path']);require(alias['sha256']==old['input_pins'].get(str(actual))==sha(actual)==sha(base) and actual.read_bytes()==base.read_bytes(),'generated predecessor alias differs');needles=[str(actual),actual.as_posix()];needles=list(dict.fromkeys(needles));found=[needle for needle in needles if modern.count(needle)==1]
  require(len(found)==1 and sum(modern.count(needle) for needle in needles)==1,'normal module mapping differs');modern=modern.replace(found[0],(cp.parent/n).as_posix())
 require(not c['module_predecessor_aliases'],'new runtime mapping unexpectedly needs aliases')
 require(c['paired_runtime_sources']==old['paired_runtime_sources'],'six nested module paths changed')
 require(modern==Path(c['generated_mod']).read_text('utf-8-sig') and len(re.findall(r'#\[path = r#"',modern))==362,'full graph differs beyond the reviewed runtime path')
 require(Path(old['generated_root']).read_text('utf-8-sig').replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix())==Path(c['generated_root']).read_text('utf-8-sig'),'library root differs')
 require(c['paired_tests']==old['paired_tests'] and len(c['paired_tests'])==7,'retained seven integrations changed')
 kinds=['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'];require([(r['profile'],r['kind']) for r in c['compiler_runs']]==[(p,k) for p in ['Debug','Release'] for k in kinds],'compiler target order differs')
 libraries={p:next(r['product'] for r in c['compiler_runs'] if r['profile']==p and r['kind']=='library') for p in ['Debug','Release']}
 for row in c['compiler_runs']:
  require(row['exit_code']==0 and row['finished_utc'] and not row.get('original_handle_cleanup'),'compiler not normally finished');check_pins(row['logs']);product=owned(row['product'],P/'target/current/verification/native-E149-reveal23-tame36-metadata');require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'metadata output changed')
  command=list(next(r['command'] for r in old['compiler_runs'] if r['profile']==row['profile'] and r['kind']=='library'));kind=row['kind'];command[command.index(old['generated_root'])]=c['generated_root'] if kind in ['library','library-test'] else c['paired_tests'][kind];command[command.index('-o')+1]=row['product']
  if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
  else:require(product.name.startswith('lib') and row['product_bytes']>0,'library metadata absent')
  if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+libraries[row['profile']]]
  require(command==row['command'] and '--emit=metadata' in command and '--emit=link' not in command and '--cfg' not in command,'actual compiler command differs')
 contract=read(candidate['contract']);require(sha(candidate['contract'])==candidate['contract_sha256'],'contract changed');require(all(sha(p)==h for p,h in contract['original_pins'].items()),'original read-only sources changed')
 names={'LiTo381-Modern/src/modern_lito/'+n for n in ['native_skills.rs','native_skill_selection.rs','native_additional_finite_effects.rs','native_world.rs']};require(set(candidate['changed_runtime_files'])==set(contract['source_changes'])==names,'runtime scope differs');tests=0;ignored=0
 for name,replacements in contract['source_changes'].items():
  source=Path(baseline['candidate_paths'][name]).read_text('utf-8-sig')
  for before,after in replacements:require(source.count(before)==1,'source anchor differs');source=source.replace(before,after)
  if name in contract['QA_templates']:
   template=contract['QA_templates'][name];require(sha(template['path'])==template['sha256'],'QA template changed');qa=Path(template['path']).read_text('utf-8-sig');tests+=len(re.findall(r'#\[test\]',qa));ignored+=qa.count('#[ignore=');source+=qa
  require(source==(cp.parent/name).read_text('utf-8-sig'),'runtime differs beyond reviewed changes')
 require(tests==10 and ignored==1 and all(contract[k] for k in ['both_require_explicit_other_actor','native_screen_not_unbounded','NPC_domain_and_companion_state_server_owned','no_local_cost_probability_or_pet_creation','existing_2510_and_S23_sources_preserved','capacity_message_319_classified']),'source/QA scope differs')
 proof_path=Path(contract['native_snapshot_contract']);require(sha(proof_path)==contract['native_snapshot_contract_sha256'],'formal source changed');proof=read(proof_path);check_pins(proof['source_pins']);require(proof['definitions']==contract['formal_definitions'] and proof['request_bytes']=={'23':7,'36':11} and proof['wire_prefixes']=={'23':[20,2,6],'36':[20,4,3]} and proof['caster_action']==19 and proof['target_effect_opcode']==55 and proof['tests_executed']==proof['methods_executed']==0,'formal contract differs')
 import importlib.util
 spec=importlib.util.spec_from_file_location('reveal23_source',T/'prepare-native-E149-reveal23-tame36.py');prep=importlib.util.module_from_spec(spec);spec.loader.exec_module(prep);originals=prep.methods(V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt');protocol=prep.methods(V/'native-E149-area2253053-contract/Protocol.stdout.txt')
 for path,h in proof['source_pins'].items():
  p=Path(path)
  if p.parent==proof_path.parent and p.name.endswith('.javap.txt'):
   name=p.name.removesuffix('.javap.txt');expected=protocol[name.removeprefix('Protocol.')] if name.startswith('Protocol.') else originals[name];require(p.read_text('utf-8-sig')==expected,'method extract differs from original')
 for name,offsets in proof['method_offset_assertions'].items():
  for offset,needle in offsets.items():require(re.search(r'^\s*'+str(offset)+r':[^\n]*'+re.escape(needle),originals[name],re.M),'formal branch changed')
 catref=w['current_E149_skill_catalog_development'];require(candidate['current_catalog_qualification']==catref and sha(catref['qualification'])==catref['qualification_sha256'],'catalog context changed');catalog=read(catref['qualification']);require(sha(catalog['facts'])==catalog['facts_sha256'],'catalog facts changed');facts=read(catalog['facts']);require(proof['definitions']=={str(n):next(r['definitions'][0]['core'] for r in facts['definitions_by_id'] if r['skill_id']==n) for n in [23,36]},'formal definitions changed')
 require(sha(proof['original_resource'])==proof['original_resource_sha256']=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','original resource changed');lines=Path(proof['original_resource']).read_bytes().splitlines();require(proof['finite_effects']==contract['finite_effects'] and [e['graphics'] for e in proof['finite_effects']]==[228,2166,2167,2168,2169],'finite scope differs');specs=[];nums=lambda vs:'['+','.join(map(str,vs))+']'
 for e in proof['finite_effects']:
  block=lines[e['source_line_start']-1:e['source_line_end']];require([list(l) for l in block]==e['block'] and list(block[0])==e['header'],'original sprite block differs');body=b'\n'.join(block);frames=[(int(a),int(b),int(t),[] if not s else [int(s)]) for a,b,t,s in re.findall(rb'(\d+)[.](\d+):(\d+)(?:\[(\d+))?',body)];attrs=[(int(k),list(v)) for k,v in re.findall(rb'(\d+)[.](?:type|attr)\(([^)]*)\)',body)];require([list(f) for f in frames]==e['frames'] and [list(a) for a in attrs]==e['attributes'],'original timeline/attrs differ');fs=','.join('('+','.join([str(a),str(b),str(t),'&'+nums(s)])+')' for a,b,t,s in frames);ats=','.join('('+str(k)+',&'+nums(v)+')' for k,v in attrs);specs.append('Spec{sprite:'+str(e['graphics'])+',bank:'+str(e['bank'])+',header:&'+nums(e['header'])+',attrs:&['+ats+'],frames:&['+fs+']},\n')
 require(specs==proof['generated_specs'],'finite source differs from primary data')
 previous=read(candidate['previous_skill_qualification']['qualification']);old_coverage=previous['current_source_coverage'];require(old_coverage['recognized_count']==54 and old_coverage['missing_request_count']==126,'previous coverage differs');request=cp.parent/'LiTo381-Modern/src/modern_lito/native_skills.rs';text=request.read_text('utf-8-sig');body=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')];recognized=set()
 for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',body,re.M):recognized.update(map(int,re.findall(r'\d+',match[1])))
 ground=re.search(r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',text);item=re.search(r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',text);require(ground and item,'request parser differs');recognized.update(map(int,re.findall(r'\d+',ground[1])));recognized.update(map(int,re.findall(r'\d+',item[1])));missing=sorted(set(old_coverage['missing_request_ids'])-{23,36});require(recognized==set(old_coverage['recognized_ids'])|{23,36} and len(recognized)==56 and len(missing)==124,'request changes exceed #23/#36')
 coverage=dict(request_source=str(request),request_source_sha256=sha(request),recognized_ids=sorted(recognized),recognized_count=56,catalog_literal_ids=180,missing_request_ids=missing,missing_request_count=124,source_recognition_only=True,learned_book_or_runtime_verified=False)
 next_path=V/'native-E149-reveal23-tame36-source-review/20261024-073933-279174/next-catalog-selection.json';next_selection=read(next_path);require(next_selection['next_ids']==[27,40] and all(n in missing for n in next_selection['next_ids']) and next_selection['definitions']=={str(n):next(r['definitions'][0]['core'] for r in facts['definitions_by_id'] if r['skill_id']==n) for n in [27,40]} and all(r['skill_id']!=24 for r in facts['definitions_by_id']) and next_selection['ID24_absent_in_formal_literal_catalog'] and not next_selection['SQL_executed'] and next_selection['tests_executed']==0,'next catalog selection differs')
 identities=[c['pipeline'],*[r['original_process'] for r in c['compiler_runs']]];ids=','.join(str(r['pid']) for r in identities);query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress";raw=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in identities:require(not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live),'original compiler still live')
 require(len(read(c['base_compiled_inputs']))==1049,'normal input count differs');check_pins({str(R/n):h for n,h in read(c['base_compiled_inputs']).items()})
 for row in w['current_E149_integrated_client_development']['images'].values():require(sha(row['path'])==row['sha256'],'older GUI PE changed')
 before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint changed');tools=[T/n for n in ['prepare-native-E149-reveal23-tame36.py','compile-native-E149-reveal23-tame36.py','qualify-native-E149-reveal23-tame36.py']]
 for p in tools:ast.parse(p.read_text('utf-8-sig'),filename=str(p))
 out=owned(V/'native-E149-reveal23-tame36-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'original-terminal-CIM.json').write_bytes(raw);pins={str(p):sha(p) for p in [*tools,T/'templates/native_reveal23_tame36_tests.rs',T/'native-E149-reveal23-tame36.md',T/'native-E149-reveal23-tame36-followup.md',Path(r'D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-reveal23-tame36-source-review\20261024-073933-279174\next-catalog-selection.json'),proof_path,cp,Path(candidate['contract']),*[cp.parent/n for n in candidate['candidate_files']]]};pins.update(proof['source_pins'])
 record=dict(status='unselected_reveal23_tame36_native_screen_requests_and_finite_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate=str(cp),candidate_sha256=sha(cp),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*18,
  metadata_products={r['profile']+'-'+r['kind']:dict(path=r['product'],sha256=r['product_sha256'],bytes=r['product_bytes']) for r in c['compiler_runs']},resolved_module_path_attributes=362,changed_runtime_files=candidate['changed_runtime_files'],changed_files=4,effective_overlay_files=29,new_tests_written=10,new_opt_in_original_resource_cases=1,retained_skill_tests_written=119,
  contract=candidate['contract'],contract_sha256=sha(candidate['contract']),formal_evidence=str(proof_path),formal_evidence_sha256=sha(proof_path),current_source_coverage=coverage,wire_prefixes=proof['wire_prefixes'],definitions=proof['definitions'],native_screen_not_unbounded=True,NPC_domain_and_companion_state_server_owned=True,existing_2510_alias149_action0_no_audio_preserved=True,new_finite_graphics=[228,2166,2167,2168,2169],capacity_message_319_classified=True,original_nested_paths_preserved=6,module_predecessor_aliases=c['module_predecessor_aliases'],source_pins=pins,original_compiler_identities_terminal=True,
  current_fixture_binding_unchanged=w['current_E149_fixture_binding'],current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],previous_curse33_drain39_qualification=w['current_E149_curse33_drain39_development'],next_definition_ids=[27,40],next_catalog_selection=str(Path(r'D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-reveal23-tame36-source-review\20261024-073933-279174\next-catalog-selection.json')),next_catalog_selection_sha256=sha(Path(r'D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-reveal23-tame36-source-review\20261024-073933-279174\next-catalog-selection.json')),
  normal_sources_unchanged=True,normal_PEs_unchanged=True,old_source_success_transferred=False,tests_executed=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,new_sessions=0,GUI_verified=False,desktop_inputs=0,WAV_playbacks=0,whole_goal_complete=False,
  limitations=['metadata only; nine new pure and one original-resource case plus119 preceding written cases remain unrun','56 recognized request IDs is source coverage only; current E149 books/casts/eligibility/costs/resources/playback/GUI unqualified','caster action/effect not success; no client-created companion, weakness, resource spending or expiry','complete companion commands/owner/natural3600s and transfer retirement need current GAME/GUI qualification','G06 actual suite, original graphics base and G01-G12 remain pending','compiler-pinned future-task #24 label superseded by catalog-backed #27/#40; #24 absent, no implementation claimed'],recorded_utc=datetime.now(timezone.utc).isoformat())
 (out/'qualified-development.json').write_bytes(encoded(record));record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),previous_goal_turn_evidence=w['last_goal_turn_evidence']);archive={'qualified-development.json':out/'qualified-development.json','cohort.json':run/'cohort.json','candidate.json':cp,'contract.json':Path(candidate['contract']),'original-terminal-CIM.json':out/'original-terminal-CIM.json','previous-curse33-drain39-qualification.json':Path(candidate['previous_skill_qualification']['qualification'])};archive.update({'source-pins/'+Path(p).relative_to(R).as_posix():Path(p) for p in pins});archive.update({'candidate/'+n:cp.parent/n for n in candidate['candidate_files']})
 for row in c['compiler_runs']:
  for p in row['logs']:archive[row['profile']+'-'+row['kind']+'/'+Path(p).name]=Path(p)
 for p in [run/'lib.rs',run/'modern_lito.rs',*map(Path,c['paired_tests'].values())]:archive['compiled-source/'+p.name]=p
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n,p in archive.items():z.write(owned(p),n)
 with zipfile.ZipFile(out/'raw.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in archive.items()),'raw archive differs')
 record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'));(out/'evidence.json').write_bytes(encoded(record));work=('#23能量感測／#36迷魅術的正式reader與原畫面入口已接：C14 14 02 06只接actor ID共7bytes、14 04 03另接x/y共11bytes；兩者range=-1沿原生畫面，不當無限制。'
  'NPC領域／弱點bitmask／可迷魅性／HP與MP及40318×1／容量／原3600秒全由E149決定；保留remove怪物→送companion object→新ID的S37。caster19／effect不是成功，本機不造同伴／狀態／預扣數值；正式319容量拒絕補接。'
  '228原7frame／Sound132與2166..2169各11frame／Sound809..812嚴格定義已接；2510既有alias149 action0單frame／無聲保留，不選action3造duration，不按SPR英文註解改元素。'
  '四Runtime差分仍29檔十五Runtime／十一Rust QA／三Java，兩版library／library-test／七skill整合18metadata退出0，新10（9純＋1原資源opt-in）＋前119案全部未跑。請求來源識別56／剩124，非玩法驗收。'
  'S23／#33／#39及G06來源保留，正常1049輸入／所有PE、registry與G06 binding／16案640次不變。未套用／選入／連結或執行PE／Java／SQL／測試／GUI／播放，測試與桌面持續暫緩。下一群先核對#27壞物術／#40黑闇之影；完整companion指令／owner／自然到期、原gfx初始化與LT綁定及G11預算仍待。')
 staged={}
 for n in before:
  p=R/n
  if p.suffix=='.json':
   value=read(p);value['current_E149_reveal23_tame36_development']=record;value.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);value['current_focus']=dict(item='reveal23_tame36_native_screen_and_finite_resources',work=work,applies_to_normal_EXE=False,goal_complete=False)
   if n.endswith('next-validation-action.json'):value['next_action']=work
   staged[n]=encoded(value)
  else:
   _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');require(sep,'document format differs');staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in before:z.write(R/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 require(all(sha(R/n)==h for n,h in before.items()),'concurrent checkpoint writer');check_pins(c['normal_product_pins'])
 for n,blob in staged.items():(R/n).write_bytes(blob)
 require(all((R/n).read_bytes()==blob for n,blob in staged.items()),'checkpoint readback differs');(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(evidence=str(out/'evidence.json'),metadata_compilers=18,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--terminal-chunk',required=True);a=p.parse_args();publish(a.run,a.terminal_chunk)
