"""Derive source/compiler publication with independent nested wrapper reconstruction."""
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';w=read(R/'工作佇列.json');q=read(w['current_E149_freeze50_development']['qualification'])
def original(name):
 p=T/name;require(sha(p)==q['source_pins'][str(p)],'ancestor differs');return p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'publication anchor differs '+a[:90]);s=s.replace(a,b)
s=original('qualify-native-E149-freeze50.py');lo=s.index("NOTE='");hi=s.index('\ndef validate(',lo)
s=s[:lo]+'''NOTE='#51召喚術候選已接獨立種類下拉選單及typed SummonTarget／controller：C14 14 06 02＋selection共七bytes，不用actor ID。正式31種選項經兩個helper與七欄constructor展開，特殊容量含+7；controlAbility5時依選項，否則由server依等級決定。顯式選取與背包revision／完整owner、connection／generation／actor／control interruption／map守衛保留；reuse1000基值、30MP／HP、三寶石、容量與1800秒皆server權威。353／743召喚拒絕可晚於action，新增scoped pending回饋；2510／755非成功ACK。四Runtime差分仍33檔／365module，原五controller＋一rental共六nested重建同target；兩版library／library-test／七skill整合18metadata退出0。新10純QA＋前214共224案全未跑；入口64／缺116，不是玩法／GUI資格。正常1049輸入／PE、正式JAR、registry與G06 binding／16案640次不變，桌面／測試／播放暫緩。'
NEXT='依當版#55／#56定義續接剩餘玩法；#51完整companion指令、server控制能力／模板／材料／拒絕、自然1800秒與owner／傳送退役須恢復測試後驗。續綁45／46／50原hurt完整guard／LT相對gfx與初始化、action0倍率／uint32 rollover，不猜base或造hurt。完整G06、library Debug／Release、Release與GUI仍依總報告，等使用者明確恢復測試。'

'''+s[hi:]
replace("s+='\\n'+extra;case_count+=extra.count('#[test]');ignored_count+=extra.count('#[ignore=')", "s+='\\n'+extra;case_count+=extra.count('#[test]');ignored_count+=extra.count('#[ignore=')\n  if row['generated_suffix']:s+='\\n'+row['generated_suffix']")
lo=s.index("  needles=list(dict.fromkeys([row['predecessor']");hi=s.index(" require(len(k['runtime_overrides'])",lo)
s=s[:lo]+'''  previous=row['predecessor'];replacement=row['path']
  if name in old['original_nested_runtime_pairs']:
   pair=old['original_nested_runtime_pairs'][name];current=c['original_nested_runtime_pairs'][name];require(current['source']==row['path'] and current['source_sha256']==row['sha256'] and current['rewrites']==pair['rewrites'] and current['predecessor']==pair,'nested pair identity differs');wrapped=s
   for rewrite in pair['rewrites']:
    require(wrapped.count(rewrite['original_attribute'])==1 and sha(rewrite['resolved_path'])==c['input_pins'][rewrite['resolved_path']],'nested target differs');wrapped=wrapped.replace(rewrite['original_attribute'],rewrite['compiled_attribute'])
   require(wrapped==Path(current['path']).read_text('utf-8-sig') and sha(current['path'])==current['sha256'],'nested wrapper differs');previous=pair['path'];replacement=current['path']
  needles=list(dict.fromkeys([previous,Path(previous).as_posix()]));found=[n for n in needles if modern.count(n)==1];require(len(found)==1 and sum(modern.count(n) for n in needles)==1,'runtime mapping differs');modern=modern.replace(found[0],Path(replacement).as_posix())
'''+s[hi:]
replace('case_count==12 and ignored_count==1','case_count==10 and ignored_count==0')
replace("c['original_nested_runtime_pairs']==old['original_nested_runtime_pairs']", "set(c['original_nested_runtime_pairs'])==set(old['original_nested_runtime_pairs'])")
needle=" previous=prior['current_source_coverage'];";require(s.count(needle)==1,'coverage marker differs');s=s.replace(needle," m=re.search(r'pub fn summon_targeted\\(skill:u16\\)->bool \\{matches!\\(skill,([0-9|]+)\\)\\}',s);require(m is not None,'summon source recognition absent');ids.update(map(int,re.findall(r'\\d+',m[1])))\n"+needle)
replace("previous['recognized_count']==62 and ids==set(previous['recognized_ids'])|{50} and len(ids)==63", "previous['recognized_count']==63 and ids==set(previous['recognized_ids'])|{51} and len(ids)==64")
replace('recognition changes exceed50','recognition changes exceed51')
replace("recognized_count=63,missing_request_ids=sorted(set(previous['missing_request_ids'])-{50}),missing_request_count=117", "recognized_count=64,missing_request_ids=sorted(set(previous['missing_request_ids'])-{51}),missing_request_count=116")
replace("proof['native_class']==4 and proof['S23_wire_type']==0 and proof['secondary_graphics']==219 and proof['flight_events']==proof['secondary_events']==[] and proof['server_freeze_authority_only']", "len(proof['choices'])==31 and proof['control_ability']==5 and proof['special_capacity_adjustment']==7 and proof['material_before_action'] and proof['action_before_map_level_capacity_failure']")
needle=" libraries={p:next";require(s.count(needle)==1,'library marker differs');s=s.replace(needle," expected='pub const SUMMON_SELECTIONS:&[SummonSelection]=&[\\n'+''.join('    SummonSelection{selection:'+str(row['selection'])+',label:'+json.dumps(row['base_label'],ensure_ascii=False)+',required_level:'+str(row['required_level'])+'},\\n' for row in proof['choices'])+'];\\n';require(expected==k['generated_summon_selections'] and k['runtime_overrides']['LiTo381-Modern/src/modern_lito/native_skills.rs']['generated_suffix']==expected,'generated choices differ')\n"+needle)
replace("ref=w['current_E149_sun46_weak47_development']", "ref=w['current_E149_freeze50_development']")
replace("'current_E149_freeze50_development' not in w", "'current_E149_summon51_development' not in w")
replace('new_tests_written=12,new_opt_in_original_resource_cases=1,retained_skill_tests_written=202','new_tests_written=10,new_opt_in_original_resource_cases=0,retained_skill_tests_written=214')
lo=s.index('  freeze50_silent_flight_and_secondary=');hi=s.index('  current_fixture_binding_unchanged=',lo)
s=s[:lo]+'''  summon_selection_count=31,typed_summon_domain=True,explicit_nonmodal_selection_UI=True,full_inventory_owner_and_InputStamp_fence=True,server_controlled_selection_or_direct_grade=True,scoped_map_and_level_failure_after_cast_action=True,summon_effects_are_not_ACK=True,original_six_nested_targets_preserved=True,no_local_cost_damage_or_status_prediction=True,shared_audio_and_fragment_caps_unchanged=True,companion_commands_and_natural_lifetime_qualified=False,
'''+s[hi:]
replace('request_registered_ids=[50]','request_registered_ids=[51]')
replace('new_tests_written=12,tests_executed=0,request_recognition_count=63,missing_request_count=117','new_tests_written=10,tests_executed=0,request_recognition_count=64,missing_request_count=116')
replace("actual_progress='50 request/menu, silent756 class4 and active admission/failed reprepare retirement; eighteen metadata commands only'", "actual_progress='51 typed selection UI and controller, original31 helpers, scoped late refusal; eighteen metadata commands only'")
replace("item='freeze50_silent_class4_and_flight_history'", "item='summon51_typed_selection_UI_and_controller'")
replace('recognized_requests=63,remaining=117','recognized_requests=64,remaining=116')
replace("j['current_E149_freeze50_development']", "j['current_E149_summon51_development']")
s=s.replace('unselected_freeze50_silent_class4_and_history_metadata_only','unselected_summon51_selection_UI_controller_metadata_only').replace('freeze50','summon51').replace('50 silent class4 and history','51 selection UI and controller')
# Global stage renaming must not rename the real predecessor queue pointer.
s=s.replace("ref=w['current_E149_summon51_development'];require(ref['qualification']==k['predecessor_qualification']", "ref=w['current_E149_freeze50_development'];require(ref['qualification']==k['predecessor_qualification']")
out=T/'qualify-native-E149-summon51.py';require(not out.exists(),'qualifier exists');out.write_text(s,encoding='utf-8',newline='\n')
s=original('readback-native-E149-freeze50.py')
replace('(4,33,19,11,3,365,6,12,202,1)','(4,33,19,11,3,365,6,10,214,0)')
lo=s.index(" require(all(q[x] for x in ['freeze50_silent_flight_and_secondary'");hi=s.index(" require(sha(q['raw_archive'])",lo)
s=s[:lo]+''' require(all(q[x] for x in ['typed_summon_domain','explicit_nonmodal_selection_UI','full_inventory_owner_and_InputStamp_fence','server_controlled_selection_or_direct_grade','scoped_map_and_level_failure_after_cast_action','summon_effects_are_not_ACK','original_six_nested_targets_preserved','no_local_cost_damage_or_status_prediction','shared_audio_and_fragment_caps_unchanged']) and not any(q[x] for x in ['companion_commands_and_natural_lifetime_qualified','relative_graphics_hurt_binding_complete','optional_action0_speed_and_uint32_rollover_qualified','actual_GAME_overrides_observed','PEs_linked','outputs_executed','production_applied','canonical_registry_selected','GUI_verified','whole_goal_complete','old_source_success_transferred']) and q['tests_executed']==q['new_sessions']==q['desktop_inputs']==q['WAV_playbacks']==0,'execution/source scope differs')
 require(q['request_registered_ids']==[51] and q['summon_selection_count']==31 and cov['recognized_count']==64 and cov['missing_request_count']==116,'profile/coverage differs')
'''+s[hi:]
replace("['#50冰矛圍籬','214','63／剩117','無音效','hurt','測試與桌面操作暫緩']", "['#51召喚術','224','64／剩116','selection','hurt','測試與桌面操作暫緩']")
replace('new_cases_written=12','new_cases_written=10')
replace('recognized_requests=63,remaining_request_ids=117,silent_class4_source_integrated=True,freeze50_hurt_dispatched=False','recognized_requests=64,remaining_request_ids=116,summon51_selection_UI_source_integrated=True,companion_lifetime_qualified=False')
s=s.replace('unselected_freeze50_silent_class4_and_history_metadata_only','unselected_summon51_selection_UI_controller_metadata_only').replace('current_E149_freeze50_development','current_E149_summon51_development').replace('freeze50','summon51')
out=T/'readback-native-E149-summon51.py';require(not out.exists(),'reader exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
