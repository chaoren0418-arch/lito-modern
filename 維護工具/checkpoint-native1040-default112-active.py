"""Record a birth-verified broad cohort after exact-source cold C02 was sealed."""
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
assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-default112') and args.session_id > 0
cohort = read(run/'cohort.json')
job = dict(pid=cohort['pipeline_pid'],creation_utc=cohort['pipeline_creation_utc'])
assert cohort['status'] in ['executing_current_default112_profile','independently_reading_current_default112_profile']
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(job['pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
live = json.loads(subprocess.check_output(['pwsh', '-NoProfile', '-Command', command]).decode('utf-8-sig'))
assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds()) < .001
cold = ROOT/'驗證紀錄/current/native1040-cone-cold-cohort/20261018-074514-726757'
qualified = read(cold/'qualified/qualified.json')
assert qualified['source_manifest_sha256'] == cohort['source_manifest_sha256']
assert qualified['actual_sessions'] == 24 and qualified['original_exit_code'] == 0 and qualified['recorded_owned_all_terminal']
normal = Path(cohort['normal_qualification']).parent.parent
for name,digest in read(normal/'compiled-inputs.json').items():
    assert sha(ROOT/name) == digest,name
for name,digest in cohort['validation_pins'].items():
    assert sha(ROOT/'維護工具'/name) == digest,name
trackers = list(cold.glob('tracker-*/readback.json'))
assert len(trackers) == 1
before = read(trackers[0])['after_files']
assert len(before) == 8 and all(sha(ROOT/name) == digest for name, digest in before.items())
assert not read(ROOT/'工作佇列.json').get('active_workloads')
out = run/'active-checkpoint'
out.mkdir()
with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(ROOT/name, name)
with zipfile.ZipFile(out/'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in before.items())
record = dict(status='running_current1040_broad_default112_cohort', run=str(run), source_manifest_sha256=cohort['source_manifest_sha256'], original_session_id=args.session_id, pipeline=job, observed_live=live, qualification_granted=False, GUI_verified=False, physical_input_verified=False, physical_audio_verified=False, whole_G06_complete=False, whole_goal_complete=False)
save(out/'evidence.json', record)
work = '來源4E1F7A4E控制器、非同步Presentation及冰錐C02來源本體限定範圍已獨立核對。當版廣域原'+str(args.session_id)+'沿兩版112整合目標＋主程式執行；產品與對應QA凍結，須從同一session讀取實際終止並按每個測試映像、來源／名稱、raw及原程序身分獨立核對，不以候選或舊版本資格代替。廣域與完整library尚未授予資格；正常GUI／實體輸入音效、未播放舊音效跨端點拒絕、同端點與其他服務及完整G06／G01–G12仍待。35辨識／39缺口仍是靜態請求範圍；8／60秒、Release及G11預算不變，登入器／重建版獨立且相鄰只讀。'
for name in before:
    path = ROOT/name
    if path.suffix == '.json':
        value = read(path)
        assert value['current_product_identity']['source_manifest_sha256'] == cohort['source_manifest_sha256']
        value.update(current_native1040_broad_active=record, active_workloads={'broad_default':record}, active_workload_sessions={'broad_default':args.session_id}, source_and_corresponding_QA_frozen=True, next_task=work, next_focus=work, last_goal_turn_classification='progress', last_goal_turn_evidence=str(out/'evidence.json'), goal_complete=False, whole_goal_complete=False)
        value['current_focus'] = dict(item='current1040-broad-default112-continuation', work=work, applies_to_current_source=True, goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        save(path, value)
    else:
        first, separator, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert first.startswith('2026-10-18') and separator
        path.write_text('2026-10-18：'+work+'\n\n'+rest, encoding='utf-8', newline='\n')
save(out/'readback.json', dict(before_files=before, after_files={name:sha(ROOT/name) for name in before}, before_archive_sha256=sha(out/'before.zip'), evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name) == digest for name, digest in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json', dict(**record, active_checkpoint=str(out/'readback.json'), must_poll_original_session_before_restarting=True, next_action='Read original broad terminal, independently observe all profile test-image/raw/source/name results and recorded process identities, then checkpoint before full library. Preserve any failure; do not restart.'))
print(json.dumps(dict(checkpoint=str(out), original_session_id=args.session_id, live_pipeline=live), ensure_ascii=False))
