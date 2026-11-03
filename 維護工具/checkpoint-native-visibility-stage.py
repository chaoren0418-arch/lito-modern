"""Keep the applied visibility source separate from the qualified 6D baseline."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-detection-invisibility/application-20261017-015820-458055'
RUN=ROOT/'驗證紀錄/current/native-detection-invisibility/Debug-preflight-20261017-020935-621024'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
a=read(APP/'apply-receipt.json');c=read(RUN/'cohort.json');pins=read(APP/'applied-compiled-inputs.json')
pointer=APP/'current-stage.json';session_id=37519;source_sha=a['source_manifest_sha256'];repair=None;integration=None
if pointer.exists():
    stage=read(pointer);RUN=Path(stage['run']).resolve(strict=True);assert RUN.is_relative_to(APP.parent)
    session_id=stage['session_id'];assert isinstance(session_id,int) and session_id>0
    snapshot=Path(stage['input_snapshot']).resolve(strict=True);assert snapshot.is_relative_to(APP)
    integration=read(snapshot/'integration.json') if (snapshot/'integration.json').exists() else None
    reference=Path(integration['parent_snapshot']) if integration else snapshot
    repair=read(reference/'repair.json');assert Path(repair['application']).resolve()==APP
    pins=read(snapshot/'compiled-inputs.json');source_sha=sha(snapshot/'source-inputs.json')
    assert source_sha==(integration or repair)['source_manifest_sha256']==stage['source_manifest_sha256']
    c=read(RUN/'cohort.json')
assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
assert c['source_manifest_sha256']==source_sha
qualified=RUN/'qualified/qualified.json';done=qualified.exists()
active=c['status'] not in ['failed_preserved','executed_product_visibility_Debug_preflight_and_unique_normal']
record=dict(status=c['status'],application=str(APP),apply_receipt_sha256=sha(APP/'apply-receipt.json'),
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=source_sha,changed_sources=a['changed_sources'],
    candidate_applied_to_product=True,Debug_run=str(RUN),original_Debug_session_id=session_id,Debug_qualified=done,
    source_and_runner_frozen=active,Release_qualified=False,actual_new_formal_sessions=0,actual_new_casts=0,
    last_qualified_learned_request_recognition_count=21,last_qualified_missing_learned_requests=53,
    new_source_support_qualification_pending=True,GUI_verified=False,whole_goal_complete=False)
if done:
    q=read(qualified);assert q['observed_terminal_exit_code']==0 and q['original_session_id']==session_id
    record.update(Debug_qualification=str(qualified),Debug_qualification_sha256=sha(qualified),Debug_counts=q['counts'],Debug_normal_sha256=q['normal_sha256'])
live_active=False;live_qualified=False;live_session=None
if (APP/'live-stage.json').exists():
    live_stage=read(APP/'live-stage.json');live_run=Path(live_stage['run']).resolve(strict=True)
    assert live_run.is_relative_to(APP.parent)
    live=read(live_run/'cohort.json');assert live['source_manifest_sha256']==source_sha
    live_session=live_stage['session_id'];assert isinstance(live_session,int) and live_session>0
    live_active=live['status']=='running_normal_Debug_visibility_cases'
    live_path=live_run/'qualified/qualified.json';live_qualified=live_path.exists()
    record['Debug_live_cases']=dict(run=str(live_run),original_session_id=live_session,status=live['status'],
        qualified=live_qualified,observed_compared_cases=sum(v.get('status')=='executed_and_independently_compared' for v in live['cases']))
    if live_qualified:
        lq=read(live_path);assert lq['original_session_id']==live_session and lq['observed_terminal_exit_code']==0
        assert lq['source_manifest_sha256']==source_sha
        record.update(actual_new_formal_sessions=lq['actual_client_sessions'],actual_new_casts=lq['accepted_casts'],
            actual_cast_requests=lq['actual_cast_requests'],negative_cases=lq['negative_cases'])
        record['Debug_live_cases'].update(qualification=str(live_path),qualification_sha256=sha(live_path))
release_active=False;release_done=False;release_session=None;release_live_active=False;release_live_done=False;release_live_session=None
if (APP/'release-stage.json').exists():
    rs=read(APP/'release-stage.json');release_run=Path(rs['run']).resolve(strict=True)
    assert release_run.is_relative_to(APP.parent) and rs['source_manifest_sha256']==source_sha
    rc=read(release_run/'cohort.json');assert rc['profile']=='Release' and rc['source_manifest_sha256']==source_sha
    release_session=rs['session_id'];assert isinstance(release_session,int) and release_session>0
    release_active=rc['status'] not in ['failed_preserved','executed_product_visibility_Release_preflight_and_unique_normal']
    release_q=release_run/'qualified/qualified.json';release_done=release_q.exists()
    record['Release_preflight']=dict(run=str(release_run),original_session_id=release_session,status=rc['status'],qualified=release_done)
    if release_done:
        rq=read(release_q);assert rq['profile']=='Release' and rq['observed_terminal_exit_code']==0 and rq['original_session_id']==release_session
        record.update(Release_qualified=True,Release_normal_sha256=rq['normal_sha256'],Release_counts=rq['counts'])
        record['Release_preflight'].update(qualification=str(release_q),qualification_sha256=sha(release_q))
if (APP/'release-live-stage.json').exists():
    rs=read(APP/'release-live-stage.json');release_live_run=Path(rs['run']).resolve(strict=True)
    assert release_live_run.is_relative_to(APP.parent)
    rc=read(release_live_run/'cohort.json');assert rc['build_profile']=='Release' and rc['source_manifest_sha256']==source_sha
    release_live_session=rs['session_id'];assert isinstance(release_live_session,int) and release_live_session>0
    release_live_active=rc['status']=='running_normal_Release_visibility_cases'
    release_live_q=release_live_run/'qualified/qualified.json';release_live_done=release_live_q.exists()
    record['Release_live_cases']=dict(run=str(release_live_run),original_session_id=release_live_session,status=rc['status'],qualified=release_live_done,
        observed_compared_cases=sum(v.get('status')=='executed_and_independently_compared' for v in rc['cases']))
    if release_live_done:
        rq=read(release_live_q);assert rq['profile']=='Release' and rq['observed_terminal_exit_code']==0 and rq['original_session_id']==release_live_session
        assert rq['source_manifest_sha256']==source_sha
        record['actual_new_formal_sessions']+=rq['actual_client_sessions'];record['actual_new_casts']+=rq['accepted_casts']
        record['actual_cast_requests']+=rq['actual_cast_requests'];record['negative_cases']+=rq['negative_cases']
        record['Release_live_cases'].update(qualification=str(release_live_q),qualification_sha256=sha(release_live_q))
record['source_and_runner_frozen']=active or live_active or release_active or release_live_active
g06_active=False;g06_done=False;g06_record=None
if (APP/'current-G06-stage.json').exists():
    gs=read(APP/'current-G06-stage.json');g06_run=Path(gs['run']).resolve(strict=True)
    assert g06_run.is_relative_to(ROOT/'驗證紀錄/current/native-G06-controller')
    gc=read(g06_run/'cohort.json');assert gc['source_manifest_sha256']==source_sha
    g06_session=gs['session_id'];assert isinstance(g06_session,int) and g06_session>0
    g06_active=gc['status'] not in ['failed_preserved','executed_current_both_profiles_G06_controller_TW_extended_six_cases']
    completion=g06_run/'completion-observation.json';g06_done=completion.exists()
    g06_record=dict(run=str(g06_run),original_session_id=g06_session,status=gc['status'],qualified=g06_done,
        source_manifest_sha256=source_sha,source_and_corresponding_QA_frozen=g06_active)
    if gc['status']=='failed_preserved' and (g06_run/'failure-before-repair/readback.json').exists():
        failed=read(g06_run/'failure-before-repair/readback.json')
        assert failed['original_session_id']==g06_session and failed['observed_terminal_exit_code']==1
        g06_record.update(failed_original_retained=True,failure_readback=str(g06_run/'failure-before-repair/readback.json'),
            failure_archive_sha256=failed['raw_archive_sha256'],error=gc.get('error'))
    if g06_done:
        gq=read(completion);assert gq['original_session_id']==g06_session and gq['observed_terminal_exit_code']==0
        assert gq['source_manifest_sha256']==source_sha and gq['actual_qualified_formal_sessions']==96 and gq['recorded_original_owned_processes_all_terminal']
        g06_record.update(completion_observation=str(completion),completion_sha256=sha(completion),actual_qualified_formal_sessions=96)
    record['source_and_runner_frozen']=record['source_and_runner_frozen'] or g06_active
if (APP/'qualified-pair-stage.json').exists():
    pair_stage=read(APP/'qualified-pair-stage.json');pq=Path(pair_stage['qualification'])
    assert pair_stage['source_manifest_sha256']==source_sha and sha(pq)==pair_stage['qualification_sha256']
    record['limited_both_profile_prerequisite']=pair_stage
files=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json','README.md','docs/HISTORY.md']
out=APP/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in files}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert set(z.namelist())==set(files)
    for n,h in before.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
note=('來源6D189143完整library雙版各1935/0/473已80951actualexit0、逐項2408/各31PNG/1719報告還原及11組記錄程序退出獨立核對；'
    '同版G06六分支96sessions、map563120sessions與warm合成p95167899微秒保留各自限定資格。'
    '偵測/隱身隔離模組98810actualexit0已獨立讀回：13之140104、60之SELF_TICK/unsigned32、各6拒絕、選單、749之23幀/24sample/sound108一次；'
    '完整TW prepare未由隔離模組編譯，無真實施法/揭露，原生呼叫者時鐘未認定；候選使用已查核ABI的Windows uptime provider。'
    '原紅測試885dc1為Rust101/wrapper1、舊版不支援13，映像/來源/raw已封存，舊正常兩主逐byte不變。'
    '現已套用五檔，來源90F3A28B，接入13/60請求/選單、749效果與初始隔離fixture；')
if repair:
    note=note.replace('來源90F3A28B','初次來源90F3A28B')
    note+=f'原37519實際wrapper1/Rust101，新增749測試誤讀shard而MissingEntry；原失敗已封存942E7F0E。僅測試改讀實際base Sprite，runtime不變，修後來源{source_sha[:8]}。'
    record['reference_only_repair']=repair
    if integration:
        note+='正常probe已接雙玩家隱身/偵測路徑，另改既有兩檔，等待實際正常EXE驗證；'
        record['normal_probe_integration']=integration
        record['changed_sources']=record['changed_sources']+integration['changed_paths']
        if integration.get('retained_positive_case'):
            positive_path=Path(integration['retained_positive_case']);positive=read(positive_path)
            assert positive['actual_cast_requests']==4 and positive['final_independent_reader_exit_code']==0
            record['retained_positive_case']=dict(qualification=str(positive_path),sha256=sha(positive_path),
                source_manifest_sha256=positive['source_manifest_sha256'],actual_client_sessions=4,actual_cast_requests=4,
                applies_to_current_source=positive['source_manifest_sha256']==source_sha)
            note+='來源3B099C42之Debug正常TW雙玩家已4sessions/4實際施法，原61978wrapper1之reader路徑/S15猜測錯誤已封存，修後獨立2e49eb/352960actual0；本人Half/他人移除/新目標及749各端兩次/sound108各兩次符，三張診斷PNG已目視，GUI未驗。當前只修probe靜默拒絕等待，這組保留原3B來源。'
note+=(f'Debug四實際測試映像及唯一入口正常已原{session_id}actualexit0並獨立讀回；Release/真實施法揭露仍待。' if done else
       f'Debug完整產品模組/三整合目標+main及唯一入口正常沿原{session_id}，當前{c["status"]}，尚未算通過；Release/真實施法揭露仍待。')
note+='原74/21/53是6D之資格，不以新來源接入宣告玩法完成；舊scope不移記90F3。廣域回歸待接入後版本；登入器只讀，完整G01–G12未完成。'
if live_session:
    note+=(f'當前{source_sha[:8]}之Debug TW/legacy八案已原{live_session}actualexit0及獨立逐案/32組退出/raw核對：32sessions、14請求、8接受/6拒絕、749/音效/PNG RGBA原hash符；Release與廣域仍待。' if live_qualified else
        f'當前{source_sha[:8]}之Debug TW/legacy八案沿原{live_session}，已獨立比較{record["Debug_live_cases"]["observed_compared_cases"]}案，尚未記為整批通過。')
if live_qualified:
    note=(f'當前來源{source_sha[:8]}：偵測13／隱身60已接正常請求、選單與749效果。Debug四實際測試EXEs45/0/0及唯一入口正常主已獨立核對；'
        f'TW與legacy八實際雙玩家案原{live_session}actualexit0，32 LOGIN/GAME sessions、14請求（8接受／6拒絕），'
        'MP／一顆寶石、S34本人Half／S15他人移除、偵測重新出現與舊目標失效、749／sound108事件、PNG RGBA原hash及32組記錄程序退出均獨立核對，四張診斷PNG已目視。'
        '原61978 reader錯誤與61582 CP950批次錯誤已封存；沿用已完成首案續跑，未改原失敗。'
        '下一步以同來源建置Release並跑相同八案，再補當版完整library／G06／map563／廣域與必要原資源回歸。'
        '6D之library／G06／map563／warm及74/21/53僅保留原版本資格；GUI／實體輸入／音效裝置／clean host／G11–G12與完整G01–G12仍待。'
        '重建版與登入器獨立，登入器只讀，替換提案已撤回。')
if release_session:
    note+=(f'Release四目標與唯一入口正常主已原{release_session}actualexit0及獨立讀回，計{record["Release_counts"]}；' if release_done else
        f'Release四目標與唯一入口正常主正沿原{release_session}執行，{rc["status"]}，來源及該runner固定，尚未算通過；')
if release_live_session:
    note+=(f'Release TW/legacy八案亦已原{release_live_session}actualexit0與獨立raw／退出核對；兩版合64sessions、28請求、16接受／12拒絕，當版library／G06／map563／廣域仍待。' if release_live_done else
        f'Release八案正沿原{release_live_session}，已獨立比較{record["Release_live_cases"]["observed_compared_cases"]}案，整批尚未算通過。')
if live_qualified and release_live_done:
    record['status']='limited_visibility_both_profiles_qualified_broad_and_other_gameplay_pending'
    note=(f'當前來源{source_sha[:8]}：Debug四實際測試EXEs45/0/0、Release四實際EXEs41/0/0與唯一入口正常兩主均已獨立核對。'
        f'TW／legacy兩版各八雙玩家案，原Debug{live_session}與Release{release_live_session}均actualexit0：合64 LOGIN/GAME sessions、28請求（16接受／12拒絕）；'
        'MP／一顆寶石、S34本人Half／S15他人移除、偵測重新出現與舊目標失效、749／sound108事件、PNG RGBA原hash與兩批各32組記錄程序退出均符，兩版共八張診斷PNG已目視。'
        '只代表這組控制器／繪圖案例，未認定GUI／實體音效或所有技能完成；SELF_TICK以Windows uptime供應，原生呼叫者時鐘來源仍未核實。'
        '下一步先補同版G06六分支96sessions，再廣域111目標／map563／完整library與必要原資源回歸；6D之既有結果及74/21/53僅保留原版本資格。'
        '原61978 reader及61582 CP950失敗已封存。GUI／physical／IME／多DPI／clean host／G11–G12與完整G01–G12未完成；登入器只讀，替換提案已撤回。')
if g06_record:
    note+=(f'當版G06六分支兩版亦已原{g06_session}actualexit0、96sessions與記錄owned退出獨立核對；尚未代表GUI或同端點成功。' if g06_done else
        f'當版G06原{g06_session}已wrapper1／Rust101／核心1，在傳送前第三步後的八秒繪圖就緒等待失敗；原始來源／兩測試PE／端點紀錄已封存ADFC6F71。不得算通過，亦不移記6D之96sessions。' if gc['status']=='failed_preserved' else
        f'當版G06六分支正沿原{g06_session}，當前{gc["status"]}，來源及對應QA固定；不以舊6D之96sessions移記。')
diagnostic_active=False;diagnostic=None
if (APP/'movement-diagnostic-stage.json').exists():
    diagnostic=read(APP/'movement-diagnostic-stage.json');assert diagnostic['source_manifest_sha256']==source_sha
    diagnostic_active=diagnostic['active_at_last_verified_snapshot'];record['movement_timing_diagnostic']=diagnostic
    record['source_and_runner_frozen']=record['source_and_runner_frozen'] or diagnostic_active
    note+='隔離探針的預設設定對照曾成功，未用來通過TW失敗；相同TW設定另已Rust101／核心1，raw89EB029E保留。'
    if diagnostic.get('timing_observation'):
        td=Path(diagnostic['timing_observation']);assert sha(td)==diagnostic['timing_observation_sha256']
        timing=read(td);assert timing['original_session_id']==8299 and timing['observed_terminal_exit_code']==1
        note+='TW耗時探針原8299亦失敗，raw3270436E已封存；Debug Model載入A16.7秒／B2.7秒，後段取樣繪圖120–446ms且動畫可回idle，但B在行走時被fixture結束。此資料未定論原八秒失敗根因，非G11接受證據；產品來源與八秒／六十秒門檻均未改。'
    else:
        note+=f'現沿原{diagnostic["original_session_id"]}{"執行明示TW設定的" if diagnostic["phase"].startswith("running") else "編譯"}同版rlib載入／逐幀耗時診斷，原八秒移動與六十秒accept門檻不變；產品來源未修，非G11效能接受證據。'
release_control_active=False;release_control=None
if (APP/'release-G06-control-stage.json').exists():
    control_stage=read(APP/'release-G06-control-stage.json');control_path=Path(control_stage['run'])
    control=read(control_path/'build.json');assert control['source_manifest_sha256']==source_sha
    release_control_active=control['status']=='compiling_original_Release_G06_control_targets'
    release_control=dict(control_stage,status=control['status'],compile_exit_code=control.get('compile_exit_code'),
        source_and_corresponding_inputs_frozen=release_control_active,actual_new_sessions=0,G06_qualified=False)
    record['original_Release_G06_control']=release_control
    record['source_and_runner_frozen']=record['source_and_runner_frozen'] or release_control_active
    control_q=control_path/'cross-endpoint-control/qualified-control.json'
    if control_q.exists():
        cq=read(control_q);assert cq['observed_terminal_exit_code']==0 and cq['actual_formal_sessions']==6
        packet=control_q.parent/'independent-packet-readback.json';pr=read(packet)
        assert pr['original_control_qualification_sha256']==sha(control_q) and pr['independent_verifier_exit_code']==0
        release_control.update(actual_new_sessions=6,qualification=str(control_q),qualification_sha256=sha(control_q),
            packet_readback=str(packet),packet_readback_sha256=sha(packet),original_control_runtime_session_id=cq['original_session_id'])
        note+='原樣Release兩G06目標原91349已actualexit0；原66433之TW跨端點六sessions亦actualexit0及獨立封包／接續核對，只為Release對照，未改記Debug失敗或整體G06。'
    elif release_control_active:
        note+=f'正沿原{control_stage["original_session_id"]}建置原樣Release G06兩目標作對照，尚未完成。'
    else:
        note+='原樣Release G06兩目標建置已終止，跨端點資格仍待獨立讀回。'
phase_active=False;phase_record=None
if (APP/'phase-gated-G06-stage.json').exists():
    ps=read(APP/'phase-gated-G06-stage.json');phase_run=Path(ps['run']);pc=read(phase_run/'cohort.json')
    assert pc['source_manifest_sha256']==source_sha and pc['cross_endpoint_fixture_override']
    phase_active=pc['status'] not in ['failed_preserved','executed_current_both_profiles_G06_controller_TW_extended_six_cases']
    phase_record=dict(ps,status=pc['status'],active_at_last_verified_snapshot=phase_active,
        fixture_override=pc['cross_endpoint_fixture_override'],G06_qualified=False)
    phase_completion=phase_run/'completion-observation.json'
    if phase_completion.exists():
        pq=read(phase_completion)
        assert pq['original_session_id']==ps['original_session_id'] and pq['observed_terminal_exit_code']==0
        assert pq['actual_qualified_formal_sessions']==96 and pq['recorded_original_owned_processes_all_terminal']
        assert pq['source_manifest_sha256']==source_sha
        phase_record.update(G06_qualified=True,completion_observation=str(phase_completion),completion_sha256=sha(phase_completion),
            actual_qualified_formal_sessions=96)
    record['phase_gated_G06_controller']=phase_record
    record['source_and_runner_frozen']=record['source_and_runner_frozen'] or phase_active
    note+='目的地C之accept計時已另以候選fixture對齊既有A傳送完成階段；原Debug PE控制案10052／獨立封包核對通過六sessions，rawE9261B98。產品來源未變，八秒／六十秒／整體120秒未調整。'
    if phase_record['G06_qualified']:
        note+=f'原{ps["original_session_id"]}之雙版六分支已actualexit0、96sessions及逐案封包／背包／記錄程序退出核對；只取得明示fixture修正下的控制器資格，GUI與完整G06仍待，原45538失敗保留。'
    elif pc['status']=='failed_preserved':
        failure_path=phase_run/'failure-before-repair/readback.json'
        if failure_path.exists():
            pf=read(failure_path);assert pf['original_session_id']==ps['original_session_id'] and pf['observed_terminal_exit_code']==1
            phase_record.update(failure_readback=str(failure_path),failure_readback_sha256=sha(failure_path),failure_archive_sha256=pf['raw_archive_sha256'])
            note+=f'原{ps["original_session_id"]}已wrapper1／Rust101／核心0，前三Debug案成功但wrong-service中A等B完成超六十秒；兩角色恢復報告皆存在仍未算通過，raw{pf["raw_archive_sha256"][:8]}與原45538失敗保留。Release未啟動。'
        else:
            note+=f'原{ps["original_session_id"]}之雙版六分支再遇失敗，保留原始紀錄並待封存定位；未算通過，亦未改記原45538失敗。'
    else:
        note+=f'雙版六分支正沿原{ps["original_session_id"]}，當前{pc["status"]}；尚未整批通過，原45538失敗保留。'
town_run=ROOT/'驗證紀錄/current/native-G06-controller/20261017-030505-804788/town-load-replay/20261017-033321-769292/independent-readback/readback.json'
if town_run.exists():
    tq=read(town_run);assert tq['observed_terminal_exit_code']==0 and tq['actual_new_sessions']==0
    record['offline_town_load_diagnostic']=dict(readback=str(town_run),sha256=sha(town_run),results=tq['results'],performance_accepted=False)
    note+='離線捕獲租裝村雙角色載入／首draw已36698actualexit0及獨立封存D7F0F24F：Debug載入2.9／4.4秒、首draw1.4／2.6秒，Release載入各約2.6秒、首draw0.18–0.23秒。未清OS cache、未排除主機其他負載，原八秒失敗根因仍未定論，非G11接受證據。'
failure_timing_active=False;failure_timing=None
if (APP/'failure-timing-stage.json').exists():
    failure_timing=read(APP/'failure-timing-stage.json');assert failure_timing['source_manifest_sha256']==source_sha
    failure_timing_active=failure_timing['active_at_last_verified_snapshot']
    record['failure_recovery_timing_diagnostic']=failure_timing
    record['source_and_runner_frozen']=record['source_and_runner_frozen'] or failure_timing_active
    if failure_timing.get('runtime_readback'):
        tr=Path(failure_timing['runtime_readback']);assert sha(tr)==failure_timing['runtime_readback_sha256']
        ft=read(tr);assert ft['observed_terminal_exit_code']==1 and ft['original_session_id']==75086
        note+='恢復階段時間探針原75086已wrapper1／Rust101／核心1，raw2257024E封存；A初次Model載入11.9秒、恢復Model6.9秒，約85.8秒才完成恢復，B在初次Model前之六十秒等待先逾時，fixture其後accept逾時。沒有修復或G11因果認定；首份NUL診斷映像封存未執行，產品來源及各時限不變。'
    else:
        note+=f'正沿原{failure_timing["original_session_id"]}執行恢復階段時間探針準備／量測，產品來源及原有各時限不變；首份路徑生成NUL之診斷映像已封存且未執行，修後另建，不計任何通過。'
remote_active=False;remote_stage=None
if (APP/'remote-actor-replay-stage.json').exists():
    remote_stage=read(APP/'remote-actor-replay-stage.json');rr=read(Path(remote_stage['run'])/'cohort.json')
    assert rr['source_manifest_sha256']==source_sha
    remote_active=rr['status']=='running_offline_14_versus_15_actor_town_replay'
    record['offline_remote_actor_replay']=dict(remote_stage,status=rr['status'],active_at_last_verified_snapshot=remote_active)
    if remote_stage.get('readback'):
        rqpath=Path(remote_stage['readback']);assert sha(rqpath)==remote_stage['readback_sha256']
        rq=read(rqpath);assert rq['observed_terminal_exit_code']==0 and rq['original_session_id']==96616
        note+='原96616之離線14／15actor兩版對照已actualexit0及獨立封存9B3D23AD；Release首draw各約0.20秒，未見額外遠端Mage造成大幅成本。首次Debug載入28秒，其中封存檔觀察／hash23.65秒；同程序下一Model該階段0.88秒。順序／OS cache／主機其他負載未隔離，無新session或G11因果接受。'
        cache_plan=Path(remote_stage['run']).parent/'shared-archive-owner-next-action.json'
        if cache_plan.exists():
            cp=read(cache_plan);assert cp['source_manifest_sha256']==source_sha and not cp['source_pool_reuse_implemented']
            record['shared_archive_owner_next_action']=dict(path=str(cache_plan),sha256=sha(cache_plan),applied=False)
            note+='下一步先實作及驗證場景間共用已驗證且持續禁止寫／刪的資源擁有者候選；初次完整hash、取消、來源漂移、弱引用最終回收與像素一致性都須保留，尚未套產品。登入器／伺服器來源維持只讀且非部署依賴。'
    else:
        note+=f'實際失敗場景15actors與原離線14actors有差異；現沿原{remote_stage["original_session_id"]}，以既有不可變Debug／Release探針映像比較14actor基準與額外一筆匹配實際snapshot之遠端Mage封包，狀態{rr["status"]}。僅離線組合資料，無新session或GUI／效能接受。'
workloads={}
if active:workloads['visibility_Debug_preflight']=session_id
if live_active:workloads['visibility_Debug_cases']=live_session
if release_active:workloads['visibility_Release_preflight']=release_session
if release_live_active:workloads['visibility_Release_cases']=release_live_session
if g06_active:workloads['visibility_G06_controller']=g06_session
if diagnostic_active:workloads['movement_timing_diagnostic']=diagnostic['original_session_id']
if release_control_active:workloads['original_Release_G06_control_build']=control_stage['original_session_id']
if phase_active:workloads['phase_gated_G06_controller']=phase_record['original_session_id']
if failure_timing_active:workloads['failure_recovery_timing_diagnostic']=failure_timing['original_session_id']
if remote_active:workloads['offline_remote_actor_replay']=remote_stage['original_session_id']
retained_keys=['current_default110','current_geometry_full_library','current_geometry_G06_controller_validation',
    'current_map563_retained_geometry_revalidation','current_G11_warm_reload_followup','current_native_skill_support','current_wire_learned_native_skill_gap']
for n in files[:4]:
    v=read(ROOT/n)
    for k in retained_keys:
        if isinstance(v.get(k),dict):v[k]['applies_to_current_visibility_source']=False
    if 'retained_product_identity_before_visibility' not in v:v['retained_product_identity_before_visibility']=v.get('current_product_identity')
    v.update(current_visibility_application=record,source_freeze_for_visibility_Debug_preflight=active,
        source_freeze_for_visibility_live_cases=live_active,
        source_freeze_for_visibility_Release_preflight=release_active,source_freeze_for_visibility_Release_cases=release_live_active,
        source_freeze_for_visibility_G06_controller=g06_active,current_visibility_G06_controller_validation=g06_record,
        source_freeze_for_movement_timing_diagnostic=diagnostic_active,
        source_freeze_for_original_Release_G06_control=release_control_active,
        source_freeze_for_phase_gated_G06_controller=phase_active,
        source_freeze_for_failure_recovery_timing_diagnostic=failure_timing_active,
        current_phase_gated_G06_controller_validation=phase_record,
        active_workload_sessions=workloads,
        current_product_version_scope='source'+source_sha[:8]+('_visibility_both_eight_cases_qualified_broad_pending' if release_live_done and live_qualified else '_visibility_Debug_eight_cases_qualified_Release_and_broad_pending' if live_qualified else '_visibility_Debug_qualified_Release_and_actual_casts_pending' if done else '_visibility_Debug_pending_6D_scopes_retained'),
        current_product_identity=dict(source_manifest_sha256=source_sha,source_files=1024,compiled_input_files=1025,
            Debug_normal_sha256=record.get('Debug_normal_sha256'),Release_normal_qualified_for_current_source=release_done,
            Release_normal_sha256=record.get('Release_normal_sha256'),
            source_qualification_pending=not done,GUI_verified=False,whole_goal_complete=False),
        current_detection_invisibility_preparation=dict(candidate_applied_to_product=True,full_product_validation_pending=True,
            Debug_product_validation_pending=not done,Debug_live_cases_qualified=live_qualified,
            Release_product_validation_pending=not release_done,Release_live_cases_qualified=release_live_done,
            isolated_probe_qualification=str(ROOT/'驗證紀錄/current/native-detection-invisibility-candidate/20261017-015322-825372/independent-readback.json'),
            actual_new_casts=record['actual_new_casts'],whole_goal_complete=False),
        current_working_tree_verified=False,working_tree_modified_since_current_normal_qualification=not done,
        source_freeze_while_Cargo=active or release_active or release_control_active or (g06_active and gc['status']=='compiling_current_G06_controller_targets') or (phase_active and pc['status']=='compiling_current_G06_controller_targets'),source_freeze_for_live_library_job=False,
        next_task=note,last_goal_turn_classification='progress',last_goal_turn_evidence=[str(APP/'apply-receipt.json'),str(APP/'red-independent-readback.json'),str(RUN/'cohort.json')],
        progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    for item in v.get('items',[]):
        if item['id']=='G02-real-peer-battle':item.update(state='retained_melee_scope_source'+source_sha[:8]+'_ordinary_two_round_validation_pending',complete=False)
        if item['id']=='G03-current-regression':item.update(state='source'+source_sha[:8]+('_related_both_profiles_qualified_broad_library_G06_pending' if release_done and done else '_visibility_Debug_preflight_'+('qualified' if done else 'pending')+'_old6D_library_and_G06_retained'),complete=False)
        if item['id']=='G05-gameplay-matrix':item.update(state='source'+source_sha[:8]+('_visibility_both_eight_cases_qualified_other_gameplay_pending' if release_live_done and live_qualified else '_visibility_Debug_eight_cases_qualified_Release_pending' if live_qualified else '_visibility_actual_cast_validation_pending'),complete=False)
        if item['id']=='G06-room-service-branches':item.update(state='source'+source_sha[:8]+('_controller_six_branches_qualified_GUI_same_endpoint_and_map563_pending' if g06_done else '_controller_cross_endpoint_failed_movement_timing_investigation' if g06_record and g06_record.get('failed_original_retained') else '_controller_six_branches_running_old6D_scopes_retained' if g06_active else '_current_controller_scope_pending_old6D_retained'),complete=False)
        if item['id']=='G06-room-service-branches' and phase_record:
            item.update(state='source'+source_sha[:8]+('_phase_gated_fixture_controller_qualified_original_failure_retained_GUI_same_endpoint_and_map563_pending' if phase_record['G06_qualified'] else '_phase_gated_fixture_controller_running_original_failure_retained' if phase_active else '_phase_gated_fixture_controller_terminal_pending_readback_or_repair'),complete=False)
        if item['id']=='G08-player-gui':item.update(state='source'+source_sha[:8]+'_normal_compiled_GUI_physical_IME_multiDPI_pending',complete=False)
        if item['id']=='G09-audio':item.update(state='retained_WinMM_device_scope_source'+source_sha[:8]+'_sound_preparation_only_GUI_device_and_failure_paths_pending',complete=False)
        if item['id']=='G10-release-package':item.update(state='retained_old_candidate_source'+source_sha[:8]+'_portable_package_and_clean_host_not_yet_qualified',complete=False)
        if item['id']=='G11-stability-performance':item.update(state='retained_source6D_warm_budget_met_source'+source_sha[:8]+'_GUI_thirty_player_and_long_stability_pending',complete=False)
    save(ROOT/n,v)
for n in files[4:]:
    p=ROOT/n;header,_,body=p.read_text(encoding='utf-8-sig').partition('\n')
    p.write_text(header+'\n\n2026-10-17：'+note+'\n'+body,encoding='utf-8',newline='\n')
save(out/'record.json',dict(before=before,before_archive_sha256=sha(out/'before.zip'),after={n:sha(ROOT/n) for n in files},
    visibility_application=record,whole_goal_complete=False))
(out/'checkpoint-tool.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(checkpoint=str(out),source_manifest_sha256=source_sha,Debug_qualified=done,Debug_live_cases_qualified=live_qualified,actual_casts=record['actual_new_casts'],whole_goal_complete=False),ensure_ascii=False))
