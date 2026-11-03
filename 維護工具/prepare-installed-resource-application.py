"""Prepare a test-only transaction; application requires current normal terminal proof."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
source='B0628AC904FE5AB7C2227FF176811A4A6208EAC7B5BA8892C73D0AFD99F2F2C4'
candidate=T/'candidates/installed-resource-profiles-20261019-201101-087897'
s=(T/'apply-native-render-placement-repair.py').read_text('utf-8-sig')
s=s.replace("C=R/'維護工具/candidates/native-render-placement-20261019-200226-719223'","C=Path("+repr(str(candidate))+")")
s=s.replace("OLD='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'","OLD="+repr(source))
start=s.index("assert w['current_full_default_library_verified']")
end=s.index("assert all(sha(R/n)==h for n,h in inputs.items())",start)
s=s[:start]+'''normal=w['current_native_render_placement_normal_qualification']
assert normal['source_manifest_sha256']==OLD and normal['qualification_granted']
assert sha(normal['qualification'])==normal['qualification_sha256']
qualification=read(normal['qualification']);assert qualification['observed_terminal_exit_code']==0 and qualification['recorded_owned_processes_all_terminal']and qualification['passed_per_profile']==19
base=Path(normal['run']);pins=read(base/'source-inputs.json');inputs=read(base/'compiled-inputs.json')
assert sha(base/'source-inputs.json')==OLD and len(pins)==1048 and len(inputs)==1049
lib=w['current_native_catalog_library'];res=w['current_native_library_resource_optin'];failed=w['current_native_library_shared_resource_optin']
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(R)
'''+s[end:]
s=s.replace("assert c['failed_evidence']==failed['analysis']and c['failure_archive_sha256']==failed['raw_archive_sha256']", "assert sha(c['independent_Text_oracle'])==c['independent_Text_oracle_sha256']and len(c['new_ignored_tests'])==4 and c['product_runtime_code_changed']is False")
s=s.replace("assert len(c['changed_files'])==1", "assert len(c['changed_files'])==5")
s=s.replace("V/'native-render-placement-normal'", "V/'native-installed-resource-profile-normal'")
s=s.replace("status='applied_native_render_placement_pending_validation'", "status='applied_installed_resource_profile_tests_pending_validation'")
s=s.replace("normal_product_source_modified=True", "normal_product_source_modified=False,test_only_source_modified=True")
s=s.replace("change_scope=c['change_scope']", "change_scope='Test-only current Text reference bound to independent CSharp hashes and four explicit current launcher TW cases; retain every legacy root-pack entry point, golden, pixel/clock/ownership assertion.'")
start=s.index("work='來源'");end=s.index('\nstaged=dict(payloads)',start)
s=s[:start]+"work='來源'+source[:8]+'套用五檔cfg(test)資源資格更新，新增四項現行launcher TW忽略案例；舊根目錄測試與原圖像／時鐘斷言保留。B062原生繪製標號／排序修補保留，正常兩版19冒煙保留原來源。先當版正常兩版與完整library Debug／Release、必要原資源，接續正常GUI／Release全流程，未驗證前不交人工。'"+s[end:]
s=s.replace("v['qualification_before_render_placement_repair']", "v['qualification_before_installed_resource_profiles']")
s=s.replace("application=v['current_native_rental_service_entry_return_application']", "application=v['current_native_render_placement_application']")
s=s.replace('current_native_render_placement_application=record', 'current_native_installed_resource_profile_application=record')
s=s.replace("item='native-render-placement-validation'", "item='native-installed-resource-profile-validation'")
s=s.replace('assert len(staged)==9', 'assert len(staged)==13')
ast.parse(s)
p=T/'apply-installed-resource-profile-tests.py';assert not p.exists();p.write_text(s,encoding='utf-8',newline='\n')
out=V/'native-installed-resource-profile-tooling'/('application-preparation-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'generator.py').write_bytes(Path(__file__).read_bytes());(out/'receipt.json').write_text(json.dumps(dict(application_tool=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest().upper(),requires_normal_qualification=source,applied=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tool=str(p),tooling=str(out),applied=False),ensure_ascii=False))
