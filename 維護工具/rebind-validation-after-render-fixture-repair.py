"""Archive prepared tools before rebinding to the test-fixture-only final source."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
OLD='9F99C1203C0BB3F2CF87A873C7A81F7B20B08AFD3546A2282A84B62F6C8124C8'
NEW='4018AE86474EA2977F0083034E81A86D4F77F1C51EDF828B2456B369876DA24D'
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==NEW and not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
focused=w['current_native_render_fixture_regression'];assert focused['source_manifest_sha256']==NEW and focused['qualified']and sha(focused['qualification'])==focused['qualification_sha256']
names=set(read(V/'native-installed-resource-profile-tooling/validation-preparation-20261019-201342-436781/tooling.json')['tools'])
names.update(read(V/'native-installed-resource-profile-tooling/followups-20261019-201729-292173/tooling.json')['tools'])
names.update(['run-installed-shared-resource-library.py','checkpoint-installed-shared-resource-library.py','prepare-installed-shared-resource-selection.py','update-master-report-installed-resource-profile.py']);assert len(names)==34
out=V/'native-render-test-fixture-tooling'/('rebind-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
before={n:sha(T/n)for n in names};staged={}
for name in sorted(names):
 s=(T/name).read_text('utf-8-sig').replace(OLD,NEW).replace('來源9F99C120','來源4018AE86')
 if name=='qualify-native-installed-resource-profile-normal.py':
  start=s.index("candidate=read(application['candidate'])");end=s.index("verify_archive(app/'applied-payload.zip'",start)
  s=s[:start]+'''candidate=read(application['candidate'])
assert sha(application['candidate'])==application['candidate_sha256']
assert candidate['base_source_manifest_sha256']==application['before_source_manifest_sha256']=='9F99C1203C0BB3F2CF87A873C7A81F7B20B08AFD3546A2282A84B62F6C8124C8'
assert candidate['test_only_fixture_repair']and candidate['production_prefix_unchanged']
assert sha(candidate['failure_archive'])==candidate['failure_archive_sha256']
old=read(app/'before-compiled-inputs.json');assert set(old)==set(inputs)
changed={n for n,h in inputs.items()if old[n]!=h}
assert changed==set(application['changed_files'])==set(candidate['candidate_files'])=={'LiTo381-Modern/src/modern_lito/world_scene.rs'}
assert application['created_files']==[] and application['after_files']==candidate['candidate_files']and len(old)==len(inputs)==1049
'''+s[end:]
 if name=='update-master-report-installed-resource-profile.py':
  s=s.replace('保留原生繪製回傳修補，並套五檔cfg(test)資源資格更新；','保留原生繪製／資源修補，再修正一檔測試fixture；')
  s=s.replace("{link('資源測試更新／回復',applications[0])}","{link('資源測試更新／回復',V/'native-installed-resource-profile-normal/application-20261019-201457-707212/application.json')}")
  s=s.replace('四項新本文尚未執行。',"四項新本文尚未執行。\n\n{link('最新fixture修正／回復',applications[0])}：屋頂測試改以角色ID1定位；重現案視口改512×256並加入每個物件的完整邊界斷言，保留全部原像素／命中／排序檢查。先跑兩項原失敗Debug回歸，再跑兩版正常／完整library。")
  mark="for key,label in [('current_native_library_resource_optin'"
  assert s.count(mark)==1
  s=s.replace(mark,"""focused=w.get('current_native_render_fixture_regression',{})
if focused.get('source_manifest_sha256')==source and focused.get('qualified')is True:
    assert sha(focused['qualification'])==focused['qualification_sha256']
    rows.append('| Debug兩項原失敗定位回歸 | 2過0敗0忽略、2456篩除 | '+link('限定資格',focused['qualification'])+'；不是完整library或兩版正常資格 |')
"""+mark)
  s=s.replace('| BC2CA7E9正常兩版 |',"| 9F99C120正常兩版 | 各19過0敗0忽略 | {link('限定資格',w['qualification_before_render_test_fixture_repair']['normal']['qualification'])}；非當版資格 |\n| 9F99C120完整library | Debug1968過2敗488忽略；Rust101／wrapper1，Release未啟動；6記錄程序皆終止 | {link('失敗封存',Path(w['qualification_before_render_test_fixture_repair']['library_failure']['raw_archive']).with_name('evidence.json'))}；兩項為測試角色索引／視口配置，已修且待整批重驗 |\n| BC2CA7E9正常兩版 |")
  s=s.replace("V/'native-installed-resource-profile-tooling/followups-20261019-201729-292173/tooling.json'",repr(out/'tooling.json').replace("WindowsPath(","Path("))
  s=s.replace("V/'native-installed-resource-profile-tooling/shared-resource71-20261019-202309-616629/tooling.json'",repr(out/'tooling.json').replace("WindowsPath(","Path("))
  s=s.replace("{link('當版非預設初始fixture',", "{w['current_native_nondefault_initial_fixture_preparation']['source_manifest_sha256'][:8]}{link('初始fixture（依當版重備）',")
  s=s.replace("{link('非預設同端點probe候選',", "{w['current_native_nondefault_same_endpoint_probe_preparation']['source_manifest_sha256'][:8]}{link('非預設probe候選（需重綁）',")
 ast.parse(s);staged[name]=s.encode('utf-8')
with zipfile.ZipFile(out/'before-tools.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n in names:z.write(T/n,n)
with zipfile.ZipFile(out/'before-tools.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(T/n)==h for n,h in before.items())
for n,b in staged.items():(T/n).write_bytes(b)
(out/'generator.py').write_bytes(Path(__file__).read_bytes())
(out/'tooling.json').write_text(json.dumps(dict(source_manifest_sha256=NEW,before_source_manifest_sha256=OLD,before_archive_sha256=sha(out/'before-tools.zip'),tools={n:dict(before_sha256=before[n],after_sha256=sha(T/n))for n in sorted(names)},executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),rebound_tools=len(names),source_manifest_sha256=NEW,executed=False),ensure_ascii=False))
