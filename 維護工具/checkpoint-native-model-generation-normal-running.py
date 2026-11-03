"""Register a currently observed original build; do not qualify an in-flight image."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-model-generation-normal');c=read(run/'cohort.json');app=Path(c['application']);runner=read(run/'original-runner-process.json')
assert c['status']=='building_model_generation_normal' and c['owned_process'] is not None
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
jobs=[runner,c['owned_process']];ids=','.join(str(j['pid']) for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig'));live=[live] if isinstance(live,dict) else live
for j in jobs:assert any(row['ProcessId']==j['pid'] and abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())<.001 for row in live)
before=read(app/'tracker-readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/'active-checkpoint';out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
(out/'live-process-CIM.json').write_bytes(raw)
e=dict(status='running_original_model_generation_normal_build_and_related_regressions',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session_id,original_runner=runner,current_profile=c['active_profile'],snapshot_owned_process=c['owned_process'],recorded_runner_and_child_confirmed_live=True,original_terminal_exit_code=None,normal_related_validation_complete=False,compiled_source_archive=c['compiled_source_archive'],integration_targets_each=1,selected_library_groups_each=0,opted_in_original_resource_tests_each=0,normal_GUI_runtime_inferred=False,whole_goal_complete=False)
save(out/'evidence.json',e)
work='來源'+c['source_manifest_sha256'][:8]+'已整合Model世代界限一個Runtime檔與三個新回歸；原'+str(a.session_id)+'經唯一入口建置正常兩版及每版23個直接相關場景案例，須續等原程序實際終止並獨立核對，不先標通過。CBC7每版164及同地圖新世代失敗保留原來源，隔離候選兩版各3案已核對。完整廣域／library與當版G06仍待，8／60秒、Release與G11界限不變；相鄰只讀且產物獨立，正常GUI／實體音效及完整G01–G12未完成。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        x.update(current_native_model_generation_normal_active=e,active_workloads={'model_generation_normal':str(run)},active_workload_sessions={'model_generation_normal':a.session_id},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='model-generation-normal-build-and-regression',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d') and sep;p.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(run.parent/'current-continuation.json',dict(status=e['status'],run=str(run),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=a.session_id,original_terminal_exit_code=None,must_poll_original_session_before_restarting=True,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),next_action=work,whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),session=a.session_id,original_runner_and_child_live=True,terminal_exit_code=None),ensure_ascii=False))
