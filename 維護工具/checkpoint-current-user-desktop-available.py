"""Persist the user's fresh desktop-availability reply without granting a pass."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib
import json
import zipfile
R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
data = lambda value: (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
w = read(R / '工作佇列.json')
assert w['active_workload_sessions'] == {'broad_default': 41862} and w['source_and_corresponding_QA_frozen']
source = w['current_product_identity']['source_manifest_sha256']
assert source == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
out = V / 'user-desktop-availability' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
record = dict(status='desktop_available_confirmed_by_user_for_next_owned_GUI_automation',
              user_reply='現在桌面可用', request_id='call_7a299fdb2c4f47aabca021ce15523fb7', recorded_utc=datetime.now(timezone.utc).isoformat(),
              source_manifest_sha256=source, original_active_session=41862,
              next_action='Finish and independently close the original Debug/Release broad cohort first; then start a fresh owned normal GUI case and observe actual Sky state before each input. Agent performs synthetic loopback login and all steps.',
              actual_tool_recovery_observed=False, GUI_qualification_granted=False,
              old_failed_cases_and_cleanup_preserved=True, manual_user_login_or_acceptance_requested=False,
              original_workload_or_protocol_deadlines_changed=False, whole_goal_complete=False)
(out / 'evidence.json').write_bytes(data(record))
(out / 'checkpoint.py').write_bytes(Path(__file__).read_bytes())
work = '使用者已明確回覆「現在桌面可用」。先沿原41862完成當版廣域Release及獨立終止核對，再由代理新開正常GUI隔離批次、重新觀測Sky可用狀態與操作；不需使用者找視窗或登入。舊最小化／input guard失敗與清理結果保留，不等同工具已恢復或GUI通過。當版來源／對應QA固定，已通過範圍不重跑，#43候選及兩項效果資源仍未套用／未執行。'
staged = {}
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        value['current_user_desktop_availability'] = dict(**record, evidence=str(out / 'evidence.json'), evidence_sha256=sha(out / 'evidence.json'))
        value.setdefault('current_source_goal_validation', {}).setdefault('G08-player-gui', {}).update(state='user_desktop_available_fresh_GUI_after_original_broad_pending', complete=False)
        for item in value.get('items', []):
            if item.get('id') == 'G08-player-gui':
                item.update(state='user_desktop_available_fresh_GUI_after_original_broad_pending', complete=False)
        value.update(next_task=work, next_focus=work, last_goal_turn_evidence=str(out / 'evidence.json'), last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        staged[name] = data(value)
    else:
        first, sep, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert sep
        staged[name] = (datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d') + '：' + work + '\n\n' + rest).encode('utf-8')
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(R / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in before.items())
assert all(sha(R / name) == digest for name, digest in before.items())
for name, payload in staged.items():
    (R / name).write_bytes(payload)
(out / 'readback.json').write_bytes(data(dict(before_files=before, after_files={name: sha(R / name) for name in before}, before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out), user_desktop_available=True, GUI_qualified=False, original_session=41862), ensure_ascii=False))
