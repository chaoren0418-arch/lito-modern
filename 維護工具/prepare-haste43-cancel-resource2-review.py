"""Fresh ignored-source partition for the two reviewed resource2 families."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';OLD='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6';NEW='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==NEW
library=V/'native-installed-resource-profile-library-default/20261020-115615-042695';audit_path=library/'Debug/inventory-audit/ignored-source-reasons.json';audit=read(audit_path);inputs=read(library/'compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
prior=V/'native-library-remaining-optin-review/20261020-041932-322366/review.json';old=read(prior);assert old['remaining_exact_tests']==407
available={x['test']:x for x in audit['tests']};assert len(available)==488
for row in old['tests']:
    assert row['test'] in available and sha(R/row['source'])==row['source_sha256']==inputs[row['source']]
    assert any(x['source']==row['source'] and x['source_sha256']==row['source_sha256'] for x in available[row['test']]['candidates'])
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native-library-remaining-optin-review'/('E0E6409E-'+stamp);out.mkdir();fresh=dict(old);fresh.update(source_manifest_sha256=NEW,library=str(library),audit=str(audit_path),audit_sha256=sha(audit_path),prior_review=str(prior),prior_review_sha256=sha(prior),all407_exact_source_files_unchanged=True,new_source_runtime_qualifications_granted=False,tests_executed=0)
path=out/'review.json';path.write_text(json.dumps(fresh,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
mapping={'run-current-library-motion-resource2.py':'run-haste43-cancel-motion-resource2.py','checkpoint-current-library-motion-resource2.py':'checkpoint-haste43-cancel-motion-resource2.py','run-current-library-haste-resource2.py':'run-haste43-cancel-haste-resource2.py','checkpoint-current-library-haste-resource2.py':'checkpoint-haste43-cancel-haste-resource2.py'}
toolout=V/'native-haste43-cancel-tooling'/('resource2-'+stamp);toolout.mkdir();proof={};diff=[]
for name,new_name in mapping.items():
    original=(T/name).read_text('utf-8-sig');assert OLD in original;new=original.replace(OLD,NEW)
    old_literal="V / 'native-library-remaining-optin-review/20261020-041932-322366/review.json'"
    if name.startswith('run-'):assert new.count(old_literal)==1;new=new.replace(old_literal,'Path(r"'+str(path)+'")')
    if name.startswith('checkpoint-'):assert new.count('== 2460')==1;new=new.replace('== 2460','== 2461')
    for a,b in mapping.items():new=new.replace(a,b)
    ast.parse(new);target=T/new_name;assert not target.exists();(toolout/(name+'.before')).write_bytes((T/name).read_bytes());target.write_text(new,encoding='utf-8',newline='\n');(toolout/new_name).write_bytes(target.read_bytes());proof[new_name]=dict(origin=name,before_sha256=sha(T/name),after_sha256=sha(target));diff.extend(difflib.unified_diff(original.splitlines(True),new.splitlines(True),fromfile=name,tofile=new_name))
(toolout/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');record=dict(source_manifest_sha256=NEW,review=str(path),review_sha256=sha(path),tools=proof,filtered_out_expected2461_because_total2463_minus2=True,test_names_and_body_assertions_unchanged=True,full_library_completion_required_before_plan_or_execution=True,executed=False);(toolout/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(preparation=str(toolout),review=str(path),executed=False),ensure_ascii=False))
