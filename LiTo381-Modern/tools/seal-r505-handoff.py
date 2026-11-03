from r491_common import *
proof=read(L/'r505-independent-contract.json');player=read(L/'r505-player-candidate.json');consumer=read(L/'r505-transfer-consumer-candidate.json');gaps=read(L/'r505-open-gaps.json')
records=['r505-source-study.json','r505-global-references.json','r505-open-gaps.json','r505-unpacker-minimum-dependency.json','r505-source-oracle.json','r505-production-source-snapshot.json','r505-entry-source-contract.json','r505-player-candidate.json','r505-transfer-consumer-candidate.json','r505-transfer-consumer-exit.json','r505-event-details.json','r505-count-erratum.json','r505-execution-history.json','r505-independent-contract.json']
path=L/'r505-native-transfer-prefix-address-gates-handoff.json'
h=dict(state='R505_FINITE_SOURCE_TRANSFER_PREFIX_PASS_FULL_CHANNEL_PATH_NOT_ACHIEVED',previous_R504_handoff=pin(L/'r504-native-refusal-fresh-epoch-retry-handoff.json'),records=[pin(L/n)for n in records],source_difference=pin(L/'r505-source-difference.patch'),entry_difference=pin(L/'r505-player-entry-difference.patch'),tools=[pin(p)for p in sorted((P/'tools').glob('*r505*'))],production_player=player['executable'],production_library=player['production_library'],consumer={k:consumer[k]for k in ('path','bytes','sha256')},contract=proof,source_gaps=gaps['source_gaps'],unpacker=dict(chat='01a10c34-86a9-7d61-af9b-36fb29c55378',output_root='D:/lineage開發/3-lito開發/lito脫殼資源',minimum_dependency=pin(L/'r505-unpacker-minimum-dependency.json')),full_channel_roundtrip=False,minimum_player_path_achieved=False,production_authentication=False,GUI=0,LOADED=False,overall_complete=False,complete_modes=16,total_modes=40,next='Consume independentlyverifiedoriginal externaltarget/module/API conversion and64AD74 initializer fromunpacker;closeUIrow/startupcaller;thenactualsourceproducer->page40channel callback/return/reconnect/nameReplay/newroster lifecycle usingexistingRuntime/EntryCommand/IO/Sender,onlynewrisks,no oldPASSrerun,no flags/phase injection.')
save(path,h);hp=pin(path);txt=L/'Native-R505_頻道來源前段與地址缺口交接.txt'
txt.write_text(f'''R505 頻道來源前段與地址缺口交接
本輪只完成來源可確認的封包前段與生命週期風險段；最低「人物→page40→頻道→大廳」或「lobbyReturn→channel→reenter」玩家路徑未達成，完整頻道往返仍 OPEN。不是將原版GUI失敗複製為Modern規格。GUI0／正式認證0／overall16/40。

已實作
新增 native_entry_transfer，使用同 Runtime／Epoch／既有有界 Receiver／IO／Sender；未加協定控制器、旗標setter、任意phase、profile地址權威。243=原版4AC520 signed c count後count個d，更新64AD2C十格表前段、保留未更新尾段，初值零。244=4AC570格式502F2C的ASCII cdh：signed c索引、d地址值、h埠號（不是cdd）。來源tableFFFF guard直接返回。Modern增加十格容量／索引／精確長度防護，避免越界寫到64AD54選中人物名稱。
有效244跨過sentinel後，需要外部地址轉換結果才能選46EB30或46ED50；目前明示Modern未支援來源政策：正常retire舊socket/epoch/roster/selected/table/RX/TX/token，顯示地址來源未確認及重新連線；不是聲稱原版遇244必斷線。stream terminal丟棄舊coalesced C9 tail；sameOpenLogin建立新socket/seed/auth/3+203，不能復活舊table或人物token。Close同既有回收。

來源必要條件（供登入器對照GUI症狀；不是診斷結論）
46EB30，ASM236286..236345：cD9，AFBC1/B8=1，cCB，timer469B10，ccCA0，469CA0(1)，AFC8=1。前次口頭讀成469CA0(0)不正確，本轮原件是1。
46ED50，ASM236486..236569：cD9，AFBC1/AFBE0，4EFE4A轉incoming d地址→字串64AE74，h埠號64AD70，c15，dispatch3，存primary64A9C4/64A9C0，AFC7=1/AFC8=0，timer469B10，close4C65F0/connect4C62A0。IDA noreturn標記截斷，PE和續段確認實際有後續return。
469AD0/469AF0：unsigned now嚴格大於wrapping(start+3000)且AFC8非零→4699F0恢復回呼/清AFC7；否則嚴格大於wrapping(start+10000)→交替重連。不可直接改成elapsed>=。469B10 timeout：AFBD1/AFBC0、fallback64AD74、port2000、469BD0；469BD0 timeout：AFBD0/AFBC1、savedprimary64A9C4/C0、469B10。
4A76D0 C8success：AFBC优先→B8=2、清AFBC/BE、mode0、phase2、497020普通分支page40；AFBD→B8zero、清AFBD/BE、ccCA100；普通B8zero也只ccCA100。4A7850 C9success：AFBC→B8=1 clearBC，接著AFBD→B8zero clearBD，46BA10phase3普通tail mode1。
4928C0來源raw roster alias複製到64AD54（28bytes）。4A7550遇AFBC或AFBD，203在解析人物record之前走alternate branch：AFBE0只發一次cs201保存的64AD54，然後BE1；不能當成新ordinary roster或重新取新selectedalias。
475F40來源UI，ASM253003..253275：message202/命中範圍、object+30有效index、object+1C vector有效行（stride0x118）、row+110!=FFFFFFFF、row+114 capacity與混淆8316B0/B4/B8 guard，externalprimary conversion與row+104比較；same→46EB30，different→46ED50(row+104,row+108)。476370是invalid vector<T> subscript/std::out_of_range例外路徑，不能解讀為補建資料列。
另00474FC0匿名原版區塊，ASM251010..251091：依64B0A4/64B0A8=3或5切AFBC/BD，選64B0B0/C0地址對再close/connect；尚無可達caller證明，不加Modern invoker。
channelBack=46B1A0 mode0→495280destroy→46F010；lobbyReturn=455330直接46F010。B8=0無協定；B8=1為cCB+ccCA100/AFBD1/B8zero/timer469BD0；B8=2為c15/savedprimary/fallbackport2000/dispatch3/AFBD1/B8zero/BE0/close-connect。roomLeave mode1的modal648F20/cD0、mode2/3 cD0獨立，不能把c15/cCB/cCA/cD0混用。46F010沒有直接建page40，後續C8需要正確AFBC/BD條件。
使用者這次原版GUI「返回頻道後再進大廳失敗、頻道返回無效」只列為失敗證據；首次大廳技能空白是另一路未調查，沒有修復或oracle聲稱。

脫殼組可直接消費的缺口（chat01a10c34-86a9-7d61-af9b-36fb29c55378）
004AC59B CALL773F5200（E8608CF476），格式來源62C6A4固定ASCII127.0.0.1、28byte stack destination；外部本體/API身份未確認。
004AC5A7 CALL004EFE56；PE004EFE56開頭E985594075→JMP758F57E0。ASM501120 NOP與PE不一致，採PE精確bytes。004AC5BD比較返回EAX與incoming LE decoded d，004AC5C1/5CD才選46EB30/46ED50。
0046EE93 CALL004EFE4A；wrapper90E9F05A4075→758F5940，返回EAX字串指標供複製64AE74。另475F40+1D8=00476118→004EFE5C→7591FF10為fallbackhostname相關結構讀取，需原件身份，不猜API。
64AD74（256bytes）固定PE全零、ASM55968440hdup(0)；named references只讀，没有合格初始化writer。不能推論絕對無indirectwriter，也不能拿FixtureProfile.endpoint/currentTransport.peer填作原版地址。
最小輸出：帶original/module/SHA pins的外部target→module/export/VA/RVA對照、可靠呼叫規約/函式本體或conversion定義（127.0.0.1 exact返回32bit/endian/error值）；以及64AD74可達初始化store/前驅呼叫链/byte來源/NUL/連線生命週期。只提供建議函式名不足以接線。
最小依賴：{L/'r505-unpacker-minimum-dependency.json'}
該檔SHA：{pin(L/'r505-unpacker-minimum-dependency.json')['sha256']}

新驗證與封存
6流程，512signed count/index邊界與7malformed parser拒絕；真正TCP局部243未完成不改表/計數；短表保留尾格、負count零讀、unavailable244無TX；有效244明示缺口並清舊owned狀態/coalesced C9；fresh actualsocket/roster不能復活舊control/table；真正錯cdh長度退役；正常Close。
7實際完整TX frame=6compat/auth前置+1選角，導航TX0。原structured setup_TX_frames誤填7，stdout及complete_TX7正確；保留原source/exe/stdout/eventdetails，獨立勘誤，不重跑。3IOworkers/3endpointthreads Join，2Runtime Closed，{proof['elapsed_seconds']:.3f}秒，觀察最大poll{proof['max_observed_Runtime_poll_us']}us（非GUI保證）。
1librarybuild／1playerlink／playerexec0含--version0／1consumerlink／1consumerexec exit0；functionalFailure0、metadata count勘誤1、historical PASS rerun0。
619production sources，2修改Runtime/Scope、1新增transfer、616其他檔不變；221固定consumer sourcepairs。productiondiff{proof['actual_diff_bytes']}bytes，entrydiff{proof['entry_diff_bytes']}bytes。{proof['ASM_fragments_verified']}ASM/{proof['PE_fragments_verified']}PE逐行／byte／Capstone重新核對，{proof['pin_instances']}pins/{proof['unique_paths']}unique。source snapshot、固定player/rlib/consumer/所有來源與原證據保留。
寫入Modern自有目錄／允許ASCII暫存／監製進度。無server/Java/SQL/sharedBIN/provider/實帳密/原版或T73/GUI執行，也沒有monitor。

玩家：{player['executable']['path']}
玩家SHA：{player['executable']['sha256']}
固定rlib SHA：{player['production_library']['sha256']}
consumer SHA：{consumer['sha256']}
handoff：{path}
handoff SHA：{hp['sha256']}
下一步必須消費脫殼組有來源的地址結果與initializer，再接可達page40/channelBack/lobbyReturn/reconnect/nameReplay/newroster；不得宣稱本輪頻道往返完成。
''',encoding='utf-8')
progress('R505_transfer_prefix6flows_finite_PASS_external_address_initializer_OPEN_overall_unfinished','R505619sources221pairs/2changed1new616unchanged/6flows512signedboundaries7actualTX(6setup1select0nav)/3IO3endpointJoin2Closed;28ASM15PE/2388pins2213unique. F3/F4cdh/sentinelprefixclosed;fullchannel/minimumpath NOTachieved. Exactexternal773F5200/4EFE56->758F57E0/4EFE4A->758F5940 and64AD74initializer gaps recordedforunpackerchat01a10c34-86a9-7d61-af9b-36fb29c55378;no flags/profileaddressguess;GUI0/auth0/overall16of40.')
pp=P.parent/'監製進度.json';v=read(pp);v['r505']=dict(status=h['state'],handoff=hp,text=pin(txt),independent_contract=pin(L/'r505-independent-contract.json'),unpacker=h['unpacker'],source_files=619,source_pairs=221,changed=2,added=1,unchanged=616,source_difference=h['source_difference'],entry_difference=h['entry_difference'],flows=6,signed_boundaries=512,actual_TX_frames=7,setup_TX_frames=6,character_TX_frames=1,navigation_TX_frames=0,IO_workers_joined=3,endpoint_threads_joined=3,Runtime_Closed=2,player_executions=0,consumer_executions=1,functional_failures=0,metadata_count_errata=1,historical_PASS_reruns=0,minimum_player_path_achieved=False,full_channel_roundtrip=False,production_authentication=False,GUI=0,LOADED=False,complete_modes=16,total_modes=40,new_player_flow_review='PENDING_FINITE_PREFIX_ONLY_NOT_CHANNEL_ACCEPTANCE',next=h['next']);v['completed'].append('R505：來源243/244cdh十格表與sentinel前段、新epoch/table/selected退役6新流程PASS；精確外部地址函式與64AD74initializer缺口交脫殼組；最低頻道玩家路徑未完成、GUI0、overall16/40。');save(pp,v);save(L/'r505-continuation.json',v['r505']);save(L/'r505-final-audit.json',dict(state=h['state'],handoff=hp,text=pin(txt),independent_contract=pin(L/'r505-independent-contract.json'),unpacker_dependency=h['unpacker']['minimum_dependency'],GUI=0,full_channel_roundtrip=False,overall_complete=False))
print('R505 sealed',hp['sha256'],'prefix6flowsPASS/fullchannelOPEN')
