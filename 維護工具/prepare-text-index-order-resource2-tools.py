"""Fresh 407-name static review; distinguish the two tests in the changed source."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
OLD='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D';NEW='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==NEW
active=w['current_native_catalog_library_active'];assert active['source_manifest_sha256']==NEW;library=Path(active['run']);inputs=read(library/'compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
audit_path=library/'Debug/inventory-audit/ignored-source-reasons.json';audit=read(audit_path);available={x['test']:x for x in audit['tests']};assert len(available)==488
prior=V/'native-library-remaining-optin-review/E0E6409E-20261020-120830-464631/review.json';old=read(prior);assert old['remaining_exact_tests']==407
application=w['current_native_installed_resource_profile_application'];app=Path(application['candidate_review']).parent;name='LiTo381-Modern/src/modern_lito/resources.rs'
with zipfile.ZipFile(app/'before.zip') as z:old_resources=z.read(name).decode('utf-8').replace('\r\n','\n')
current=(R/name).read_text('utf-8-sig');cut='    fn real_shared_text_matches_csharp_reference() {';assert current[:current.index(cut)]==old_resources[:old_resources.index(cut)]
rows=[];changed=[]
for original in old['tests']:
    row=dict(original);entry=available[row['test']];assert len(entry['candidates'])==1;item=entry['candidates'][0];assert item['source']==row['source'] and sha(R/row['source'])==inputs[row['source']]==item['source_sha256']
    if item['source_sha256']!=row['source_sha256']:
        assert row['source']==name and current.index('fn '+row['test'].split('::')[-1]+'(')<current.index(cut);changed.append(row['test']);row['prior_source_sha256']=row['source_sha256'];row['source_hash_changed_only_in_another_verified_test']=True
    row['source_sha256']=item['source_sha256'];rows.append(row)
assert len(changed)==2 and all('windows_reparse_' in n for n in changed)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native-library-remaining-optin-review'/('3E58268A-'+stamp);out.mkdir();fresh=dict(old);fresh.update(source_manifest_sha256=NEW,library=str(library),audit=str(audit_path),audit_sha256=sha(audit_path),prior_review=str(prior),prior_review_sha256=sha(prior),tests=rows,all407_exact_source_files_unchanged=False,all407_test_bodies_unchanged=True,test_bodies_in_changed_source_file=changed,remaining405_tests_whole_source_unchanged=True,new_source_runtime_qualifications_granted=False,tests_executed=0);path=out/'review.json';path.write_text(json.dumps(fresh,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
toolout=V/'native-text-index-order-tooling'/('resource2-'+stamp);toolout.mkdir(parents=True);proof=[];old_path=str(prior)
for name in ['run-haste43-cancel-motion-resource2.py','checkpoint-haste43-cancel-motion-resource2.py','run-haste43-cancel-haste-resource2.py','checkpoint-haste43-cancel-haste-resource2.py','prepare-haste43-cancel-haste-resource2-plan.py']:
    origin=T/name;before=origin.read_text('utf-8-sig');after=before.replace(OLD,NEW).replace('haste43-cancel','text-index-order').replace(old_path,str(path));target=T/name.replace('haste43-cancel','text-index-order');assert not target.exists();ast.parse(after)
    if name.startswith('run-'):assert str(path) in after
    (toolout/name).write_bytes(origin.read_bytes());target.write_text(after,encoding='utf-8',newline='\n');(toolout/(target.name+'.diff')).write_text(''.join(difflib.unified_diff(before.splitlines(True),after.splitlines(True),fromfile=name,tofile=target.name)),encoding='utf-8');proof.append(dict(origin=str(origin),before_sha256=sha(origin),current=str(target),current_sha256=sha(target)))
(toolout/'preparation.json').write_text(json.dumps(dict(source_manifest_sha256=NEW,review=str(path),review_sha256=sha(path),tools=proof,all_test_assertions_retained=True,filtered_out_expected=2461,executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(toolout/'preparer.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(preparation=str(toolout/'preparation.json'),review=str(path),executed=False),ensure_ascii=False))
