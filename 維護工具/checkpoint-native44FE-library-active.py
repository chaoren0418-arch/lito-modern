"""Record a birth-verified full-library cohort after exact-source broad was sealed."""
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

ability_prep=read(ROOT/'維護工具/candidates/native44FE-ability-buff-requests/preparation.json')
assert ability_prep['world_integration_typecheck_verified']and ability_prep['world_integration_metadata_only']and not ability_prep['world_integration_executed']
metadata_path=Path(ability_prep['world_integration_typecheck']);metadata=read(metadata_path)
assert sha(metadata_path)==ability_prep['world_integration_typecheck_sha256']and metadata['status']=='isolated_whole_crate_metadata_typecheck_qualified_only'
assert not metadata['machine_code_generated']and not metadata['tests_executed']and not metadata['normal_product_modified']
metadata_record=dict(qualification=str(metadata_path),qualification_sha256=sha(metadata_path),candidate_manifest_sha256=metadata['candidate_manifest_sha256'],whole_crate_metadata_typecheck_verified=True,machine_code_generated=False,tests_executed=False,actual_sessions=0,normal_product_modified=False)
parser = argparse.ArgumentParser()
parser.add_argument('run', type=Path)
parser.add_argument('--session-id', type=int, required=True)
parser.add_argument('--refresh',action='store_true')
args = parser.parse_args()
run = args.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native44FE-library-default') and args.session_id > 0
cohort = read(run/'cohort.json')
job = dict(pid=cohort['pipeline_pid'],creation_utc=cohort['pipeline_creation_utc'])
assert cohort['status'] in ['compiling_current_full_library_test_image','running_current_full_library_default_selection']
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(job['pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
live = json.loads(subprocess.check_output(['pwsh', '-NoProfile', '-Command', command]).decode('utf-8-sig'))
assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds()) < .001
broad = Path(cohort['input_cohort'])
qualified = read(broad/'completion-observation.json')
assert qualified['source_manifest_sha256'] == cohort['source_manifest_sha256']
assert qualified['observed_terminal_exit_code'] == 0 and qualified['recorded_original_owned_processes_all_terminal']
normal = Path(cohort['normal_qualification']).parent.parent
for name,digest in read(normal/'compiled-inputs.json').items():
    assert sha(ROOT/name) == digest,name
for name,digest in cohort['validation_pins'].items():
    assert sha(ROOT/'維護工具'/name) == digest,name
before = read(read((run if args.refresh else broad).parent/'current-continuation.json')['latest_readback'])['after_files']
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
record = dict(status='running_current44FE_full_library_default_cohort', run=str(run), source_manifest_sha256=cohort['source_manifest_sha256'], original_session_id=args.session_id, pipeline=job, observed_live=live, qualification_granted=False, GUI_verified=False, physical_input_verified=False, physical_audio_verified=False, whole_G06_complete=False, whole_goal_complete=False)
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
save(out/'evidence.json', record)
work = '當版44FEA738廣域原67934之兩版各112整合目標＋main限定範圍已獨立核對。完整library default原'+str(args.session_id)+'沿同一程序編譯、盤點、忽略來源稽核及實際執行，產品與對應QA固定，完整library尚未授予資格；須沿原session取得實際終止再獨立核對。兩種能力強化／S6B-S6C隔離候選之5案元件與整庫metadata型別檢查已通過，沒有資源或實際施法資格，未改正常產品。WORLD8sessions、旅館16sessions及控制器／正常模組音效各96sessions維持各自範圍，同端點契約失敗保留；74技能書ID辨識35／缺39未改。先自行完成開發及自動端到端後才人工驗收，開發期間不請使用者登入；8秒／60秒／Release與G11預算不變，相鄰只讀且產物獨立，完整G01–G12未完成。'
if profile_records:work+='完整library已獨立核對之profile：'+'；'.join(p+str(q['passed'])+'過0敗'+str(q['ignored'])+'忽略'for p,q in profile_records.items())+'，忽略本文未執行；其他profile及整批終止尚待原程序完成。'
for name in before:
    path = ROOT/name
    if path.suffix == '.json':
        value = read(path); value['current_native44FE_ability_metadata']=metadata_record
        assert value['current_product_identity']['source_manifest_sha256'] == cohort['source_manifest_sha256']
        value.update(current_native44FE_library_active=record, active_workloads={'full_library':record}, active_workload_sessions={'full_library':args.session_id}, source_and_corresponding_QA_frozen=True, next_task=work, next_focus=work, last_goal_turn_classification='progress', last_goal_turn_evidence=str(out/'evidence.json'), goal_complete=False, whole_goal_complete=False)
        value['current_focus'] = dict(item='current44FE-full-library-continuation', work=work, applies_to_current_source=True, goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        save(path, value)
    else:
        first, separator, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert separator; datetime.strptime(first[:10],'%Y-%m-%d')
        path.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest, encoding='utf-8', newline='\n')
save(out/'readback.json', dict(before_files=before, after_files={name:sha(ROOT/name) for name in before}, before_archive_sha256=sha(out/'before.zip'), evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name) == digest for name, digest in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json', dict(**record, active_checkpoint=str(out/'readback.json'), latest_readback=str(out/'readback.json'), latest_checkpoint=str(out), must_poll_original_session_before_restarting=True, next_action='Read original library terminal, independently qualify Debug and Release default bodies and ignored inventory, then observe exact process identities and completed checkpoint. Preserve failure; no automatic restart.'))
print(json.dumps(dict(checkpoint=str(out), original_session_id=args.session_id, live_pipeline=live), ensure_ascii=False))
