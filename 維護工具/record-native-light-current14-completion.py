"""Observe completed current14 evidence without rewriting runtime receipts."""
import datetime
import hashlib
import json
import pathlib
import subprocess
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
COHORT = ROOT / '驗證紀錄/current/native-teleport-current/20261015-232637-257-TW-shared'
STAGE = ROOT / '驗證紀錄/current/native-light-spell/current-regression'

def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))

def sha(path):
    return hashlib.file_digest(path.open('rb'), 'sha256').hexdigest().upper()

def write_new(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write('\n')

q = read(COHORT / 'qualified.json')
follow = read(STAGE / 'current14-followup.json')
normal_dir = ROOT / '驗證紀錄/current/native-light-spell/normal'
normal = read(normal_dir / 'qualified.json')
assert follow['status'] == 'qualified_current_explicit_TW14_GUI_cases'
assert q['scenarios'] == 14 and q['actual_formal_sessions'] == 112
assert q['extra_C_MOVE'] == 0 and q['raw_archive_readback_verified']
assert not q['whole_goal_complete']
inputs = read(normal_dir / 'source-inputs.json')
assert len(inputs) == 970
assert sha(normal_dir / 'source-inputs.json') == q['source_manifest_sha256']
for name, expected in inputs.items():
    assert sha(ROOT / name) == expected, name
for profile, product in normal['profiles'].items():
    assert sha(pathlib.Path(product['path'])) == product['sha256']
    assert q['profiles'][profile]['main_sha256'] == product['sha256']
archive = COHORT / 'GUI-raw-evidence.zip'
assert sha(archive) == q['raw_archive_sha256']
with zipfile.ZipFile(archive) as z:
    assert len(z.namelist()) == len(q['raw_archive_files']) == 834
    assert z.testzip() is None
    for name, expected in q['raw_archive_files'].items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected, name

# Observe actual command lines; never terminate unrelated processes.
ps = subprocess.run(['powershell.exe', '-NoProfile', '-NonInteractive', '-Command',
    '[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); Get-CimInstance Win32_Process | Select-Object ProcessId,Name,ExecutablePath,CommandLine | ConvertTo-Json -Depth 3 -Compress'],
    stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
processes = json.loads(ps.stdout.decode('utf-8-sig'))
root_name = str(ROOT).casefold()
live = [p for p in processes if
        p['ProcessId'] == 52496 or
        ((p.get('ExecutablePath') or '').casefold().startswith(root_name) and
         (p['Name'].casefold().startswith(('litomodern', 'qualified-client', 'native_')) or '\\deps\\' in (p.get('ExecutablePath') or ''))) or
        (p['Name'].casefold() in ('java.exe', 'cargo.exe', 'rustc.exe') and root_name in (p.get('CommandLine') or '').casefold())]
assert not live, live
now = datetime.datetime.now(datetime.timezone.utc).isoformat()
relq = (COHORT / 'qualified.json').relative_to(ROOT).as_posix()
marker = {
    'status': 'completed_pipeline_and_qualified14_observed_no_owned_GUI_or_test_processes',
    'observed_utc': now, 'completed_session': 57274,
    'terminal_exit_code': None,
    'terminal_exit_code_note': 'The tool session is already closed; this observation uses the final pipeline receipt, whole-cohort qualifier, hashes and absence of owned processes, without inventing a recovered exit code.',
    'qualification': relq, 'qualification_sha256': sha(COHORT / 'qualified.json'),
    'pipeline_sha256': sha(STAGE / 'current14-followup.json'),
    'source_files_rechecked': 970, 'source_manifest_sha256': q['source_manifest_sha256'],
    'normal_mains': {k: v['sha256'] for k, v in normal['profiles'].items()},
    'raw_archive_sha256': q['raw_archive_sha256'], 'raw_archive_files_readback': 834,
    'scenarios': 14, 'actual_formal_sessions': 112,
    'owned_GUI_or_test_processes_observed': live,
    'runtime_receipts_rewritten': False, 'actual_new_sessions_executed_by_marker': 0,
    'whole_goal_complete': False,
}
marker_path = STAGE / 'current14-completion-observation.json'
write_new(marker_path, marker)
trackers = [ROOT / '工作佇列.json', ROOT / '監製進度.json',
            ROOT / '驗證紀錄/current/native-teleport-wire/progress.json',
            ROOT / '驗證紀錄/current/native-terrain-residence/continuation.json']
before_path = STAGE / 'current14-tracker-before.zip'
with zipfile.ZipFile(before_path, 'x', compression=zipfile.ZIP_DEFLATED) as z:
    for p in trackers:
        z.write(p, p.relative_to(ROOT).as_posix())
with zipfile.ZipFile(before_path) as z:
    assert z.testzip() is None
next_work = '現版912D/FB94、970來源的預設91整合＋main與TW+extended十四案已各自完成資格及封存讀回；勿重啟38055、28420、57274。產品及對應GUI QA凍結已在完成與無自有程序後解除。繼續重建版內的原source.lmp照明合成實作與原片段比對，再處理G06.3d其餘服務／同端點參考缺口及G01–G12其餘玩法、輸入音效與交付；登入器與其產物唯讀，不作部署依賴。'

def refresh_nested(value):
    if isinstance(value, dict):
        if value.get('id') == 'G06.3d':
            value.update(current_GUI_evidence=relq, current_GUI_defined_scope_verified=True,
                current_TW_GUI_defined_scope_verified=True, last_qualified_TW14_evidence=relq,
                current_Debug_TW7_evidence=(COHORT / 'Debug-current-cohort.json').relative_to(ROOT).as_posix(),
                current_source_revalidation_note='當前912D/FB94、970來源的TW+extended兩版十四案已獨立驗證112 sessions、220移動零多送、80原場景PNG、56 glyph、128舊通知及350物件／wire檢查，834 raw檔讀回。僅192DPI私有桌面訊息；G06.3d其餘分支與整體未完成。')
        if value.get('id') == 'G03-current-regression':
            value['state'] = 'current_default91_light_five_GUI_and_TW14_defined_scopes_verified_broader_TCP_library_pending'
            if relq not in value['evidence']:
                value['evidence'].append(relq)
        for child in value.values():
            refresh_nested(child)
    elif isinstance(value, list):
        for child in value:
            refresh_nested(child)

for p in trackers:
    t = read(p)
    gui = t['current_light_TW14_GUI']
    gui.update(status=q['status'], qualified_cases=14, qualified_actual_sessions=112,
        evidence=relq, completion_observation=marker_path.relative_to(ROOT).as_posix(),
        whole_cohort_qualified=True, source_and_corresponding_GUI_QA_must_remain_frozen=False,
        session=None, completed_session=57274, pid=None, active_profile=None, active_scenario=None,
        last_observed_utc=now, Release_batch_verified=True,
        Release_completed_scenarios=[c['scenario'] for c in q['profiles']['Release']['cases']])
    t['current_light_default91']['source_must_remain_frozen'] = False
    t['current_light_TW14_dispatcher']['source_and_corresponding_GUI_QA_must_remain_frozen'] = False
    t['current_light_TW14_dispatcher']['pid'] = None
    t['next_task'] = next_work
    t['last_goal_turn_classification'] = 'progress'
    if 'updated_date' in t:
        t['updated_date'] = '2026-10-16'
    refresh_nested(t)
    p.write_text(json.dumps(t, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

old = '現版TW+extended傳送／恢復十四案已啟動，尚未全數驗證；完整G01–G12未完成。'
new = '同版TW+extended傳送／恢復十四案已全部獨立核對：112實際sessions、220移動零多送、80原世界PNG、56 glyph、128舊通知及350物件／wire檢查；834份raw ZIP讀回。僅192DPI私有桌面訊息操作，G06.3d其餘分支及完整G01–G12未完成。'
for p in (ROOT / 'README.md', ROOT / 'docs/ARCHITECTURE.zh-TW.md'):
    s = p.read_text(encoding='utf-8-sig')
    assert old in s
    p.write_text(s.replace(old, new, 1), encoding='utf-8')
history = ROOT / 'docs/HISTORY.md'
with history.open('a', encoding='utf-8') as f:
    f.write('\n\n2026-10-16：現版912D/FB94、970來源之TW+extended十四案收尾完成，112實際sessions（56LOGIN／56GAME）、220移動零多送、80原世界PNG、56 glyph、128舊通知及350物件／wire檢查；834原檔ZIP SHA '+q['raw_archive_sha256']+'讀回。完成觀察另核對970來源、兩正常主與ZIP每檔，未改寫runtime收據；57274工具session已關閉，外部父程序與自有GUI／Java／測試已不在，未補造不可恢復的exit code。四份tracker與G06.3d限定證據更新並解除本輪凍結；其他服務／同端點／實體輸入及全G01–G12仍待。重建版與登入器維持獨立，無外部產物替換。\n')
print(json.dumps({'status': marker['status'], 'qualification': relq, 'source_files': 970,
    'raw_files': 834, 'archive_sha256': q['raw_archive_sha256'], 'trackers_updated': 4}, ensure_ascii=False))
