"""Record a birth-verified full-library cohort after exact-source normal builds were independently sealed."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, hashlib, json, subprocess, time, zipfile

ROOT = Path(__file__).resolve().parents[1]

def read(path):
    for attempt in range(10):
        try:
            return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt == 9:
                raise
            time.sleep(.05)

def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

def save(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

parser = argparse.ArgumentParser()
parser.add_argument('run', type=Path)
parser.add_argument('--session-id', type=int, required=True)
parser.add_argument('--refresh',action='store_true')
args = parser.parse_args()
run = args.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-initial-source-confirmation-library-default') and args.session_id > 0
cohort = read(run/'cohort.json')
job = dict(pid=cohort['pipeline_pid'],creation_utc=cohort['pipeline_creation_utc'])
assert cohort['status'] in ['compiling_current_full_library_test_image','running_current_full_library_default_selection']
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(job['pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
live = json.loads(subprocess.check_output(['pwsh', '-NoProfile', '-Command', command]).decode('utf-8-sig'))
assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds()) < .001
broad = Path(cohort['input_cohort'])
qualified = read(broad/'qualified/qualified.json')
assert qualified['source_manifest_sha256'] == cohort['source_manifest_sha256']
assert qualified['observed_terminal_exit_code'] == 0 and qualified['recorded_owned_processes_all_terminal']
normal = Path(cohort['normal_qualification']).parent.parent
for name,digest in read(normal/'compiled-inputs.json').items():
    assert sha(ROOT/name) == digest,name
for name,digest in cohort['validation_pins'].items():
    assert sha(ROOT/'維護工具'/name) == digest,name
before = read(Path(read(ROOT/'工作佇列.json')['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(ROOT/name) == digest for name, digest in before.items())
queue=read(ROOT/'工作佇列.json')
if args.refresh:assert queue['active_workload_sessions']=={'full_library':args.session_id}and queue['source_and_corresponding_QA_frozen']
else:assert not queue.get('active_workloads')
out = run/('checkpoint-running-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')if args.refresh else'active-checkpoint')
out.mkdir()
with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(ROOT/name, name)
with zipfile.ZipFile(out/'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in before.items())
record = dict(status='running_current_catalog_full_library_default_cohort', run=str(run), source_manifest_sha256=cohort['source_manifest_sha256'], original_session_id=args.session_id, pipeline=job, observed_live=live, qualification_granted=False, GUI_verified=False, physical_input_verified=False, physical_audio_verified=False, whole_G06_complete=False, whole_goal_complete=False)
record['user_requested_goal_continuation'] = dict(
    requested_date='2026-10-19',
    objective='依總開發報告完成真正 Windows x64 LiTo 重建剩餘開發；先完成當版 Debug／Release 完整 library，再完成正常 GUI 全流程自動測試，確認無誤才交人工測驗，並持續更新總報告。',
    ordered_work=['current-source Debug and Release full library', 'normal GUI full-flow automation and required fixes', 'remaining G01-G12 development and final-source Release regression', 'human acceptance only after verified self-tests'],
    human_login_required=False, historical_blocker_revalidation='User explicitly resumed autonomous development and testing; no human-login wait is applicable.',
    whole_goal_complete=False)
profile_records={}
for profile in ['Debug','Release']:
    qp=run/profile/'full-default-qualified/qualified.json'
    if not qp.exists():continue
    q=read(qp);assert q['status']=='independently_qualified_current_full_library_default_selection'and q['profile']==profile
    assert q['source_manifest_sha256']==cohort['source_manifest_sha256']and q['failed']==q['filtered_out']==q['Rust_exit_code']==0
    assert q['passed']==cohort['profiles'][profile]['counts']['passed']and q['ignored']==cohort['profiles'][profile]['counts']['ignored']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']and q['recorded_profile_owned_processes_terminal']and not q['ignored_bodies_executed']
    profile_records[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),passed=q['passed'],failed=0,ignored=q['ignored'],inventory=q['inventory'],actual_test_sha256=q['actual_test_sha256'],ignored_bodies_executed=False)
record['independently_qualified_profiles']=profile_records
turn_classification='progress'if not args.refresh or profile_records!=queue.get('current_native_catalog_library_active',{}).get('independently_qualified_profiles',{})else'verified_wait'
record['goal_turn_classification']=turn_classification
save(out/'evidence.json', record)
work='來源A9FE9AD5原'+str(args.session_id)+'：當版正常兩版及相關19案已核對，完整library沿原程序執行、逐名稱與結果及終止待核對。初次GAME C8來源確認之自然超30秒／快速GUI與租裝目的地／恢復、廣域／控制器仍待；4F2F成果與Release49194來源到期失敗各保留原版。自行完整測試後才交人工，不請使用者操作；相鄰只讀，30秒來源／旅館數量、8秒／60秒與Release／G11預算不變。'
staged={}
for name in before:
    path = ROOT/name
    if path.suffix == '.json':
        value = read(path)
        assert value['current_product_identity']['source_manifest_sha256'] == cohort['source_manifest_sha256']
        value.update(current_native_catalog_library_active=record, active_workloads={'full_library':record}, active_workload_sessions={'full_library':args.session_id}, source_and_corresponding_QA_frozen=True, next_task=work, next_focus=work, last_goal_turn_classification=turn_classification, last_goal_turn_evidence=str(out/'evidence.json'), goal_complete=False, whole_goal_complete=False)
        value['current_user_requested_goal'] = record['user_requested_goal_continuation']
        value['current_focus'] = dict(item='current_catalog-full-library-continuation', work=work, applies_to_current_source=True, goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        staged[name]=(json.dumps(value,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first, separator, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert separator; datetime.strptime(first[:10],'%Y-%m-%d')
        staged[name]=((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(ROOT/name)==h for name,h in before.items())
for name,payload in staged.items():(ROOT/name).write_bytes(payload)
save(out/'readback.json', dict(before_files=before, after_files={name:sha(ROOT/name) for name in before}, before_archive_sha256=sha(out/'before.zip'), evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name) == digest for name, digest in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json', dict(**record, active_checkpoint=str(out/'readback.json'), latest_readback=str(out/'readback.json'), latest_checkpoint=str(out), must_poll_original_session_before_restarting=True, next_action='Read original library terminal, independently qualify Debug and Release default bodies and ignored inventory, then observe exact process identities and completed checkpoint. Preserve failure; no automatic restart.'))
print(json.dumps(dict(checkpoint=str(out), original_session_id=args.session_id, live_pipeline=live), ensure_ascii=False))
