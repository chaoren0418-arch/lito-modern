from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
candidate=T/'candidates/native-render-test-fixture-20261019-203944-248950'
s=(T/'apply-native-render-placement-repair.py').read_text('utf-8-sig')
s=s.replace("C=R/'維護工具/candidates/native-render-placement-20261019-200226-719223'",'C=Path('+repr(str(candidate))+')')
s=s.replace("OLD='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'","OLD='9F99C1203C0BB3F2CF87A873C7A81F7B20B08AFD3546A2282A84B62F6C8124C8'")
start=s.index("assert w['current_full_default_library_verified']");end=s.index("assert all(sha(R/n)==h for n,h in inputs.items())",start)
s=s[:start]+'''normal=w['current_native_installed_resource_profile_normal_qualification'];failed=w['current_native_catalog_library_failure']
assert normal['source_manifest_sha256']==failed['source_manifest_sha256']==OLD
assert normal['qualification_granted']and failed['observed_exit_code']==1 and failed['recorded_original_owned_processes_all_terminal']
assert sha(normal['qualification'])==normal['qualification_sha256']and sha(failed['raw_archive'])==failed['raw_archive_sha256']
qualification=read(normal['qualification']);assert qualification['observed_terminal_exit_code']==0 and qualification['recorded_owned_processes_all_terminal']and qualification['passed_per_profile']==19
base=Path(normal['run']);pins=read(base/'source-inputs.json');inputs=read(base/'compiled-inputs.json')
assert sha(base/'source-inputs.json')==OLD and len(pins)==1048 and len(inputs)==1049
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(R)
'''+s[end:]
s=s.replace("assert c['failed_evidence']==failed['analysis']and c['failure_archive_sha256']==failed['raw_archive_sha256']", "assert c['failure_archive_sha256']==failed['raw_archive_sha256']and c['test_only_fixture_repair']and c['production_prefix_unchanged']")
s=s.replace("V/'native-render-placement-normal'", "V/'native-installed-resource-profile-normal'")
s=s.replace("status='applied_native_render_placement_pending_validation'", "status='applied_render_test_fixture_repair_pending_validation'")
s=s.replace('normal_product_source_modified=True','normal_product_source_modified=False,test_only_source_modified=True')
start=s.index("work='來源'");end=s.index('\nstaged=dict(payloads)',start)
s=s[:start]+"work='來源'+source[:8]+'只修world_scene測試fixture：屋頂以角色ID1定位，重現案512x256並驗所有物件邊界，原像素／命中／排序斷言保留。先Debug兩項原失敗定位回歸，再唯一入口兩版正常19冒煙及完整library Debug／Release、必要原資源與GUI；9F99C120的1968過2敗488忽略／Release未啟動已封存，不交人工。'"+s[end:]
old="v['qualification_before_render_placement_repair']=dict(source_manifest_sha256=OLD,normal=normal,library=lib,resource=res,resource_failure=failed,application=v['current_native_rental_service_entry_return_application'])"
new="v['qualification_before_render_test_fixture_repair']=dict(source_manifest_sha256=OLD,normal=normal,library_failure=failed,application=v['current_native_installed_resource_profile_application'])"
assert s.count(old)==1;s=s.replace(old,new)
s=s.replace('current_native_render_placement_application=record','current_native_installed_resource_profile_application=record')
s=s.replace("item='native-render-placement-validation'","item='native-render-test-fixture-validation'")
ast.parse(s);p=T/'apply-render-test-fixture-repair.py';assert not p.exists();p.write_text(s,encoding='utf-8',newline='\n')
out=V/'native-render-test-fixture-tooling'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'generator.py').write_bytes(Path(__file__).read_bytes());(out/'receipt.json').write_text(json.dumps(dict(tool=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest().upper(),applied=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tool=str(p),applied=False),ensure_ascii=False))
