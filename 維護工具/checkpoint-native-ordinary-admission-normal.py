"""Record qualified profile scopes without borrowing historical JAR/gameplay results."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--mode',choices=['Debug','complete'],required=True);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk');a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-ordinary-admission-normal');c=read(run/'cohort.json');complete=a.mode=='complete';profiles={}
for profile in ['Debug','Release']if complete else ['Debug']:
    matches=list(run.glob(profile+'-qualified-*/qualified.json'));assert len(matches)==1;qp=matches[0];q=read(qp)
    assert(q['passed'],q['failed'],q['ignored'],q['integration_passed'],q['selected_library_passed'])==(164,0,0,118,46)
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['recorded_owned_all_terminal'] and q['prior_resource_tests_preserved_byte_prefix'] and q['new_ordinary_admission_resource_regressions']==8
    assert sha(q['normal']['path'])==q['normal']['sha256'] and sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),normal_sha256=q['normal']['sha256'],passed=164,integration_passed=118,selected_library_passed=46)
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
if complete:
    o=read(run/'completion-observation.json');assert o['status']=='independently_qualified_ordinary_admission_normal_both_profiles_and_terminal_processes' and o['original_session']==a.session and o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal'] and o['cohort_sha256']==sha(run/'cohort.json')
    assert a.terminal_chunk and a.terminal_chunk.lower()not in ['pending','unknown'] and a.terminal_chunk==o['terminal_tool_chunk']
else:
    assert c['active_profile']=='Release' and c['owned_process'] is not None
    jobs=[read(run/'original-runner-process.json'),c['owned_process']];ids=','.join(str(j['pid'])for j in jobs);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
    raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig'));live=[live]if isinstance(live,dict)else live
    for job in jobs:assert any(row['ProcessId']==job['pid'] and abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds())<.001 for row in live)
contpath=run.parent/'current-continuation.json';cont=read(contpath);assert cont['original_session_id']==a.session;latest=Path(cont['latest_readback']).resolve(strict=True);assert latest.is_relative_to(run);before=read(latest)['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='both_normal_profiles_and_related_164_qualified'if complete else'Debug_normal_and_related_164_qualified_Release_original_running',source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,normal_profiles=profiles,original_session_id=a.session,original_terminal_exit_code=0 if complete else None,terminal_tool_chunk=a.terminal_chunk if complete else None,normal_related_validation_complete=complete,normal_product_registered=complete,full_library_verified=False,current_broad_default_verified=False,current_formal_compatibility_qualified=False,full_G06_verified=False,GUI_verified=False,physical_audio_verified=False,ordinary_S4A_and_S20_same_world_recreation_regressions_passed=True,invalid_source_ABA_regression_passed=True,new_connection_and_actor_reincarnation_regression_passed=True,pending_death_pose_regression_passed=True,direct_new_world_map_reset_regression_passed=True,valid_second_resource_source_verified=False,guardian_motion_verified=False,whole_goal_complete=False)
if complete:record.update(normal_completion=str(run/'completion-observation.json'),normal_completion_sha256=sha(run/'completion-observation.json'),normal_recorded_terminal_identities=o['recorded_original_owned_identities'])
known=V/'nativeCBC7-same-map-generation/20261018-154229-682111/receipt.json';known_receipt=read(known);assert known_receipt['baseline_child_exit_code']==101 and not known_receipt['baseline_regression_passed'] and known_receipt['source_manifest_sha256']==c['source_manifest_sha256']
record.update(known_same_map_new_generation_Model_failure=str(known),known_same_map_new_generation_Model_failure_sha256=sha(known),same_map_new_generation_Model_reuse_verified=False)
save(out/'evidence.json',record);work='來源'+c['source_manifest_sha256'][:8]+('正常兩版各164相關檢查過、各0敗0忽略，原'+str(a.session)+'實際退出0且全部記錄程序終止已獨立讀回。'if complete else'正常Debug164相關檢查過、0敗0忽略；Release沿原'+str(a.session)+'執行，須續等原程序終止，尚無兩版資格。')+'八個場景回歸涵蓋S4A／角色動作已發布與待發布取消、新事件一次、無效資源路徑ABA、新連線／角色重用、待發布死亡姿勢與新世界換圖隔離。每版17整合及4選定library群組含兩個明確opt-in原資源案例；另有同地圖新世界世代直接重用Model的死亡姿勢遺留回歸實際101，已保留；須先核對隔離世代界限修補，再恢復當版G06驗證。完整library、廣域與不同有效資源來源仍待。33199的G06控制器96sessions只保留原來源資格；v2跨圖缺陷未套入。8／60秒、Release及G11界限不變，相鄰只讀且產物獨立，正常GUI／實體音效及完整G01–G12未完成。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        j=read(p);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==c['source_manifest_sha256']
        for profile,proof in profiles.items():identity.update({profile+'_normal_sha256':proof['normal_sha256'],profile+'_normal_qualified_for_current_source':True})
        identity['source_qualification_pending']=not complete
        j.update(current_native_ordinary_admission_normal=record,source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if complete:j.update(active_workloads={},active_workload_sessions={})
        j['current_focus']=dict(item='ordinary-admission-current-G06'if complete else'ordinary-admission-Release-running',work=work,applies_to_current_source=True,goal_complete=False)
        for item in j.get('items',[]):
            if item['id'].startswith('G01')and complete:item.update(state='ordinary_admission_normal_baseline_qualified_limited',complete=True,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
            if item['id'].startswith(('G03','G06')):item.update(state='ordinary_admission_related_qualified_remaining_scope_pending'if complete else'ordinary_admission_Release_validation_running',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'])
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(p,j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d')and sep;p.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
(out/'continuation-before.json').write_bytes(contpath.read_bytes());cont.update(status=record['status'],normal_profiles=profiles,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),next_action=work,must_poll_original_session_before_restarting=not complete,original_terminal_exit_code=0 if complete else None);save(contpath,cont)
print(json.dumps(dict(checkpoint=str(out),profiles=list(profiles),normal_related_complete=complete,whole_goal_complete=False),ensure_ascii=False))
