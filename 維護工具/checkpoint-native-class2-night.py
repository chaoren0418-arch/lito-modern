"""Publish bounded candidate evidence, preserving pixel and normal-product gaps."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]

def read(p):
    return json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('--release-refusal-run', type=Path, required=True)
a = ap.parse_args()
runs = [ROOT / '驗證紀錄/current/native-class2-live' / name for name in [
    '20261018-001311-469195', '20261018-001511-977453', '20261018-002437-199591']]
runs.append(a.release_refusal_run.resolve(strict=True))
assert runs[-1].is_relative_to(ROOT / '驗證紀錄/current/native-class2-live')
records, candidate_pins = [], None
for index, run in enumerate(runs):
    qpath = next(run.glob('qualified-*/qualified.json'))
    q, receipt = read(qpath), read(run / 'receipt.json')
    build = Path(receipt['compile'])
    b = read(build / 'receipt.json')
    assert q['status'] == 'independently_qualified_class2_C02_candidate_controller_presentation_scope'
    assert q['original_exit_code'] == 0 and q['recorded_owned_all_terminal']
    assert q['actual_sessions'] == 4 and q['controlled_visual_night_fixture']
    assert q['non15_ambient_lifecycle_verified'] == (index % 2 == 0)
    assert not q['non15_brightness_change_verified']
    assert not q['normal_product_registered'] and not q['GUI_verified']
    assert q['actual_accepted_casts'] == (2 if index % 2 == 0 else 0)
    assert q['actual_requests'] == (2 if index % 2 == 0 else 1)
    profile = 'Debug' if index < 2 else 'Release'
    assert b['profile'] == profile and not b['normal_product_build']
    assert sha(receipt['product']) == receipt['product_sha256'] == q['product_sha256']
    assert sha(build / 'receipt.json') == receipt['compile_receipt_sha256']
    pins = {name: entry['sha256'] for name, entry in b['candidate_inputs'].items()}
    if candidate_pins is None:
        candidate_pins = pins
    assert pins == candidate_pins
    for name, entry in b['candidate_inputs'].items():
        assert sha(entry['path']) == entry['sha256']
        assert sha(build / name) == b['snapshot_candidate_inputs'][name]
    for name, digest in read(build / 'compiled-inputs.json').items():
        assert sha(ROOT / name) == digest
    archive = qpath.parent / 'raw.zip'
    if not archive.is_file():
        archives = [p for p in qpath.parent.glob('*.zip') if sha(p) == q['raw_archive_sha256']]
        assert len(archives) == 1
        archive = archives[0]
    assert sha(archive) == q['raw_archive_sha256']
    with zipfile.ZipFile(archive) as z:
        assert all(hashlib.sha256(z.read(name)).hexdigest().upper() == digest for name, digest in q['raw_files'].items())
    records.append(dict(profile=profile, run=str(run), qualification=str(qpath), qualification_sha256=sha(qpath),
        original_session_id=q['original_session_id'], terminal_tool_chunk=q['terminal_tool_chunk'],
        product_sha256=q['product_sha256'], actual_sessions=q['actual_sessions'], actual_requests=q['actual_requests'],
        actual_accepted_casts=q['actual_accepted_casts'], night_callback_evidence=q['night_callback_evidence'],
        deferred_hurt_evidence=q['deferred_hurt_evidence'], recorded_owned_all_terminal=True))

binding = ROOT / '驗證紀錄/current/native-class2-light-consumer/20261018-002430-599983/binding.json'
assert read(binding)['status'] == 'primary_static_mask_binding_qualified_dynamic_ambient_pixel_consumer_unresolved'
previous = ROOT / '驗證紀錄/current/native1030-library-default/20261017-234321-712547/checkpoint-20261018-000557-972571/readback.json'
before = read(previous)['after_files']
assert len(before) == 8 and all(sha(ROOT / name) == digest for name, digest in before.items())
out = ROOT / '驗證紀錄/current/native-class2-night-checkpoint' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for name in before:
        z.write(ROOT / name, name)
with zipfile.ZipFile(out / 'before.zip') as z:
    assert all(hashlib.sha256(z.read(name)).hexdigest().upper() == digest for name, digest in before.items())
record = dict(status='both_profiles_class2_C02_candidate_authority_and_ambient_lifecycle_qualified_pixel_consumer_pending',
    candidate_pins=candidate_pins, cases=records, actual_sessions=16, actual_accepted_casts=4, actual_refused_requests=2,
    controlled_local_S21_replay=True, real_C02_night_clock=False,
    primary_light_binding=str(binding), primary_light_binding_sha256=sha(binding),
    failed_first_night_run=str(ROOT / '驗證紀錄/current/native-class2-live/20261018-000618-665700/failure-preserved.json'),
    hurt_receipt_fix='Retain already dispatched hurt callback metadata through environment-frame rejection; filter Model owner, scene generation and target incarnation before publication.',
    non15_ambient_lifecycle_verified=True, pixel_brightness_verified=False,
    original_dynamic_pixel_consumer_unresolved=True, real_same_map_S5A_during_flash_pending=True,
    normal_product_registered=False, normal_product_source_modified=False,
    GUI_verified=False, physical_audio_verified=False, full_G06_verified=False, whole_goal_complete=False)
save(out / 'evidence.json', record)
work = '來源E15907AC正常兩版與1031編譯輸入未改，default與完整library兩版既有資格維持。技能34相同11模組隔離候選已實測Debug／Release各成功與魔力不足案，共16 LOGIN/GAME sessions、4次成功各18MP與Sound34一次、2次拒絕；實際受擊回呼在環境影格捨棄後可正確交付且不重複。明示本地S21夜間重播驗得6→15→6與退役／來源身分preflight，並非C02夜間時鐘；map564仍走全亮遮罩，像素提亮未通過。原版固定地圖遮罩與動態亮度讀值分開保留，動態像素消費端待證，不臆改繪圖公式。原35752失敗封存不變。下一步實際傳圖中效果退役與目標移動生命週期；正常產品尚未註冊，GUI／實體輸入音效、同端點、完整G06及G01–G12未完成。8／60秒、G11與Release設定不變；登入器僅供只讀線索，不替換其獨立產物。'
for name in before:
    path = ROOT / name
    if path.suffix == '.json':
        value = read(path)
        assert value['current_product_identity']['source_manifest_sha256'] == 'E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182'
        value.update(current_native_class2_night_candidate=record, next_task=work, next_focus=work,
            last_goal_turn_classification='progress', last_goal_turn_evidence=str(out / 'evidence.json'),
            source_and_corresponding_QA_frozen=True, goal_complete=False, whole_goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        else:
            value['current_focus'] = dict(item='G04-canonical34-lifecycle-candidate', work=work,
                applies_to_current_source=False, normal_product_registered=False, goal_complete=False)
        save(path, value)
    else:
        first, separator, rest = path.read_text(encoding='utf-8-sig').partition('\n\n')
        assert first.startswith('2026-10-18') and separator
        path.write_text('2026-10-18：' + work + '\n\n' + rest, encoding='utf-8', newline='\n')
(out / 'checkpoint.py').write_bytes(Path(__file__).read_bytes())
after = {name: sha(ROOT / name) for name in before}
save(out / 'readback.json', dict(before_files=before, after_files=after, evidence_sha256=sha(out / 'evidence.json')))
assert all(sha(ROOT / name) == digest for name, digest in read(out / 'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out), cases=4, actual_sessions=16, normal_source_changed=False,
    pixel_brightness_verified=False, whole_goal_complete=False), ensure_ascii=False))
