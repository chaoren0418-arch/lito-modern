"""Stage normal-EXE hotel cases with the genuine current limited normal baseline."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
names=['native33A3_hotel_gate.py','run-native33A3-hotel.py','qualify-native33A3-hotel-rooms.py','qualify-native33A3-hotel-keys.py','qualify-native33A3-hotel-share.py','qualify-native33A3-hotel.py','checkpoint-native33A3-hotel.py']
mapping={n:n.replace('native33A3','native-catalog').replace('native-catalog_hotel','native_catalog_hotel')for n in names}
out=V/'native-catalog-hotel'/('tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);pins={}
def replace(s,a,b):assert s.count(a)==1,(a[:80],s.count(a));return s.replace(a,b)
for oldname,newname in mapping.items():
    original=(T/oldname).read_text('utf-8-sig');s=original
    for a,b in mapping.items():s=s.replace(a,b)
    s=s.replace('native33A3_hotel_gate','native_catalog_hotel_gate').replace('native33A3-hotel','native-catalog-hotel').replace('current33A3','current_catalog').replace('current_native33A3_hotel','current_native_catalog_hotel').replace('CURRENT33A3_HOTEL','CURRENT_CATALOG_HOTEL')
    s=s.replace('33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21','2DAF4DE08EF396B1DE5B2264820E689859BFCD837015D71A6D74643A2DDEE9DE').replace('當版33A3CCF7','當版2DAF4DE0')
    if oldname=='native33A3_hotel_gate.py':
        s=s.replace('import ctypes, hashlib, json, os, subprocess, time','import ctypes, hashlib, json, os, subprocess, time, zipfile')
        start=s.index('def library(path):');end=s.index('\ndef bind(',start)
        s=s[:start]+'''def baseline(path):
    normal=Path(path).resolve(strict=True);assert normal.is_relative_to(V/'native-same-endpoint-catalog-normal')
    n=read(normal/'cohort.json');qp=normal/'qualified/qualified.json';q=read(qp)
    assert n['status']=='executed_same_endpoint_catalog_normal_related_both_profiles' and n['owned_process'] is None and n['source_manifest_sha256']==SOURCE
    assert q['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and q['source_manifest_sha256']==SOURCE and q['passed_per_profile']==16
    assert q['observed_terminal_exit_code']==0 and q['recorded_owned_processes_all_terminal'] and not q['source_overlays_used']
    assert not q['full_broad_regression_verified'] and not q['full_library_verified']
    inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1048 and len(read(normal/'source-inputs.json'))==1047
    assert sha(normal/'source-inputs.json')==SOURCE and sha(normal/'cohort.json')==q['cohort_sha256']
    for profile in ['Debug','Release']:
        assert q['profiles'][profile]['normal']==n['profiles'][profile]['normal']
        assert sha(n['profiles'][profile]['normal']['path'])==q['normal_mains'][profile]
        assert set(q['profiles'][profile]['targets'])=={'native_services','native_teleport'}
        assert sum(x['counts']['passed'] for x in q['profiles'][profile]['targets'].values())==16
    assert sha(q['compiled_source_archive'])==q['compiled_source_archive_sha256']
    with zipfile.ZipFile(q['compiled_source_archive']) as z:
        assert set(z.namelist())==set(inputs)
        for name,digest in inputs.items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest
    for name,digest in inputs.items():assert sha(ROOT/name)==digest,name
    return normal,q,normal,n,inputs
''' +s[end:]
        s=replace(s,"_,_,normal,n,inputs=library(g['library_cohort'])","_,_,normal,n,inputs=baseline(g['normal_baseline'])")
        s=replace(s,"sha(Path(g['library_cohort'])/'completion-observation.json')==g['library_completion_sha256']","sha(Path(g['normal_baseline'])/'qualified/qualified.json')==g['normal_baseline_qualification_sha256'] and not g['full_current_library_qualified']")
    elif oldname=='run-native33A3-hotel.py':
        s=s.replace('Eight automated normal-EXE hotel cases after same-source broad/library completion.','Eight automated normal-EXE hotel cases after the current limited normal baseline.')
        s=s.replace("'--library-cohort'","'--normal-cohort'")
        s=replace(s,'base,lc,normal,n,inputs=library(a.library_cohort)','base,lc,normal,n,inputs=baseline(a.normal_cohort)')
        s=replace(s,"library_cohort=str(base),library_completion_sha256=sha(base/'completion-observation.json')","normal_baseline=str(base),normal_baseline_qualification_sha256=sha(base/'qualified/qualified.json'),full_current_library_qualified=False")
    elif oldname=='qualify-native33A3-hotel.py':
        s=replace(s,"library_cohort=g['library_cohort'],library_completion_sha256=g['library_completion_sha256']","normal_baseline=g['normal_baseline'],normal_baseline_qualification_sha256=g['normal_baseline_qualification_sha256'],full_current_library_qualified=False")
    ast.parse(s,filename=newname);assert not(T/newname).exists()
    (out/oldname).write_text(original,encoding='utf-8',newline='\n');(T/newname).write_text(s,encoding='utf-8',newline='\n')
    (out/(newname+'.diff')).write_text(''.join(difflib.unified_diff(original.splitlines(True),s.splitlines(True))),encoding='utf-8',newline='\n')
    pins[newname]=dict(original=oldname,original_sha256=hashlib.sha256(original.encode()).hexdigest().upper(),new_sha256=hashlib.sha256(s.encode()).hexdigest().upper())
(out/'preparation.json').write_text(json.dumps(dict(tools=pins,current_source_manifest_sha256='2DAF4DE08EF396B1DE5B2264820E689859BFCD837015D71A6D74643A2DDEE9DE',normal_related_prerequisite_each=16,full_current_library_inferred=False,deep_wire_item_terrain_and_lease_checks_unchanged=True,runtime_executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(pins),AST='passed',runtime_executed=False),ensure_ascii=False))
