"""Publish qualified normal/source-runtime scopes; keep broader acceptance pending."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950';observed=normal/'completion-observation.json';o=read(observed)
assert o['status']=='independently_qualified_class2_normal_both_profiles_and_terminal_processes' and o['recorded_original_owned_processes_all_terminal'] and o['observed_terminal_exit_code']==0
c=read(normal/'cohort.json');assert sha(normal/'cohort.json')==o['cohort_sha256']
app=Path(c['application']);application=read(app/'application.json');assert sha(app/'application.json')==c['application_sha256']
for name,digest in read(normal/'compiled-inputs.json').items():assert sha(ROOT/name)==digest
runtime=[]
for profile,name in [('Debug','20261018-020237-171821'),('Release','20261018-020437-413425')]:
    qp=next((ROOT/'驗證紀錄/current/native-class2-current-live'/name).glob('qualified-*/qualified.json'));q=read(qp)
    assert q['status']=='independently_qualified_current_normal_source_WORLD_and_same_map_retirement' and q['profile']==profile
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['recorded_owned_all_terminal']
    assert q['normal_product_registered'] and not q['source_overrides_used'] and not q['normal_GUI_product_exercised']
    assert q['normal_product_sha256']==o['profiles'][profile]['normal_sha256']
    assert q['actual_sessions']==6 and q['actual_S5A']==q['actual_C08']==4 and q['same_numeric_map_transitions']==2
    assert q['same_map_real109_retirement_verified'] and not q['same_map_active_class2_flash_verified']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in q['raw_files'].items())
    runtime.append(dict(profile=profile,qualification=str(qp),qualification_sha256=sha(qp),original_session=q['original_session_id'],terminal_chunk=q['terminal_tool_chunk'],
        product_sha256=q['product_sha256'],normal_product_sha256=q['normal_product_sha256'],actual_sessions=6,actual_S5A=4,same_numeric_map_transitions=2,recorded_owned_all_terminal=True))
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';assert sha(formal)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
before=read(app/'tracker-readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/name)==digest for name,digest in before.items())
out=ROOT/'驗證紀錄/current/native-class2-normal'/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in before:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='normal_both_profiles_57_related_and_current_source_same_map_runtime_qualified_limited',source_manifest_sha256=c['source_manifest_sha256'],source_files=1034,compiled_input_files=1035,
    application=str(app),application_sha256=sha(app/'application.json'),normal_completion=str(observed),normal_completion_sha256=sha(observed),normal_profiles=o['profiles'],normal_recorded_terminal_identities=o['recorded_original_owned_identities'],
    current_source_runtime=runtime,actual_current_source_sessions=12,actual_S5A=8,actual_C08=8,same_numeric_map_transitions=4,source_overrides_used=False,
    normal_product_registered=True,normal_GUI_product_exercised=False,same_map_real109_retirement_verified=True,same_map_active_class2_flash_verified=False,
    current_formal_JAR_sha256=sha(formal),formal_JAR_modified=False,normal_related_validation_complete=True,
    current_full_library_verified=False,current_broad_default_verified=False,current_skill34_played_audio_and_refusal_runtime_pending=True,
    class1_class2_shared_ambient_overlap_qualification_pending=True,pixel_brightness_verified=False,GUI_verified=False,physical_audio_verified=False,full_G06_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源86E65C7B已進正常產品，共1034來源／1035編譯輸入。唯一入口Debug FF1B6780與Release BCBE13E5、各7實際相關測試映像57過0敗0忽略已獨立核對，原99136 actual exit0及59記錄身分終止。當版正常來源無候選覆寫之兩版C02診斷另共12 LOGIN/GAME sessions、8次S5A／C08，其中4次map4→map4；舊輸入與34／109音效退役、新109與移動及角色／物品／HPMP對應通過。診斷未驅動正常GUI EXE，實體輸入音效未驗；同圖號活動class2亮度僅保留組件範圍。E159之default／library只留舊版本資格，當版廣域與完整library未執行。下一步當版34實際播放／魔力不足拒絕與G06六分支，接廣域／library及正常GUI；共用class1／class2亮度重疊與像素消費端仍待。完整G06及G01–G12未完成，8／60秒、G11與Release設定不變，登入器／正式JAR只讀且產物獨立。'
for name in before:
    path=ROOT/name
    if path.suffix=='.json':
        j=read(path);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==c['source_manifest_sha256']
        identity.update(source_qualification_pending=False,Debug_normal_sha256=o['profiles']['Debug']['normal_sha256'],Release_normal_sha256=o['profiles']['Release']['normal_sha256'],
            existing_normal_source_manifest_sha256=c['source_manifest_sha256'],Debug_normal_qualified_for_current_source=True,Release_normal_qualified_for_current_source=True,
            qualification_scope='normal builds, seven related targets and current-source scoped WORLD/same-map diagnostics; not GUI/full-library/full-G06')
        for key,value in j.items():
            if key.startswith('current_native') and isinstance(value,dict) and value.get('source_manifest_sha256') and value['source_manifest_sha256']!=c['source_manifest_sha256']:
                value.update(applies_to_current_source=False,retained_source_version_only=True)
        j['current_native_class2_application'].update(normal_validation_pending=False,qualification=str(out/'evidence.json'),qualification_sha256=sha(out/'evidence.json'))
        j.update(current_native_class2_normal=record,next_task=work,next_focus=work,source_and_corresponding_QA_frozen=True,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        for key in j['current_source_goal_validation']:
            j['current_source_goal_validation'][key]=dict(state='86E6_related_and_scoped_WORLD_qualified_broader_acceptance_pending',complete=False)
        for item in j.get('items',[]):
            if item['id'].startswith('G01-'):item.update(state='86E6_current_normal_source_baseline_verified_limited',complete=True)
            if item['id'].startswith('G03-'):item.update(state='86E6_seven_related_targets_qualified_broad_library_pending',complete=False)
        if name.endswith('next-validation-action.json'):j['next_action']=work
        else:j['current_focus']=dict(item='class2-current-source-audio-G06-and-broad-regression',work=work,applies_to_current_source=True,goal_complete=False)
        save(path,j)
    else:
        first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={name:sha(ROOT/name) for name in before},evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name)==digest for name,digest in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),source_manifest_sha256=c['source_manifest_sha256'],passed_each=57,current_source_sessions=12,whole_goal_complete=False),ensure_ascii=False))
