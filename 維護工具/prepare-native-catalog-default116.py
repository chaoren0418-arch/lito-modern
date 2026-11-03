"""Port the unchanged default graph/name/archive checks to the current normal prerequisite."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
names=['run-native33A3-default116-both.py','run-native33A3-default116.py','qualify-native33A3-default116.py','native33A3_broad_gate.py','observe-native33A3-default116-completion-counted.py','checkpoint-native33A3-default116-active.py']
mapping={n:n.replace('native33A3','native-catalog').replace('native-catalog_broad','native_catalog_broad').replace('-completion-counted.py','-completion.py')for n in names}
out=V/'native-catalog-default116'/('tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);pins={}
def rep(s,a,b):assert s.count(a)==1,(a[:100],s.count(a));return s.replace(a,b)
for oldname,newname in mapping.items():
    original=(T/oldname).read_text('utf-8-sig');s=original
    for a,b in mapping.items():s=s.replace(a,b)
    s=s.replace('native33A3-default116','native-catalog-default116').replace('native33A3_broad_gate','native_catalog_broad_gate').replace('observe-native33A3-default116-completion.py','observe-native-catalog-default116-completion.py').replace('native1047-G06-controller','native-catalog-controller').replace('current_native33A3_broad','current_native_catalog_broad').replace('current33A3_broad','current_catalog_broad')
    s=s.replace('33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21','2DAF4DE08EF396B1DE5B2264820E689859BFCD837015D71A6D74643A2DDEE9DE')
    s=s.replace("choices=['ability-finite-normal'], default='ability-finite-normal'","choices=['catalog-normal'], default='catalog-normal'")
    if oldname=='run-native33A3-default116-both.py':
        start=s.index("startup=a.normal_run.resolve(strict=True);");end=s.index("g06=a.G06_qualification.resolve",start)
        s=s[:start]+'''queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
startup=a.normal_run.resolve(strict=True);assert startup.is_relative_to(ROOT/'驗證紀錄/current/native-same-endpoint-catalog-normal')
c=read(startup/'cohort.json');assert c['status']=='executed_same_endpoint_catalog_normal_related_both_profiles' and c['owned_process'] is None
normal=startup/'qualified/qualified.json';q=read(normal);source=q['source_manifest_sha256'];assert source==queue['current_product_identity']['source_manifest_sha256']
assert q['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and q['passed_per_profile']==16 and q['recorded_owned_processes_all_terminal']
assert q['observed_terminal_exit_code']==0 and not q['source_overlays_used'] and not q['full_library_verified']
inputs=read(startup/'compiled-inputs.json');assert len(inputs)==1048 and len(read(Path(c['application'])/'applied-source-inputs.json'))==1047
for profile in ['Debug','Release']:
    assert q['profiles'][profile]['normal']==c['profiles'][profile]['normal'] and q['normal_mains'][profile]==sha(c['profiles'][profile]['normal']['path'])

''' +s[end:]
        start=s.index("    for profile in ['Debug','Release']:\n        original=read(next(startup.glob");end=s.index('\nenv=dict(',start)
        s=s[:start]+'''    assert sha(normal)==state['normal_qualification_sha256']
    for profile in ['Debug','Release']:
        assert sha(c['profiles'][profile]['normal']['path'])==q['normal_mains'][profile]
''' +s[end:]
        s=rep(s,"selected_normal=next(startup.glob(profile+'-qualified-*/qualified.json'))","selected_normal=normal")
    elif oldname=='run-native33A3-default116.py':
        start=s.index("assert normal_path.is_relative_to(ROOT/'驗證紀錄/current/native-ability-finite-normal')");end=s.index('completion_path=args.G06_qualification',start)
        s=s[:start]+'''assert normal_path.is_relative_to(ROOT/'驗證紀錄/current/native-same-endpoint-catalog-normal')
assert normal['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and normal['passed_per_profile']==16 and normal['recorded_owned_processes_all_terminal']
assert normal['observed_terminal_exit_code']==0 and not normal['source_overlays_used'] and not normal['full_library_verified']
startup=normal_path.parent.parent;startup_receipt=read(startup/'cohort.json');assert startup_receipt['owned_process'] is None
identity=Path(startup_receipt['application'])/'applied-source-inputs.json';assert sha(identity)==normal['source_manifest_sha256']==startup_receipt['source_manifest_sha256']
pins=read(identity);assert len(pins)==normal['source_files']==1047 and pins==read(startup/'source-inputs.json')
normal['entry_sha256']=sha(ROOT/'啟動現代客戶端.ps1')
normal_products={selected:info['normal'] for selected,info in startup_receipt['profiles'].items()}
for selected,product in normal_products.items():
    assert product==normal['profiles'][selected]['normal'] and sha(product['path'])==normal['normal_mains'][selected]
''' +s[end:]
        s=rep(s,"startup_receipt['compiled_source_archive']['files']['LiTo381-Modern/windows/app.manifest']","all_inputs['LiTo381-Modern/windows/app.manifest']")
    elif oldname=='qualify-native33A3-default116.py':
        start=s.index("assert normal_kind=='ability-finite-normal'");end=s.index("extra=dict(path=",start)
        s=s[:start]+'''assert normal_kind=='catalog-normal'
startup=Path(r['normal_qualification']).parent.parent;assert startup.is_relative_to(ROOT/'驗證紀錄/current/native-same-endpoint-catalog-normal')
normal_products={profile:info['normal'] for profile,info in read(startup/'cohort.json')['profiles'].items()}
assert normal['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and normal['passed_per_profile']==16 and normal['recorded_owned_processes_all_terminal']
assert normal['observed_terminal_exit_code']==0 and not normal['source_overlays_used']
for selected,product in normal_products.items():assert product==normal['profiles'][selected]['normal'] and sha(product['path'])==normal['normal_mains'][selected]
''' +s[end:]
    elif oldname=='observe-native33A3-default116-completion-counted.py':
        s=rep(s,"sha(read(next(startup.glob(profile+'-qualified-*/qualified.json')))['normal']['path'])","sha(read(normal)['profiles'][profile]['normal']['path'])")
    elif oldname=='checkpoint-native33A3-default116-active.py':
        start=s.index("work='當版33A3CCF7");end=s.index('\ndate=',start)
        s=s[:start]+"work='來源2DAF4DE0正常兩版各16相關案、控制器96sessions、同端點租裝12sessions及Presentation非空音效96sessions已各自限定核對。廣域原'+str(a.session)+'沿同一程序執行兩版116整合目標及主程式，來源與對應QA固定；待原session實際終止及逐映像／名稱／raw／程序身分獨立讀回。舊33A3廣域／library不移記；當版完整library與其他必要正常可見性、服務及G01–G12仍待。開發期間不請使用者登入，8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立。'"+s[end:]
    ast.parse(s,filename=newname)
    if (T/newname).exists():assert (T/newname).read_text('utf-8')==s,'Existing staged tool differs; preserve it before any replacement'
    (out/oldname).write_text(original,encoding='utf-8',newline='\n');(T/newname).write_text(s,encoding='utf-8',newline='\n')
    (out/(newname+'.diff')).write_text(''.join(difflib.unified_diff(original.splitlines(True),s.splitlines(True))),encoding='utf-8',newline='\n')
    pins[newname]=dict(original=oldname,original_sha256=hashlib.sha256(original.encode()).hexdigest().upper(),new_sha256=hashlib.sha256(s.encode()).hexdigest().upper())
(out/'preparation.json').write_text(json.dumps(dict(tools=pins,source_manifest_sha256='2DAF4DE08EF396B1DE5B2264820E689859BFCD837015D71A6D74643A2DDEE9DE',normal_prerequisite_each_profile=16,graph_and_deep_readback_checks_unchanged=True,actual_executables_expected_per_profile=117,tests_executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(pins),AST='passed',tests_executed=False),ensure_ascii=False))
