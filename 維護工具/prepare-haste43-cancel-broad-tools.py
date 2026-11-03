"""Rebind broad orchestration identity/names, preserving all executable checks."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';OLD='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6';NEW='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
mapping={'native_installed_resource_profile_broad_gate.py':'haste43_cancel_broad_gate.py','run-native-installed-resource-profile-default117.py':'run-haste43-cancel-default117.py','run-native-installed-resource-profile-default117-both.py':'run-haste43-cancel-default117-both.py','observe-native-installed-resource-profile-default117-completion.py':'observe-haste43-cancel-default117-completion.py','checkpoint-native-installed-resource-profile-default117-active.py':'checkpoint-haste43-cancel-default117-active.py','checkpoint-native-installed-resource-profile-default117-progress.py':'checkpoint-haste43-cancel-default117-progress.py','checkpoint-native-installed-resource-profile-default117-completed.py':'checkpoint-haste43-cancel-default117-completed.py'}
out=V/'native-haste43-cancel-tooling'/('broad-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);diff=[];proof={}
for name,new_name in mapping.items():
    old=(T/name).read_text('utf-8-sig');new=old.replace(OLD,NEW)
    for before,after in sorted(mapping.items(),key=lambda p:-len(p[0])):new=new.replace(before,after)
    new=new.replace('from native_installed_resource_profile_broad_gate import','from haste43_cancel_broad_gate import')
    new=new.replace("work='來源A695C127原'","work='來源'+c['source_manifest_sha256'][:8]+'原'")
    ast.parse(new);target=T/new_name;assert not target.exists();(out/(name+'.before')).write_bytes((T/name).read_bytes());target.write_text(new,encoding='utf-8',newline='\n');(out/new_name).write_bytes(target.read_bytes());proof[new_name]=dict(origin=name,before_sha256=sha(T/name),after_sha256=sha(target));diff.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=name,tofile=new_name))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');j=dict(source_manifest_sha256=NEW,tools=proof,expected_targets_remain117_plus_main=True,both_profiles_and236_PE_assertions_retained=True,library_excluded_as_before=True,deadlines_and_execution_assertions_unchanged=True,executed=False)
(out/'preparation.json').write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(preparation=str(out),tools=list(proof),executed=False),ensure_ascii=False))
