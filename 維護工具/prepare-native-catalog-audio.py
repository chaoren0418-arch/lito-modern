"""Prepare unchanged audio assertions against the actual current normal body."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';P=R/'LiTo381-Modern'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def replace(s,a,b):assert s.count(a)==1,(a[:80],s.count(a));return s.replace(a,b)
normal=V/'native-same-endpoint-catalog-normal/validation-20261018-232351-306737';q=read(normal/'qualified/qualified.json');source=q['source_manifest_sha256'];assert source==read(R/'工作佇列.json')['current_product_identity']['source_manifest_sha256']
old=T/'candidates/native33A3-teleport-audio';prior=read(old/'preparation.json');candidate=T/'candidates/native-catalog-teleport-audio';candidate.mkdir()
for n,h in prior['files'].items():assert sha(old/n)==h;(candidate/n).write_bytes((old/n).read_bytes())
save(candidate/'preparation.json',dict(status='staged_current_catalog_normal_body_audio_diagnostics_not_compiled',source_manifest_sha256=source,normal_cohort=str(normal),files=prior['files'],prior_preparation=str(old/'preparation.json'),prior_preparation_sha256=sha(old/'preparation.json'),current_normal_login_sha256=sha(P/'src/modern_lito/native_login.rs'),current_body_differs_from_prior33A3=True,normal_source_bodies_unchanged=True,source_overlays_used=False,diagnostic_request_API_extension=True,normal_product_source_modified=False,normal_GUI_action_request_permission_inferred=False,full_current_library_qualified=False,compiled=False,executed=False,new_sessions=0,physical_audio_verified=False,GUI_verified=False,whole_goal_complete=False))
names=['compile-native33A3-teleport-audio.py','observe-native33A3-teleport-audio-compile.py','run-native33A3-teleport-audio.py','qualify-native33A3-teleport-audio.py','checkpoint-native33A3-audio-stage.py','run-native33A3-audio-cohort.py','qualify-native33A3-audio-cohort.py','checkpoint-native33A3-audio-cohort-running.py']
mapping={n:n.replace('native33A3','native-catalog')for n in names};out=V/'native-catalog-teleport-audio'/('tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);pins={}
for oldname,newname in mapping.items():
    original=(T/oldname).read_text('utf-8-sig');s=original
    for a,b in mapping.items():s=s.replace(a,b)
    s=s.replace('native33A3-teleport-audio','native-catalog-teleport-audio').replace('native33A3-audio-cohort','native-catalog-audio-cohort').replace('native1047-G06-controller','native-catalog-controller').replace('current_native33A3_audio','current_native_catalog_audio')
    if oldname.startswith('compile-'):
        s=replace(s,"ap.add_argument('--library-cohort',type=Path,required=True);",'')
        start=s.index("normal=Path(gc['startup_prerequisite']);");end=s.index("candidate=T/'candidates/",start)
        s=s[:start]+'''normal=Path(gc['startup_prerequisite']);assert normal.is_relative_to(V/'native-same-endpoint-catalog-normal')
c=read(normal/'cohort.json');inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1048 and c['source_files']==1047
assert c['source_manifest_sha256']==gc['source_manifest_sha256']==read(ROOT/'工作佇列.json')['current_product_identity']['source_manifest_sha256']
qp=normal/'qualified/qualified.json';q=read(qp)
assert q['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and q['passed_per_profile']==16 and q['source_manifest_sha256']==c['source_manifest_sha256']
assert q['recorded_owned_processes_all_terminal'] and not q['full_library_verified'] and not q['source_overlays_used']
library=None
''' +s[end:]
        s=replace(s,"library_cohort=str(library),library_completion_sha256=sha(library/'completion-observation.json')","library_cohort=None,library_completion_sha256=None,full_current_library_qualified=False")
        s=s.replace('metadata=normal33A3_wave_diagnostic','metadata=normal_catalog_wave_diagnostic')
    elif oldname=='qualify-native33A3-teleport-audio.py':
        s=replace(s,"=='33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21'","==read(ROOT/'工作佇列.json')['current_product_identity']['source_manifest_sha256']")
        start=s.index("nq=read(b['normal_qualification']);");end=s.index("normal=read(Path(b['normal_cohort'])",start)
        s=s[:start]+'''nq=read(b['normal_qualification']);assert sha(b['normal_qualification'])==b['normal_qualification_sha256'] and nq['source_manifest_sha256']==r['source_manifest_sha256']
assert nq['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and nq['passed_per_profile']==16 and nq['recorded_owned_processes_all_terminal']
assert set(nq['profiles'])=={'Debug','Release'} and all(set(v['targets'])=={'native_services','native_teleport'} for v in nq['profiles'].values())
assert not b['full_current_library_qualified'] and b['library_cohort'] is None
''' +s[end:]
    elif oldname=='checkpoint-native33A3-audio-cohort-running.py':
        start=s.index("work=f'來源33A3CCF7");end=s.index('\n',start)
        s=s[:start]+"work=f'當前來源正常本體音效兩版六分支沿原{a.session}執行，保留已限定核對Debug拒絕8sessions不重跑；四探針先編譯完成，期間不另編譯，來源與QA固定。只用隔離C1F API及診斷Player，正常GUI權限／實體音效不推定。正常兩版各16相關測試、同端點租裝12及控制器96sessions各維持限定資格，舊33A3廣域／library不移記。8秒／60秒／Release及G11預算不變，開發期間不請使用者登入，完整G06及G01–G12仍待。'"+s[end:]
    ast.parse(s,filename=newname);assert not(T/newname).exists()
    (out/oldname).write_text(original,encoding='utf-8',newline='\n');(T/newname).write_text(s,encoding='utf-8',newline='\n')
    (out/(newname+'.diff')).write_text(''.join(difflib.unified_diff(original.splitlines(True),s.splitlines(True),fromfile=oldname,tofile=newname)),encoding='utf-8',newline='\n')
    pins[newname]=dict(original=oldname,original_sha256=sha(out/oldname),new_sha256=sha(T/newname))
save(out/'preparation.json',dict(source_manifest_sha256=source,tools=pins,candidate=str(candidate),probe_bytes_identical_to_prior=True,normal_body_will_be_snapshotted_from_current_source=True,deep_audio_wire_cue_inventory_pixel_checks_unchanged=True,normal_related_prerequisite_per_profile=16,controller_sessions_prerequisite=96,current_broad_and_library_qualification_not_inferred=True,runtime_executed=False))
print(json.dumps(dict(tooling=str(out),tools=len(pins),AST='passed',runtime_executed=False),ensure_ascii=False))
