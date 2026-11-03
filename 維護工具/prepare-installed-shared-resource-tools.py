"""Prepare exact current-profile resource runner/readback; start no test here."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
SOURCE='9F99C1203C0BB3F2CF87A873C7A81F7B20B08AFD3546A2282A84B62F6C8124C8'
tree=ast.parse((T/'prepare-installed-shared-resource-selection.py').read_text('utf-8-sig'))
legacy=ast.literal_eval(next(n.value for n in tree.body if isinstance(n,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='LEGACY'for t in n.targets)))
runner_name='run-installed-shared-resource-library.py';reader_name='checkpoint-installed-shared-resource-library.py'
s=(T/'run-current-shared-resource-library.py').read_text('utf-8-sig')
start=s.index('PLAN=Path(');end=s.index('FIXED=',start);s=s[:start]+'LEGACY='+repr(legacy)+'\n'+s[end:]
s=s.replace("ap.add_argument('library',type=Path);a=ap.parse_args()", "ap.add_argument('library',type=Path);ap.add_argument('--plan',type=Path,required=True);a=ap.parse_args()")
s=s.replace('plan=read(PLAN);assert sha(PLAN)==PLAN_SHA', '''PLAN=a.plan.resolve(strict=True);assert PLAN.is_relative_to(V/'native-installed-resource-profile-shared-resource-plan')
PLAN_SHA=sha(PLAN);plan=read(PLAN)
assert plan['source_manifest_sha256']==SOURCE and plan['expected_tests']==71
review_path=PLAN.parent/'review.json';review=read(review_path)
assert review['plan_sha256']==PLAN_SHA and review['review_completed']and review['source_manifest_sha256']==SOURCE
assert sha(PLAN.parent/'test-bodies.json')==plan['selected_bodies_sha256']
assert sha(PLAN.parent/'shared-helper-bodies.json')==plan['shared_helpers_sha256']''')
s=s.replace("(out/'test-bodies.json').write_bytes(Path(plan['selected_bodies']).read_bytes());", "(out/'test-bodies.json').write_bytes(Path(plan['selected_bodies']).read_bytes());(out/'shared-helper-bodies.json').write_bytes(Path(plan['shared_helpers']).read_bytes());(out/'review.json').write_bytes(review_path.read_bytes());")
s=s.replace('expected_tests_per_profile=72', 'expected_tests_per_profile=71,review_sha256=sha(review_path)')
s=s.replace('len(names)==72','len(names)==71').replace('len(selected)==72','len(selected)==71').replace('len(results)==72','len(results)==71')
s=s.replace("        selected={x['test']:x for x in audit['tests']if x['test']in names}", "        available={x['test']:x for x in audit['tests']if len(x.get('candidates',[]))==1 and 'LITO_SHARED_RESOURCE_ROOT'in x['candidates'][0]['reason']}\n        assert len(available)==76 and LEGACY<=set(available)and names==set(available)-LEGACY\n        selected={x['test']:x for x in audit['tests']if x['test']in names}")
s=s.replace("        assert all(result=='ok'for _,result in results) and code==0\n", '')
s=s.replace("assert len(totals)==1 and totals[0][:5]==('ok','72','0','0','0')", "assert len(totals)==1 and totals[0][3:5]==('0','0')")
s=s.replace('row.update(passed=72,failed=0,ignored=0,', 'row.update(passed=int(totals[0][1]),failed=int(totals[0][2]),ignored=int(totals[0][3]),')
s=s.replace("q['inventory']-72", "q['inventory']-71")
s=s.replace('src=legacy/n;assert src.is_file(),n\n            shutil.copyfile', 'src=legacy/n\n            if not src.is_file():continue\n            shutil.copyfile')
s=s.replace("        assert len(pngs)>7 and metadata()==resources_before and sha(xml)==xml_sha", "        assert metadata()==resources_before and sha(xml)==xml_sha\n        save(out/'cohort.json',state)\n        assert all(result=='ok'for _,result in results)and code==0 and row['passed']==71 and row['failed']==0\n        assert set(FIXED)<=set(pngs)and len(pngs)>7")
s=s.replace('passed=72,failed=0', 'passed=71,failed=0')
s=s.replace('native-rental-service-entry-return-library-default','native-installed-resource-profile-library-default')
s=s.replace('native-library-shared-resource-optin-execution','native-installed-resource-profile-shared-resource-optin')
s=s.replace('shared_resource72','shared_resource71').replace('72 sealed source functions','71 current functions and three shared helpers')
s=s.replace('Run the ten current shared_resource71','Run the 71 current shared_resource71')
s=s.replace('BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B',SOURCE)
ast.parse(s);runner=T/runner_name;assert not runner.exists();runner.write_text(s,encoding='utf-8',newline='\n')
s=(T/'checkpoint-current-shared-resource-library.py').read_text('utf-8-sig')
s=s.replace("R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'", "R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'\nLEGACY="+repr(legacy))
s=s.replace('run-current-shared-resource-library.py',runner_name)
s=s.replace('native-library-shared-resource-optin-execution','native-installed-resource-profile-shared-resource-optin')
s=s.replace('native-rental-service-entry-return-library-default','native-installed-resource-profile-library-default')
s=s.replace("names={x['test']for x in plan['selected_tests']};assert len(names)==72", "names={x['test']for x in plan['selected_tests']};assert len(names)==71\n            available={x['test']for x in audit['tests']if len(x.get('candidates',[]))==1 and 'LITO_SHARED_RESOURCE_ROOT'in x['candidates'][0]['reason']}\n            assert len(available)==76 and LEGACY<=available and names==available-LEGACY\n            assert sha(run/'shared-helper-bodies.json')==plan['shared_helpers_sha256']\n            review=read(run/'review.json');assert review['review_completed']and review['plan_sha256']==sha(run/'plan.json')and review['source_manifest_sha256']==source\n            assert sha(run/'review.json')==c['review_sha256']")
s=s.replace('len(expected)==72','len(expected)==71').replace('len(found)==72','len(found)==71')
s=s.replace("('ok','72','0','0','0',str(q['inventory']-72))", "('ok','71','0','0','0',str(q['inventory']-71))")
s=s.replace('passed=72','passed=71').replace("q['ignored']-72", "q['ignored']-71")
s=s.replace("=402 if w.get", "=407 if w.get").replace('各72過0敗','各71過0敗').replace('原始共用資源72項','現行共用資源71項')
s=s.replace("for n in ['plan.json','test-bodies.json','resource-metadata-before.json'", "for n in ['plan.json','test-bodies.json','shared-helper-bodies.json','review.json','resource-metadata-before.json'")
s=s.replace('shared_resource72','shared_resource71')
s=s.replace('BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B',SOURCE)
ast.parse(s);reader=T/reader_name;assert not reader.exists();reader.write_text(s,encoding='utf-8',newline='\n')
out=V/'native-installed-resource-profile-tooling'/('shared-resource71-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'generator.py').write_bytes(Path(__file__).read_bytes());(out/'tooling.json').write_text(json.dumps(dict(source_manifest_sha256=SOURCE,runner=str(runner),runner_sha256=sha(runner),reader=str(reader),reader_sha256=sha(reader),selection=71,legacy_excluded=5,executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),executed=False),ensure_ascii=False))
