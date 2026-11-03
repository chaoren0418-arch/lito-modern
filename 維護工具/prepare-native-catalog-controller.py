"""Port the existing deep controller checks to the actual catalog-normal prerequisite."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具';V=ROOT/'驗證紀錄/current'
out=V/'native-catalog-controller'/('tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
mapping={
'run-native1047-G06-ability-finite-75DA.py':'run-native-catalog-controller.py',
'qualify-native1047-G06-controller-case-75DA.py':'qualify-native-catalog-controller-case.py',
'qualify-native1047-G06-controller-cohort.py':'qualify-native-catalog-controller-cohort.py',
'observe-native1047-G06-completion.py':'observe-native-catalog-controller-completion.py',
'checkpoint-native1047-G06-active.py':'checkpoint-native-catalog-controller-active.py'}
pins={}
def replace(s,old,new):
    assert s.count(old)==1,(old[:120],s.count(old));return s.replace(old,new)
for oldname,newname in mapping.items():
    original=(T/oldname).read_text('utf-8-sig');s=original
    for a,b in mapping.items():s=s.replace(a,b)
    s=s.replace('native1047-G06-controller','native-catalog-controller')
    if oldname.startswith('run-'):
        s=s.replace("choices=['ability-finite-normal'], default='ability-finite-normal'","choices=['catalog-normal'], default='catalog-normal'")
        start=s.index("assert startup.is_relative_to(ROOT/'驗證紀錄/current/native-ability-finite-normal')")
        end=s.index("extra = dict(path=",start)
        s=s[:start]+'''assert startup.is_relative_to(ROOT/'驗證紀錄/current/native-same-endpoint-catalog-normal')
c=read(startup/'cohort.json');assert c['status']=='executed_same_endpoint_catalog_normal_related_both_profiles' and c['owned_process'] is None
assert len(c['process_history'])==8 and all(v['exit_code']==0 for v in c['process_history'])
qualification=startup/'qualified/qualified.json';q=read(qualification)
assert q['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_all_terminal']
assert q['source_manifest_sha256']==c['source_manifest_sha256'] and set(q['profiles'])=={'Debug','Release'} and q['passed_per_profile']==16
assert not q['full_broad_regression_verified'] and not q['full_library_verified'] and not q['source_overlays_used']
application=Path(c['application']);identity_manifest=application/'applied-source-inputs.json';pins=read(identity_manifest)
assert len(pins)==1047 and sha(identity_manifest)==c['source_manifest_sha256'] and pins==read(startup/'source-inputs.json')
inputs=read(startup/'compiled-inputs.json');assert len(inputs)==1048
retained=dict(path=q['compiled_source_archive'],sha256=q['compiled_source_archive_sha256'],files=inputs)
assert sha(retained['path'])==retained['sha256']
with zipfile.ZipFile(retained['path']) as z:
    assert set(z.namelist())==set(inputs)
    for name,digest in inputs.items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==digest
normal=dict(profiles={profile:info['normal'] for profile,info in c['profiles'].items()})
for profile,info in q['profiles'].items():
    assert info['normal']==normal['profiles'][profile] and info['normal']['sha256']==q['normal_mains'][profile]
    assert set(info['targets'])=={'native_services','native_teleport'}
    assert {name:item['counts'] for name,item in info['targets'].items()}=={'native_services':{'passed':2,'failed':0,'ignored':0},'native_teleport':{'passed':14,'failed':0,'ignored':0}}
q['entry_sha256']=sha(ROOT/'啟動現代客戶端.ps1')
''' +s[end:]
        s=replace(s,"assert release==c['release_configuration']","assert release=={'opt-level':'z','lto':'thin','codegen-units':16,'strip':'symbols'}")
        s=replace(s,"compiled_source_archive=c['compiled_source_archive']","compiled_source_archive=retained")
    elif oldname.startswith('qualify-')and'cohort'in oldname:
        s=replace(s,"normal['profiles'][profile]['normal_sha256']","normal['profiles'][profile]['normal']['sha256']")
    elif oldname.startswith('observe-'):
        s=replace(s,"startup['profiles'][profile]['normal_sha256']","startup['profiles'][profile]['normal']['sha256']")
    elif oldname.startswith('checkpoint-'):
        s=replace(s,"completion['recorded_original_owned_processes_all_terminal']","completion['recorded_owned_processes_all_terminal']")
        s=replace(s,"assert all(v['passed']==31 for v in completion['profiles'].values())","assert completion['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and completion['passed_per_profile']==16")
        s=replace(s,"record['normal_ability_finite_related']=dict(passed_each=31,scene_cases=23,ability_components=7,finite_resource_case=1,finite_graphics=11,both_normal_profiles_qualified=True)","record['normal_catalog_related']=dict(passed_each=16,both_normal_profiles_qualified=True,current_source_broad_and_library_verified=False)")
        begin=s.index("work='來源'");end=s.index('\nfor n in before:',begin)
        s=s[:begin]+"work='來源'+c['source_manifest_sha256'][:8]+'正常兩版每版16相關測試及同端點租裝12sessions已限定核對。當版G06控制器六分支沿原'+str(a.session_id)+'執行，來源與對應QA固定；待原session實際終止及逐案讀回。舊33A3廣域／library不移記，Presentation音效、其他服務及正常GUI仍待；開發期間不請使用者登入，8秒／60秒／Release與G11預算不變，相鄰只讀且登入器產物獨立，完整G01–G12未完成。'"+s[end:]
        s=s.replace('running_current1047_G06','running_catalog_G06').replace('current_native1040_G06_active','current_native_catalog_G06_active').replace('current1047-G06','catalog-G06')
    ast.parse(s,filename=newname);assert not(T/newname).exists()
    (out/oldname).write_text(original,encoding='utf-8',newline='\n');(T/newname).write_text(s,encoding='utf-8',newline='\n')
    (out/(newname+'.diff')).write_text(''.join(difflib.unified_diff(original.splitlines(True),s.splitlines(True),fromfile=oldname,tofile=newname)),encoding='utf-8',newline='\n')
    pins[newname]=dict(original=oldname,original_sha256=hashlib.sha256(original.encode()).hexdigest().upper(),new_sha256=hashlib.sha256(s.encode()).hexdigest().upper())
(out/'preparation.json').write_text(json.dumps(dict(tools=pins,deep_case_packet_inventory_authority_checks_unchanged=True,normal_prerequisite='actual current-source catalog-normal16 both profiles; no broad/library inference',runtime_executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(pins),AST='passed',runtime_executed=False),ensure_ascii=False))
