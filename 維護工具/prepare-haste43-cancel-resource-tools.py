"""Prepare unchanged10/71 resource selections for the current library version."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';OLD='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6';NEW='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
mapping={'run-native-installed-resource-profile-library-resource-optin.py':'run-haste43-cancel-resource10.py','checkpoint-native-installed-resource-profile-library-resource-optin.py':'checkpoint-haste43-cancel-resource10.py','prepare-installed-shared-resource-selection.py':'prepare-haste43-cancel-shared71-selection.py','review-installed-shared-resource-selection-current.py':'review-haste43-cancel-shared71-selection.py','run-installed-shared-resource-library.py':'run-haste43-cancel-shared71.py','checkpoint-installed-shared-resource-library.py':'checkpoint-haste43-cancel-shared71.py'}
out=V/'native-haste43-cancel-tooling'/('resources-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);proof={};diff=[]
for name,new_name in mapping.items():
    old=(T/name).read_text('utf-8-sig');assert OLD in old;new=old.replace(OLD,NEW)
    for before,after in mapping.items():new=new.replace(before,after)
    if name=='prepare-installed-shared-resource-selection.py':assert new.count('(1974,488,2462)')==1;new=new.replace('(1974,488,2462)','(1975,488,2463)')
    ast.parse(new);target=T/new_name;assert not target.exists();(out/(name+'.before')).write_bytes((T/name).read_bytes());target.write_text(new,encoding='utf-8',newline='\n');(out/new_name).write_bytes(target.read_bytes());proof[new_name]=dict(origin=name,before_sha256=sha(T/name),after_sha256=sha(target));diff.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=name,tofile=new_name))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');j=dict(source_manifest_sha256=NEW,tools=proof,selected_10_and71_bodies_unchanged=True,strict_legacy5_exclusion_preserved=True,resources_readonly_and_diagnostic_retention_assertions_preserved=True,expected_full_library_counts_only_increment_new_cancel_test=True,executed=False)
(out/'preparation.json').write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(preparation=str(out),tools=list(proof),executed=False),ensure_ascii=False))
