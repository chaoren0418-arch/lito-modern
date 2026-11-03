"""Prepare current API/focused and generic controller tools without executing tests."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';OLD='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6';NEW='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
mapping={'qualify-current-rental-return-library-subset.py':'qualify-haste43-cancel-return-library-subset.py','run-native-rental-return-reconnect-api.py':'run-haste43-cancel-return-api.py','qualify-native-rental-return-reconnect-api.py':'qualify-haste43-cancel-return-api.py','checkpoint-native-rental-return-reconnect-api.py':'checkpoint-haste43-cancel-return-api.py','run-native-installed-resource-profile-controller.py':'run-haste43-cancel-controller.py'}
out=V/'native-haste43-cancel-tooling'/('API-controller-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);diff=[];proof={}
for name,new_name in mapping.items():
    old=(T/name).read_text('utf-8-sig');new=old.replace(OLD,NEW)
    if name=='qualify-current-rental-return-library-subset.py':
        for before,after in [('len(set(names)) == 10','len(set(names)) == 11'),('passed=10','passed=11'),('1974','1975'),('2462','2463'),('return_ten_','return_eleven_'),('Qualify ten actual','Qualify eleven actual')]:
            assert before in new;new=new.replace(before,after)
    if 'return-reconnect-api' in name:
        for before,after in mapping.items():new=new.replace(before,after)
        if name.startswith('run-'):
            assert "focused['profiles']['Debug']['passed']==10" in new;new=new.replace("focused['profiles']['Debug']['passed']==10","focused['profiles']['Debug']['passed']==11")
            new=new.replace('from native1030_broad_gate import require_idle_runtime','from haste43_cancel_runtime_gate import require_idle_runtime')
            anchor="'checkpoint-haste43-cancel-return-api.py']}";assert new.count(anchor)==1;new=new.replace(anchor,"'checkpoint-haste43-cancel-return-api.py','haste43_cancel_runtime_gate.py']}")
        if name.startswith('qualify-'):
            assert "['profiles']['Debug']['passed']==10" in new;new=new.replace("['profiles']['Debug']['passed']==10","['profiles']['Debug']['passed']==11")
    if name=='run-native-installed-resource-profile-controller.py':new=new.replace('from native1030_broad_gate import require_idle_runtime','from haste43_cancel_runtime_gate import require_idle_runtime')
    if new_name=='qualify-haste43-cancel-return-library-subset.py':assert 'assert len(inputs) == 1049' in new and '1149' not in new
    ast.parse(new);target=T/new_name;assert not target.exists();(out/(name+'.before')).write_bytes((T/name).read_bytes());target.write_text(new,encoding='utf-8',newline='\n');(out/new_name).write_bytes(target.read_bytes());proof[new_name]=dict(origin=name,before_sha256=sha(T/name),after_sha256=sha(target));diff.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=name,tofile=new_name))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');j=dict(source_manifest_sha256=NEW,tools=proof,new_original_return_regression_required=True,library_counts_changed_only_for_one_added_test=True,packet_authority_deadline_and_process_assertions_preserved=True,current_gate_sha256=sha(T/'haste43_cancel_runtime_gate.py'),executed=False)
(out/'preparation.json').write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(preparation=str(out),tools=list(proof),executed=False),ensure_ascii=False))
