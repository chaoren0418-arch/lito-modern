from r491_common import *
proof=read(L/'r504-independent-contract.json');player=read(L/'r504-player-candidate.json');consumer=read(L/'r504-refusal-consumer-candidate.json')
save(L/'r504-open-gaps.json',dict(closed_risks=['No blanket nonzero ->retry; exactNativealert/ignore classification implemented','No sameepoch automaticcharacter retry afterknownalert; clearoldtransport/epoch/roster andrequirefresh','Actual identicaloldC9 offeredafternewsubmit cannotadvance freshsocket request','BackownerDrawreadsactualcaption'],remaining=[dict(id='Native_alert_UI',gap='Nativealerts call48F190/modal/496200/4962B0. Full modal dismiss/retry callbacks notqualified; Modernnormalclose/newEpoch isexplicitpolicy,notNativeequivalence.'),dict(id='no_request_ID',gap='Wirestillhasno requestID. Protection isstricttransport/epoch retirement plusfreshroster; revisiondoesnot solve sameepochC9 ambiguity. Unknownnonzero remainslockedwaiting untilactualreplyorClose.'),dict(id='page40_AFBC_B8',gap='R503 exactproducer gaps unchanged; page40 requiresAFBC/B8 return/reconnect lifecycle, notplainBSS success. No fixtureflags/setters.',next='Close realAFBC producer469BD0/46ED50/46EB30 andtheir source-ownedtimer/reconnect/address/selectedname/roster callback prerequisites; then distinctBack/sourcepackets inexistingIO.'),dict(id='GUI_auth_full40',gap='Player/nativeGUI0,realprovider/auth0; noOriginal/T73execution/realcredentials/serverchanges. Overall16/40.' )],GUI=0))
candidate=dict(consumer);candidate.update(state='R504_FIXED_CLASSIFIER_AND_FRESH_EPOCH_RETRY_CANDIDATE',validation='PASS9newriskflows/512sourceeffectstatuses/actualidentical15byte oldlateC9/3IO2endpoint3Closed;historicalPASSrerun0',production_source_manifest=pin(L/'r504-production-source-snapshot.json'),source_scope=pin(L/'r504-source-scope.json'),source_difference=pin(L/'r504-source-difference.patch'),player=player['executable'],player_candidate_record=pin(L/'r504-player-candidate.json'));save(L/'r504-candidate.json',candidate)
path=L/'r504-native-refusal-fresh-epoch-retry-handoff.json'
names=['r504-source-study.json','r504-source-oracle.json','r504-source-scope.json','r504-production-source-snapshot.json','r504-entry-source-contract.json','r504-player-candidate.json','r504-refusal-consumer-candidate.json','r504-refusal-consumer-exit.json','r504-event-details.json','r504-console-count-erratum.json','r504-execution-history.json','r504-independent-contract.json','r504-open-gaps.json','r504-candidate.json']
h=dict(state='R504_REFUSAL_CLASSIFIER_AND_AMBIGUOUS_RETRY_POLICY_FINITE_PASS_CHANNEL_OPEN',previous_R503_handoff=pin(L/'r503-native-navigation-subset-handoff.json'),records=[pin(L/n)for n in names],production_source_difference=pin(L/'r504-source-difference.patch'),player_entry_difference=pin(L/'r504-player-entry-difference.patch'),player=player['executable'],production_library=player['production_library'],consumer={k:consumer[k]for k in ['path','bytes','sha256']},tools=[pin(P/'tools'/n)for n in ['start-r504.py','edit-r504-runtime-window.py','prepare-r504-oracle.py','prepare-r504-candidate.py','r504-refusal-consumer-body.rs','build-r504-player-candidate.ps1','build-r504-refusal-consumer.ps1','verify-r504-segment.py','seal-r504-handoff.py']],proof=proof,Native_disconnect_on_alert=False,Modern_conservative_policy='KnownC8 7/8 orC9 1/10 ->normalcloseoldtransport/epoch/allroster/selected/controls;ReconnectRequired; explicitOpenLogin createsnewSocket/epoch;freshseed/auth/3/203 beforecharacters. Othersnonzero remainpending/ignore,noauto retry. This doesnotemulateNative modal handling.',caption='ExistingBack ownerdraw reads actual HWND text, so unavailablecaption isrendered fromsamecontrol. Compilation/static verification only,GUI0.',next='Only nowcontinueactualAFBC/B8producer/socket/selectedchar/roster lifecyclefromfixedoriginalsource;page40/channelroundtripstillOPEN;neverfixture-setflags orphase.',player_executions=0,native_executions=0,GUI=0,LOADED=False,complete_modes=16,total_modes=40,overall_completed=False)
save(path,h);hp=pin(path)
txt=P/'Native-R504_拒絕回包分類與新連線重試風險修正交接.txt'
txt.write_text(f'''R504 已封存拒絕回包分類與重試過期成功回包風險修正。9個新風險流程通過；沒有重跑R503的無關PASS，完整頻道往返與原版GUI仍未完成。整體16/40、GUI0。

來源與改變
固定原版4A76D0：C8 status0成功、7/8開提示，其他非零直接返回；4A7850：C9 status0成功、1/10開提示，其他非零含signednegative走default返回。R504精確分類512個status效果，修正R503全部非零都重新開人物選擇的policy差異。未知非零維持同一pendingrequest、selected／epoch／cipher與停用人物controls；不造可重試場景。
已知Native提示仍缺原版完整modal-dismiss/回呼恢復來源。Modern明確採保守處理：正常關閉旧Socket，退役舊epoch、RX/TX、roster、selected、人物/登入token，進ReconnectRequired，要求使用同OpenLogin命令建立新Socket／epoch，再實收seed、版本、auth後的新3/203名單，才可重新送選角。這不是原版會因7/8或1/10斷線的結論。typedChange RestartAfterNativeAlert使原stream batch正常停止，舊tail與late IO不能被新handle領走；同有界R502worker保持不變。
保護來自transport/epoch生命週期，不宣稱scene revision能關聯同opcode舊C9。没有requestID的wire仍無法在同epoch重試時證明C9屬新選角；因此不允許這條自動retry路徑。未知狀態可以继续等真成功或Close；成功分類／freshsocket回包仍可進Modern大廳。
Window重連狀態显示「重新連線」，只用既有control／同Runtime。Back ownerdraw先前硬編碼「返回」，現改讀draw.hwndItem實際按鈕文字，讓大廳的「返回頻道（尚未開放）」可被繪字；GUI未執行，只有同候選Win32編譯及靜態呼叫檢查。

新風險驗證
1：全部512 C8/C9狀態對照來源成功／提示／忽略分支。
2：真正Socket未知C8/2+C9/2不清selected、不換epoch、不開retrycontrols。
3：真正C9/1已知提示走明示Modern重連policy，server實讀EOF，舊IOworker完整Join，舊submit/control/Back及資料退役。
4：同OpenLogin建立新epoch/socket；尚無新名單及只收3/count時不能選角；完整新203才出現AgainA0..3新控制，舊控制仍失效。
5：原端點保留舊TCP handle；在新cs201送完且新request仍等待時，實際向舊socket寫完15bytes成功C9。四個新synthetic名各多2bytes，使freshRXcounter與舊兩個unknown(2+3bytes)+已知alert(3bytes)後counter完全相同；舊成功frame與新預期成功frame逐byte相同。持續poll後freshRX/TX未動、未誤進lobby；並非靠cipher錯誤或revision避免誤入。
6：只有新socket實收相同successwire才進lobby，selectedalias必須AgainA0；Close後兩IOworkers及主端點thread完整Join。
7：其餘C8/7、C8/8、C9/10透過同Runtime typeddecoded入口走重連policy；已保存舊epochC9對關閉／新空epoch皆拒絕，新空epoch不可選角。
8：ReconnectRequired時Close退役政策badge/owner並等resourceworker完整退休。
9：未知status仍pending時，實際activeSocket也可正常Close，IO／端點／resourceworker完整Join。
共9個完整TX frame（6個compat/auth前置、3個新導航），全部由獨立原PE4C6700來源指令oracle與端點actualwire對照；另1個延遲舊RX 15bytes確實TCP writecomplete，並有actual新success。3IOworkers、2endpointthreads完整Join，3RuntimeClosed。耗時{proof['elapsed_seconds']:.3f}秒，最大poll{proof['max_observed_Runtime_poll_us']}us僅本次headless觀察；測試不等於GUI。

封存與紀錄
618production sources；3修改navigation/Runtime/Window，615其他既有檔不改，無新增protocolcontroller；218固定consumer sourcepairs。diff{proof['actual_diff_bytes']}bytes，entrydiff{proof['entry_diff_bytes']}bytes。6既有固定ASM片段逐行對原件，原PE/ASM pins不變；未擴寫四個currentNativeprofile欄位。{proof['pin_instances']}pins/{proof['unique_paths']}unique核對。
1librarybuild／1playerlink／playerexec0含--version0，1consumerlink／1exec exit0；compile/test/preparation/verifierfailure0。consumer stdout literal誤寫8flows，結構化逐案證據實為9個且全passed；原source/exe/stdout未改、未重跑，獨立erratum與所有交接依實際9計數。沒有隱藏或丟棄失敗。
Modern自有目錄及允許ASCII暫存內作業，沒有Java/SQL/server/sharedBIN/provider、正式憑證、原版/T73/GUI、額外monitor。玩家候選／GUI0。舊R503固定source/candidate/失敗紀錄保留。

仍OPEN與下一步
原版完整alert modal UI/重試回呼不聲稱重現。AFBC/64AFB8仍缺實際producer與timer/reconnect/address/selectedname/新roster生命周期，沒有fixture設旗標或任意phase。此風險段閉合後下一輪才續釘這條有限來源；channelBack/lobbyReturn/roomBack、完整page40往返／正式認證／原版GUI／full40未驗收。

玩家：{player['executable']['path']}
玩家SHA256：{player['executable']['sha256']}
固定rlib SHA256：{player['production_library']['sha256']}
consumer SHA256：{consumer['sha256']}
handoff：{path}
handoff SHA256：{hp['sha256']}
''',encoding='utf-8')
progress('R504_source_alert_classification_new_epoch_retry9flows_finite_PASS_channel_OPEN_overall_unfinished','R504618source218pairs/3changed615unchanged/16674bytediff;sourceC8alerts7/8,C9alerts1/10,othersnonzeroignore;explicitModernknownalertnormalclose/newEpoch/new3/203policy(notNativeDisconnect).9newflows512statuses9TXframes(6setup3nav)/actual15byteidenticaloldC9lateafternewsubmitrefused/3IO2endpointJoin3Closed.1lib1playerlink0exec1consumer1exec/functionalFailure0/stdout8->structured9erratum/historicalPASSrerun0/Backactualcaptioncompileonly/GUI0/auth0/overall16of40.')
pp=P.parent/'監製進度.json';v=read(pp);v['r504']=dict(status='finite_refusal_retry_risk_segment_sealed',handoff=hp,text=pin(txt),player=player['executable'],production_library=player['production_library'],consumer=h['consumer'],source_files=618,source_pairs=218,changed=3,added=0,unchanged=615,source_fragments=6,actual_diff_bytes=proof['actual_diff_bytes'],player_entry_diff_bytes=proof['entry_diff_bytes'],flows=9,classified_statuses=512,actual_complete_socket_frames=9,new_navigation_frames=3,actual_delayed_old_TCP_bytes=15,late_wire_equals_new_success=True,IO_workers_joined=3,endpoint_threads_joined=2,Runtime_Closed=3,player_executions=0,consumer_executions=1,consumer_failures=0,preparation_failures=0,historical_PASS_reruns=0,Native_alert_disconnect=False,Modern_new_epoch_policy=True,full_channel_roundtrip=False,production_authentication=False,GUI=0,LOADED=False,complete_modes=16,total_modes=40,new_player_flow_review='PENDING',next=h['next']);v['completed'].append('R504：精確C8 7/8、C9 1/10提示與其他ignore；明示Modern舊連線退休／新epoch名單retry風險修正，9新流程512狀態／真15byte相同舊C9延遲跨重試不誤入／3IO2端點Join3Closed；GUI0、頻道producer仍OPEN。');save(pp,v);save(L/'r504-continuation.json',v['r504']);save(L/'r504-final-audit.json',dict(state=h['state'],handoff=hp,text=pin(txt),independent_contract=pin(L/'r504-independent-contract.json'),progress_location=str(pp),GUI=0,complete_modes=16,total_modes=40))
print('R504 sealed',hp['sha256'],'9newflows/512statuses/15identicaloldlatebytes/GUI0/channelOPEN')
