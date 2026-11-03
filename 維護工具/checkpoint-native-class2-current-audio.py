"""Record four scoped current-source cases without promoting GUI/audio acceptance."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, zipfile

ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'驗證紀錄/current/native-class2-current-audio'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')

active=read(BASE/'active-validation.json');cases=active['completed_cases']
assert len(cases)==4 and active['current_run'] is None
assert {(c['profile'],c['case']) for c in cases}=={(p,c) for p in ['Debug','Release'] for c in ['positive','refused']}
source=active['source_manifest_sha256'];evidence=[]
for case in cases:
    qp=Path(case['qualification']);q=read(qp);assert sha(qp)==case['sha256']
    assert q['status']=='independently_qualified_current_normal_source_class2_audio_admission_and_refusal'
    assert q['source_manifest_sha256']==source and q['normal_build_profile']==case['profile']
    positive=case['case']=='positive'
    assert q['profile']==('disintegrate' if positive else 'disintegrate-refused')
    assert q['actual_sessions']==4 and q['actual_requests']==(2 if positive else 1) and q['actual_accepted_casts']==(2 if positive else 0)
    assert set(q['sound34_fake_sink_counts'])=={'a','b'} and all(v==(2 if positive else 0) for v in q['sound34_fake_sink_counts'].values())
    assert q['recorded_owned_all_terminal'] and q['original_exit_code']==0 and q['normal_product_registered']
    assert not q['source_overrides_used'] and not q['normal_GUI_product_exercised'] and not q['physical_audio_verified']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:
        assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in q['raw_files'].items())
    build=Path(read(Path(q['run'])/'receipt.json')['compile'])
    for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h
    evidence.append(dict(case,actual_sessions=4,requests=q['actual_requests'],accepted_casts=q['actual_accepted_casts'],sound34_fake_sink_counts=q['sound34_fake_sink_counts']))

prior=ROOT/'驗證紀錄/current/native-class2-normal/metadata-reconciliation-20261018-021735-257996/final-observation.json'
before=read(prior)['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=BASE/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
    z.write(BASE/'active-validation.json','active-validation.json')
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='four_current_normal_source_class2_audio_admission_refusal_cases_qualified_limited',source_manifest_sha256=source,
    cases=evidence,actual_sessions=16,requests=6,accepted_casts=4,MP_refused_requests=2,
    diagnostic_images_visually_observed=active['diagnostic_images_visually_observed'],
    normal_product_registered=True,source_overrides_used=False,normal_GUI_product_exercised=False,
    audio_delivery_observation='production delivery through diagnostic fake sink; no physical device',
    controlled_visual_night_fixture=True,real_C02_night_verified=False,pixel_brightness_verified=False,
    physical_audio_verified=False,GUI_verified=False,full_G06_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源86E65C7B正常Debug／Release各57相關測試及當版傳圖診斷維持原範圍。新增當版正常來源極道落雷兩版成功／MP不足四案，共16 LOGIN/GAME sessions、6次C14；4次成功各扣MP18、11736特效與sound34每次僅一次，2次拒絕278無特效或音效。來源無候選覆寫、程序皆終止及raw封存已獨立核對；音效只經診斷fake sink，受控夜間S21非C02實收，正常GUI／實體音效／像素亮度未驗。下一步當版G06六分支，再廣域／完整library與正常GUI；E159歷史結果不移記當版，共用class1／class2亮度重疊與像素消費端仍待。完整G06與G01–G12未完成，8／60秒、G11與Release設定不變。重建版與登入器產物獨立，相鄰來源只讀，替換登入器JAR不是開發或驗收依賴。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        j=read(p);assert j['current_product_identity']['source_manifest_sha256']==source
        j['current_native_class2_current_audio']=dict(record,evidence=str(out/'evidence.json'),evidence_sha256=sha(out/'evidence.json'))
        j['current_native_class2_normal']['current_skill34_played_audio_and_refusal_runtime_pending']=False
        j['current_native_class2_normal']['current_skill34_audio_qualification_scope']='four scoped direct normal-source diagnostic cases, fake sink only; no normal GUI or physical device'
        j.update(next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),whole_goal_complete=False,goal_complete=False)
        j['current_focus']=dict(item='current-source-G06-six-branches',work=work,applies_to_current_source=True,goal_complete=False)
        goal=j['current_source_goal_validation'].get('G09-audio')
        if goal is not None:goal.update(state='86E6_four_class2_fake_sink_cases_qualified_physical_audio_pending',complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(p,j)
    else:
        first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
active.update(status='completed_four_cases_qualified_limited',checkpoint=str(out),whole_goal_complete=False);save(BASE/'active-validation.json',active)
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},evidence_sha256=sha(out/'evidence.json'),active_validation_sha256=sha(BASE/'active-validation.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),actual_sessions=16,accepted_casts=4,MP_refused_requests=2,whole_goal_complete=False),ensure_ascii=False))
