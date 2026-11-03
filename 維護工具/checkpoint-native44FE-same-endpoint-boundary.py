"""Publish the failed boundary inside the rebuild project; preserve previous metadata."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]
V = ROOT / '驗證紀錄/current'

def read(p):
    return json.loads(Path(p).read_text('utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    Path(p).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

run = V / 'native44FE-same-endpoint/20261018-171204-385221-debug'
boundary = run / 'independent-boundary/boundary.json'
q = read(boundary)
assert q['status'] == 'independently_preserved_current_same_endpoint_channel_contract_failure'
assert q['actual_client_sessions'] == 6 and q['recorded_owned_all_terminal']
assert not q['same_endpoint_verified'] and not q['launcher_artifact_modified']
assert sha(boundary.parent / 'raw.zip') == q['raw_archive_sha256']
prior = V / 'native44FE-audio-cohort/20261018-165317-090522/checkpoint-completed-20261018-171116-151396/readback.json'
before = read(prior)['after_files']
assert len(before) == 8 and all(sha(ROOT / n) == h for n, h in before.items())
queue = read(ROOT / '工作佇列.json')
assert not queue['active_workloads'] and queue['current_product_identity']['source_manifest_sha256'] == q['source_manifest_sha256']
proposal = ROOT / '修補提案/同端點傳送/README.md'
text = proposal.read_text('utf-8-sig')
old = '以下 6928A147／B33F9FA1 記錄是舊版本的隔離歷史對照；現行 C02 的同端點尚未成功，見[當前邊界](../../驗證紀錄/current/native-status-scroll/same-endpoint-revalidation/boundary.json)。這份資料不再提出任何登入器替換或部署要求。'
assert text.count(old) == 1
out = run / ('checkpoint-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
archive_files = dict(before)
archive_files[proposal.relative_to(ROOT).as_posix()] = sha(proposal)
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for n in archive_files:
        z.write(ROOT / n, n)
with zipfile.ZipFile(out / 'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper() == h for n, h in archive_files.items())
record = {k: q[k] for k in ['status', 'source_manifest_sha256', 'reference_jar_sha256', 'profile',
    'original_session_id', 'original_observation_wrapper_exit_code', 'terminal_tool_chunk',
    'actual_PowerShell_exit_code', 'actual_Rust_exit_code', 'actual_server_exit_code',
    'actual_client_sessions', 'transferred_markers', 'final_handoff_reports',
    'recorded_owned_all_terminal', 'same_endpoint_verified', 'gameplay_qualification_granted']}
record.update(boundary=str(boundary), boundary_sha256=sha(boundary),
    launcher_artifact_modified=False, launcher_replacement_proposal_withdrawn=True,
    raw_archive_sha256=q['raw_archive_sha256'], whole_goal_complete=False)
save(out / 'evidence.json', record)
work = '當版44FEA738／75DA69B0同端點Debug實測未通過：原92759觀察wrapper0，實際PowerShell1／Rust101／服務0；6條sessions，兩端通過新GAME接續與舊輸入退役檢查，隨後C_CA 00皆被參考核心當租裝選擇回C9，缺後續S_CA而逾時。4組記錄程序身分均終止，隔離狀態已回收；不授予同端點成功。兩版正常23場景、控制器12案96sessions及正常模組音效12案96sessions維持各自資格。登入器替換提案已撤回，產物各自獨立；接續重建版內當版旅館、普通WORLD／其他服務、必要廣域／完整library與正常可見性。GUI／實體輸入音效、有效不同資料集ABA及完整G06／G01–G12仍待，既有時限與G11預算不變。'
date = (datetime.now(timezone.utc) + timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p = ROOT / n
    if p.suffix == '.json':
        x = read(p)
        assert x['current_product_identity']['source_manifest_sha256'] == q['source_manifest_sha256']
        x['current_same_endpoint_boundary'] = record
        x.update(next_task=work, next_focus=work, last_goal_turn_classification='progress',
                 last_goal_turn_evidence=str(out / 'evidence.json'), goal_complete=False, whole_goal_complete=False)
        if 'updated_date' in x:
            x['updated_date'] = date
        x['current_focus'] = dict(item='current-G06-hotel-WORLD-remaining-scope', work=work, applies_to_current_source=True, goal_complete=False)
        for item in x.get('items', []):
            if item.get('id', '').startswith('G06-'):
                item['current_same_endpoint_boundary'] = record
                for step in item.get('rental_acceptance_steps', []):
                    if step.get('id') == 'G06.3d':
                        step['current_same_endpoint_boundary'] = record
                        step['remaining'] = '同端點已獨立保存當版頻道契約失敗，未通過；當版旅館、普通WORLD／其他服務、正常GUI／實體輸入音效、有效不同資源資料集ABA及完整G06.3d仍待。'
        if n.endswith('next-validation-action.json'):
            x['next_action'] = work
        save(p, x)
    else:
        first, sep, rest = p.read_text('utf-8-sig').partition('\n\n')
        assert datetime.strptime(first[:10], '%Y-%m-%d') and sep
        p.write_text(date + '：' + work + '\n\n' + rest, encoding='utf-8', newline='\n')
new = '以下 6928A147／B33F9FA1 記錄僅為舊版本隔離歷史對照。當版重建來源44FEA738／唯讀參考75DA69B0的Debug同端點實測亦未通過：新GAME接續後，頻道請求收到租裝選擇回覆，缺後續頻道人數資料而逾時。見[當版失敗邊界](../../驗證紀錄/current/native44FE-same-endpoint/20261018-171204-385221-debug/independent-boundary/boundary.json)。舊C02邊界仍保留原版本資格；本資料不提出登入器替換或部署要求。'
proposal.write_text(text.replace(old, new), encoding='utf-8', newline='\n')
after = {n: sha(ROOT / n) for n in archive_files}
save(out / 'readback.json', dict(before_files=archive_files, after_files=after,
    canonical_after_files={n: after[n] for n in before}, before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json')))
assert all(sha(ROOT / n) == h for n, h in read(out / 'readback.json')['after_files'].items())
save(run.parent / 'current-continuation.json', dict(status='current_same_endpoint_failure_preserved_other_branches_pending',
    run=str(run), boundary=str(boundary), boundary_sha256=sha(boundary), latest_checkpoint=str(out), latest_readback=str(out / 'readback.json'),
    original_session_id=92759, original_terminal_exit_code=0, terminal_tool_chunk='180a91',
    must_poll_original_session_before_restarting=False, same_endpoint_verified=False, whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out), same_endpoint_verified=False, adjacent_artifacts_modified=False), ensure_ascii=False))
