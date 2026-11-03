"""Derive source/metadata publisher and independent original-process readback."""
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';w=read(R/'工作佇列.json');q=read(w['current_E149_summon51_development']['qualification'])
def original(name):
 p=T/name;authority=q
 if str(p) not in authority['source_pins']:
  require(sha(q['predecessor_qualification'])==q['predecessor_qualification_sha256'],'historical publisher qualification differs');authority=read(q['predecessor_qualification'])
 require(sha(p)==authority['source_pins'][str(p)],'publication ancestor differs');return p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'publication anchor differs '+a[:90]);s=s.replace(a,b)
s=original('qualify-native-E149-freeze50.py');lo=s.index("NOTE='");hi=s.index('\ndef validate(',lo)
s=s[:lo]+'''NOTE='#55狂暴術／#56疾病術候選已補：55七byte角色ID、自用／對人與原畫面，正式同血盟／隊伍及320秒／MP40／AC+10等由server判定；56十一byte角色ID／x/y、其他活體／三格／reuse500，64秒／一寶石／機率／近戰-6與AC+12皆server權威。3943原八方向27frame／Sound1361在source2，S37原PE接收時複製actor heading，World與Active保存接續，八bank不跟隨轉身或離場；2230原十一frame／Sound839在source1與attr128 Multiply。兩新效果typed audio用world actor0域保留真cast／Scope；finite seen保留active防ring回入重啟。六Runtime差分34檔（20 Runtime／11 Rust QA／3 Java）、365module／原六nested，兩版library／library-test／七skill整合18metadata退出0。新14（13純含合成像素＋1原資源opt-in）＋前226共240案全未跑；入口66／缺114，不是玩法／GUI資格。正常1049輸入／PE、正式JAR、registry與G06 binding／16案640次不變；桌面／測試／播放暫緩。'
NEXT='依當版#59／#61字面定義續接剩餘玩法；55／56原資源、八方向／Multiply、伺服器資格／狀態／材料／自然期限與History退役待恢復測試後驗。續綁45／46／50 hurt原guard／LT相對gfx與初始化、倍率／uint32 rollover，不猜base或造hurt；完整companion指令、G06、library Debug／Release、Release與GUI仍依總報告，等使用者明確恢復測試。'

'''+s[hi:]
replace('len(k[\'runtime_overrides\'])==4 and case_count==12 and ignored_count==1','len(k[\'runtime_overrides\'])==6 and case_count==14 and ignored_count==1')
replace("len(files)==33 and files==k['effective_overlay_files']", "len(files)==34 and files==k['effective_overlay_files']")
needle=" previous=prior['current_source_coverage'];";require(s.count(needle)==1,'coverage marker differs');s=s.replace(needle," m=re.search(r'pub fn summon_targeted\\(skill:u16\\)->bool \\{matches!\\(skill,([0-9|]+)\\)\\}',s);require(m is not None,'summon source domain absent');ids.update(map(int,re.findall(r'\\d+',m[1])))\n"+needle)
replace("previous['recognized_count']==62 and ids==set(previous['recognized_ids'])|{50} and len(ids)==63", "previous['recognized_count']==64 and ids==set(previous['recognized_ids'])|{55,56} and len(ids)==66")
replace('recognition changes exceed50','recognition changes exceed55/56')
replace("recognized_count=63,missing_request_ids=sorted(set(previous['missing_request_ids'])-{50}),missing_request_count=117", "recognized_count=66,missing_request_ids=sorted(set(previous['missing_request_ids'])-{55,56}),missing_request_count=114")
replace("proof['native_class']==4 and proof['S23_wire_type']==0 and proof['secondary_graphics']==219 and proof['flight_events']==proof['secondary_events']==[] and proof['server_freeze_authority_only']", "proof['buff_object_request_and_server_clan_or_party'] and proof['disease_actor_xy_material_world_and_battle'] and proof['S37_effect_heading_copies_receipt_actor_heading'] and proof['berserk_sound_source_frame']==2 and proof['disease_Multiply_attr128']")
needle=' libraries={p:next';require(s.count(needle)==1,'library marker differs');s=s.replace(needle," nums=lambda a:'['+','.join(map(str,a))+']';expected=''\n for gfx,attrs in [(3943,[(102,b'0'),(104,b'8'),(109,b'0 -1')]),(2230,[(102,b'0'),(104,b'128')])]:\n  raw=Path(proof['raw_definitions'][str(gfx)]['path']).read_bytes().rstrip(b'\\r\\n');frames=[(int(a),int(f),int(t),[] if not sound else [int(sound)]) for a,f,t,sound in re.findall(rb'(\\d+)[.](\\d+):(\\d+)(?:\\[(\\d+))?',raw)];expected+='Spec{sprite:'+str(gfx)+',bank:'+str(gfx)+',header:&'+nums(raw.splitlines()[0])+',attrs:&['+','.join('('+str(key)+',&'+nums(body)+')' for key,body in attrs)+'],frames:&['+','.join('('+','.join([str(a),str(f),str(t),'&'+nums(sound)])+')' for a,f,t,sound in frames)+']},\\n'\n require(expected==k['generated_specs'] and sha(k['original_inspector_failure'])==k['original_inspector_failure_sha256'],'generated source/original inspection failure differs')\n"+needle)
replace("ref=w['current_E149_sun46_weak47_development']", "ref=w['current_E149_summon51_development']")
replace("'current_E149_freeze50_development' not in w", "'current_E149_berserk55_disease56_development' not in w")
replace('changed_runtime_files=4,effective_overlay_files=33,runtime_modules=19','changed_runtime_files=6,effective_overlay_files=34,runtime_modules=20')
replace('new_tests_written=12,new_opt_in_original_resource_cases=1,retained_skill_tests_written=202','new_tests_written=14,new_opt_in_original_resource_cases=1,retained_skill_tests_written=226')
lo=s.index('  freeze50_silent_flight_and_secondary=');hi=s.index('  current_fixture_binding_unchanged=',lo)
s=s[:lo]+'''  berserk_heading_snapshot_and_eight_bank=True,disease_attr128_Multiply=True,server_clan_party_and_status_authority=True,finite_active_admission_retained=True,new_effects_world_audio_domain=True,finite_metadata_history_preserves_heading=True,existing_effect_struct_API_unchanged=True,no_local_cost_damage_or_status_prediction=True,shared_audio_and_fragment_caps_unchanged=True,
'''+s[hi:]
replace('request_registered_ids=[50]','request_registered_ids=[55,56]')
replace('effective_overlay_files=33,new_tests_written=12,tests_executed=0,request_recognition_count=63,missing_request_count=117','effective_overlay_files=34,new_tests_written=14,tests_executed=0,request_recognition_count=66,missing_request_count=114')
replace("actual_progress='50 request/menu, silent756 class4 and active admission/failed reprepare retirement; eighteen metadata commands only'", "actual_progress='55/56 request/menu, original S37 frozen heading/eight banks, Multiply and finite active admission/world audio; eighteen metadata commands only'")
replace("item='freeze50_silent_class4_and_flight_history'", "item='berserk55_disease56_receipt_heading_and_Multiply'")
replace('recognized_requests=63,remaining=117','recognized_requests=66,remaining=114')
replace("j['current_E149_freeze50_development']", "j['current_E149_berserk55_disease56_development']")
needle=" with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:";require(s.count(needle)==1,'raw marker differs');s=s.replace(needle," for path in [Path(k['original_inspector_failure']),Path(k['original_inspector_failure']).with_name('failed-inspector.py')]:raw[str(path)]=sha(path)\n"+needle)
s=s.replace('unselected_freeze50_silent_class4_and_history_metadata_only','unselected_berserk55_disease56_frozen_heading_Multiply_metadata_only').replace('freeze50','berserk55-disease56').replace('50 silent class4 and history','55/56 receipt heading and Multiply')
out=T/'qualify-native-E149-berserk55-disease56.py';require(not out.exists(),'qualifier exists');out.write_text(s,encoding='utf-8',newline='\n')
s=original('readback-native-E149-freeze50.py')
replace('(4,33,19,11,3,365,6,12,202,1)','(6,34,20,11,3,365,6,14,226,1)')
lo=s.index(" require(all(q[x] for x in ['freeze50_silent_flight_and_secondary'");hi=s.index(" require(sha(q['raw_archive'])",lo)
s=s[:lo]+''' require(all(q[x] for x in ['berserk_heading_snapshot_and_eight_bank','disease_attr128_Multiply','server_clan_party_and_status_authority','finite_active_admission_retained','new_effects_world_audio_domain','finite_metadata_history_preserves_heading','existing_effect_struct_API_unchanged','no_local_cost_damage_or_status_prediction','shared_audio_and_fragment_caps_unchanged']) and not any(q[x] for x in ['relative_graphics_hurt_binding_complete','optional_action0_speed_and_uint32_rollover_qualified','actual_GAME_overrides_observed','PEs_linked','outputs_executed','production_applied','canonical_registry_selected','GUI_verified','whole_goal_complete','old_source_success_transferred']) and q['tests_executed']==q['new_sessions']==q['desktop_inputs']==q['WAV_playbacks']==0,'execution/source scope differs')
 require(q['request_registered_ids']==[55,56] and cov['recognized_count']==66 and cov['missing_request_count']==114,'profile/coverage differs')
'''+s[hi:]
replace("['#50冰矛圍籬','214','63／剩117','無音效','hurt','測試與桌面操作暫緩']", "['#55狂暴術／#56疾病術','240','66／剩114','heading','Multiply','測試與桌面操作暫緩']")
replace('new_cases_written=12','new_cases_written=14')
replace('recognized_requests=63,remaining_request_ids=117,silent_class4_source_integrated=True,freeze50_hurt_dispatched=False','recognized_requests=66,remaining_request_ids=114,receipt_heading_and_Multiply_source_integrated=True')
s=s.replace('unselected_freeze50_silent_class4_and_history_metadata_only','unselected_berserk55_disease56_frozen_heading_Multiply_metadata_only').replace('current_E149_freeze50_development','current_E149_berserk55_disease56_development').replace('freeze50','berserk55-disease56')
out=T/'readback-native-E149-berserk55-disease56.py';require(not out.exists(),'reader exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
