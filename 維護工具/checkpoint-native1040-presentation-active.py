"""Record a birth-verified Presentation run after current G06 was sealed."""
from pathlib import Path
from datetime import datetime, timezone
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
args = parser.parse_args()
run = args.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-presentation-cohort') and args.session_id > 0
cohort = read(run/'cohort.json')
job = read(run/'runner-identity.json')
assert cohort['status'] in ['compiling_presentation_probes', 'executing_presentation_case']
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(job['pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
live = json.loads(subprocess.check_output(['pwsh', '-NoProfile', '-Command', command]).decode('utf-8-sig'))
assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds()) < .001
g06 = Path(cohort['G06'])
qualified = read(g06/'qualified/qualified.json')
completion = read(g06/'completion-observation.json')
assert qualified['source_manifest_sha256'] == completion['source_manifest_sha256'] == cohort['source_manifest_sha256']
assert qualified['actual_formal_sessions'] == 96 and completion['recorded_original_owned_processes_all_terminal']
for name, digest in read(run/'compiled-inputs.json').items():
    assert sha(ROOT/name) == digest, name
for name, digest in cohort['tool_pins'].items():
    assert sha(name) == digest, name
trackers = list(g06.glob('tracker-*/readback.json'))
assert len(trackers) == 1
before = read(trackers[0])['after']
assert len(before) == 8 and all(sha(ROOT/name) == digest for name, digest in before.items())
assert not read(ROOT/'工作佇列.json').get('active_workloads')
out = run/'active-checkpoint'
out.mkdir()
with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(ROOT/name, name)
with zipfile.ZipFile(out/'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in before.items())
record = dict(status='running_current1040_Presentation_cohort', run=str(run), source_manifest_sha256=cohort['source_manifest_sha256'], original_session_id=args.session_id, pipeline=job, observed_live=live, qualification_granted=False, GUI_verified=False, physical_input_verified=False, physical_audio_verified=False, whole_G06_complete=False, whole_goal_complete=False)
save(out/'evidence.json', record)
work = '來源4E1F7A4E控制器兩版六分支12案／96sessions已獨立讀回；同版非同步Presentation沿原'+str(args.session_id)+'續跑，先完成四診斷映像再執行兩版六分支，不以控制器資格代替繪圖退役驗證。產品與對應QA凍結，須沿原session讀取實際終止後獨立核對。正常兩版各90相關檢查保留；冰錐當版C02、廣域／完整library、正常GUI／實體輸入音效、同端點與其他服務及完整G06／G01–G12仍待。8／60秒、Release與G11預算不變，重建版與登入器產物獨立，相鄰來源只讀。'
for name in before:
    path = ROOT/name
    if path.suffix == '.json':
        value = read(path)
        assert value['current_product_identity']['source_manifest_sha256'] == cohort['source_manifest_sha256']
        value.update(current_native1040_Presentation_active=record, active_workloads={'Presentation':record}, active_workload_sessions={'Presentation':args.session_id}, source_and_corresponding_QA_frozen=True, next_task=work, next_focus=work, last_goal_turn_classification='progress', last_goal_turn_evidence=str(out/'evidence.json'), goal_complete=False, whole_goal_complete=False)
        value['current_focus'] = dict(item='current1040-Presentation-continuation', work=work, applies_to_current_source=True, goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        save(path, value)
    else:
        first, separator, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert first.startswith('2026-10-18') and separator
        path.write_text('2026-10-18：'+work+'\n\n'+rest, encoding='utf-8', newline='\n')
save(out/'readback.json', dict(before_files=before, after_files={name:sha(ROOT/name) for name in before}, before_archive_sha256=sha(out/'before.zip'), evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name) == digest for name, digest in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json', dict(**record, active_checkpoint=str(out/'readback.json'), must_poll_original_session_before_restarting=True, next_action='Read original terminal; independently qualify all twelve cases, verify recorded process identities and inspect diagnostic PNGs before completed checkpoint.'))
print(json.dumps(dict(checkpoint=str(out), original_session_id=args.session_id, live_pipeline=live), ensure_ascii=False))
