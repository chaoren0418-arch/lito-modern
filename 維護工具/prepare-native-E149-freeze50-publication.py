"""Derive source/metadata-only publication and independent readback tools."""
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具'
w=read(R/'工作佇列.json');q=read(w['current_E149_sun46_weak47_development']['qualification'])
def original(name):
 p=T/name;require(sha(p)==q['source_pins'][str(p)],'publication ancestor differs');return p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'publication anchor differs '+a[:100]);s=s.replace(a,b)
s=original('qualify-native-E149-sun46-weak47.py')
lo=s.index("NOTE='");hi=s.index('\ndef validate(',lo)
s=s[:lo]+'''NOTE='#50冰矛圍籬候選已補：C14 14 06 01＋角色ID／x/y、其他可見活體／六格、reuse500與client clock1000基值及實際選單；原756八圖依class4折四bank→219，兩者皆無音效／Hit，loader與clock明確空bank，不借用97或造Sound0。活動class4/5在raw ring淘汰後仍保留seen；重建資源準備失敗退役該flight並只記一次omitted。16秒冰凍及成本／機率／傷害皆server權威；756 arrival僅診斷，完整hurt相對gfx仍待。四Runtime差分仍33檔／365module／原六nested，兩版library／library-test／七skill整合18metadata退出0；新12（11純＋1原資源opt-in）＋前202共214案全未跑。來源入口63／缺117，非玩法或GUI資格。正常1049輸入／PE、正式JAR、registry與G06 binding／16案640次不變；桌面／測試／播放暫緩。'
NEXT='依當版#51／#55字面定義接剩餘入口；#51正式summonSpellRequest使用selection，須獨立召喚選項UI／Owner及controller，不能當actor ID或selector-only。續綁45／46／50原hurt完整guard／LT相對gfx與初始化、action0倍率／uint32 rollover，不猜base或造hurt。新silent756及class4/5 history失敗退役待恢復測試後驗原資源／重建／ring去重；完整G06、library Debug／Release、Release與GUI仍依總報告，等使用者明確恢復測試。'

''' +s[hi:]
replace("len(k['runtime_overrides'])==7 and case_count==17 and ignored_count==2", "len(k['runtime_overrides'])==4 and case_count==12 and ignored_count==1")
replace("previous['recognized_count']==60 and ids==set(previous['recognized_ids'])|{46,47} and len(ids)==62", "previous['recognized_count']==62 and ids==set(previous['recognized_ids'])|{50} and len(ids)==63")
replace('recognition changes exceed46/47','recognition changes exceed50')
replace('recognized_count=62,missing_request_ids=sorted(set(previous[\'missing_request_ids\'])-{46,47}),missing_request_count=118','recognized_count=63,missing_request_ids=sorted(set(previous[\'missing_request_ids\'])-{50}),missing_request_count=117')
replace("proof['sunburst_direct_wire_destination'] and proof['sunburst_hit_source_frame']==11 and proof['weakness_mix']=='Multiply_RGB555_attr104_128' and proof['javap_exit_code']==0", "proof['native_class']==4 and proof['S23_wire_type']==0 and proof['secondary_graphics']==219 and proof['flight_events']==proof['secondary_events']==[] and proof['server_freeze_authority_only']")
lo=s.index(" raw=Path(proof['raw_definitions']['2228']");hi=s.index(' libraries=',lo);s=s[:lo]+s[hi:]
replace("ref=w['current_E149_eruption45_entry_development']", "ref=w['current_E149_sun46_weak47_development']")
replace('changed_runtime_files=7','changed_runtime_files=4');replace('new_tests_written=17,new_opt_in_original_resource_cases=2,retained_skill_tests_written=185','new_tests_written=12,new_opt_in_original_resource_cases=1,retained_skill_tests_written=202')
lo=s.index('  class1_profiles=');hi=s.index('  current_fixture_binding_unchanged=',lo)
s=s[:lo]+'''  freeze50_silent_flight_and_secondary=True,optional_launch_audio_group=True,no_fallback_sound_or_Sound0=True,folded_bank_count=4,freeze_status_server_owned=True,flight_active_admission_retained=True,failed_flight_reprepare_retires_once=True,freeze50_hurt_dispatched=False,no_local_cost_damage_or_status_prediction=True,shared_audio_and_fragment_caps_unchanged=True,
'''+s[hi:]
replace('request_registered_ids=[46,47]','request_registered_ids=[50]')
replace('new_tests_written=17,tests_executed=0,request_recognition_count=62,missing_request_count=118','new_tests_written=12,tests_executed=0,request_recognition_count=63,missing_request_count=117')
replace("actual_progress='46/47 source, original class1 branch, metadata history and world-audio gate correction; eighteen metadata commands only'", "actual_progress='50 request/menu, silent756 class4 and active admission/failed reprepare retirement; eighteen metadata commands only'")
replace("item='sun46_weak47_and_class1_history_world_audio'", "item='freeze50_silent_class4_and_flight_history'")
replace('recognized_requests=62,remaining=118','recognized_requests=63,remaining=117')
s=s.replace('unselected_sun46_weak47_class1_history_and_world_audio_metadata_only','unselected_freeze50_silent_class4_and_history_metadata_only')
s=s.replace('current_E149_sun46_weak47_development\' not in w','current_E149_freeze50_development\' not in w')
s=s.replace("j['current_E149_sun46_weak47_development']", "j['current_E149_freeze50_development']")
s=s.replace('sun46-weak47','freeze50').replace('46/47 + class1','50 silent class4 and history')
out=T/'qualify-native-E149-freeze50.py';require(not out.exists(),'qualifier exists');out.write_text(s,encoding='utf-8',newline='\n')
s=original('readback-native-E149-sun46-weak47.py')
replace('(7,33,19,11,3,365,6,17,185,2)','(4,33,19,11,3,365,6,12,202,1)')
lo=s.index(" require(all(q[x] for x in ['class1_history_metadata_only'");hi=s.index(" require(sha(q['raw_archive'])",lo)
s=s[:lo]+''' require(all(q[x] for x in ['freeze50_silent_flight_and_secondary','optional_launch_audio_group','no_fallback_sound_or_Sound0','freeze_status_server_owned','flight_active_admission_retained','failed_flight_reprepare_retires_once','no_local_cost_damage_or_status_prediction','shared_audio_and_fragment_caps_unchanged']) and not any(q[x] for x in ['freeze50_hurt_dispatched','relative_graphics_hurt_binding_complete','optional_action0_speed_and_uint32_rollover_qualified','actual_GAME_overrides_observed','PEs_linked','outputs_executed','production_applied','canonical_registry_selected','GUI_verified','whole_goal_complete','old_source_success_transferred']) and q['tests_executed']==q['new_sessions']==q['desktop_inputs']==q['WAV_playbacks']==0,'execution/source scope differs')
 require(q['request_registered_ids']==[50] and q['folded_bank_count']==4 and cov['recognized_count']==63 and cov['missing_request_count']==117,'profile/coverage differs')
'''+s[hi:]
replace("['#46烈炎術／#47弱化術','202','62／剩118','class1','hurt','測試與桌面操作暫緩']", "['#50冰矛圍籬','214','63／剩117','無音效','hurt','測試與桌面操作暫緩']")
replace('new_cases_written=17','new_cases_written=12');replace('recognized_requests=62,remaining_request_ids=118,class1_history_source_integrated=True,sunburst_hurt_dispatched=False','recognized_requests=63,remaining_request_ids=117,silent_class4_source_integrated=True,freeze50_hurt_dispatched=False')
s=s.replace('unselected_sun46_weak47_class1_history_and_world_audio_metadata_only','unselected_freeze50_silent_class4_and_history_metadata_only').replace('current_E149_sun46_weak47_development','current_E149_freeze50_development').replace('sun46-weak47','freeze50')
out=T/'readback-native-E149-freeze50.py';require(not out.exists(),'reader exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
