"""Reprepare current normal GUI readers/plans, retaining original exact assertions."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';OLD='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6';NEW='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
out=V/'native-haste43-cancel-tooling'/('GUI-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
original_plan=V/'native-quantity-enter-failure-tooling/GUI-readers-prepared-20261020-040710-552916/room-positive-plan.json';plan=read(original_plan);assert plan['source_manifest_sha256']==OLD
plan.update(source_manifest_sha256=NEW,reprepared_from=str(original_plan),reprepared_from_sha256=sha(original_plan),GUI_executed=False);new_plan=out/'room-positive-plan.json';new_plan.write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
mapping={
'prepare-native-installed-resource-profile-sky-gui.py':'prepare-haste43-cancel-sky-gui.py',
'checkpoint-native-installed-resource-profile-sky-gui.py':'checkpoint-haste43-cancel-sky-gui.py',
'qualify-native-room-form-labels-GUI-room-service.py':'qualify-haste43-cancel-GUI-room-service.py',
'qualify-native-installed-resource-profile-GUI-rental-entry.py':'qualify-haste43-cancel-GUI-rental-entry.py',
'qualify-native-installed-resource-profile-GUI-hotel-entry.py':'qualify-haste43-cancel-GUI-hotel-entry.py',
'qualify-native-rental-return-reconnect-GUI-hotel-return.py':'qualify-haste43-cancel-GUI-hotel-return.py',
'prepare-native-quantity-Enter-GUI-negative-plan.py':'prepare-haste43-cancel-Enter-negative-plan.py',
'qualify-native-quantity-Enter-GUI-negatives.py':'qualify-haste43-cancel-Enter-negatives.py',
'prepare-native-EOF-cleanup-GUI-plan.py':'prepare-haste43-cancel-EOF-plan.py',
'qualify-native-EOF-cleanup-GUI.py':'qualify-haste43-cancel-EOF-cleanup.py'}
diff=[];proof={}
for name,new_name in mapping.items():
    old=(T/name).read_text('utf-8-sig');assert OLD in old;new=old.replace(OLD,NEW)
    if name=='qualify-native-room-form-labels-GUI-room-service.py':
        line=next(line for line in new.splitlines(True) if line.startswith('plan_path=Path('));new=new.replace(line,'plan_path=Path(r"'+str(new_plan)+'")\n')
        old_sha='BDC891773459B24F5D37A2BCB9D1875CE5F23FE6E60AC0A3BCDDC47A56A2891E';assert new.count(old_sha)==1;new=new.replace(old_sha,sha(new_plan))
    ast.parse(new);target=T/new_name;assert not target.exists();(out/(name+'.before')).write_bytes((T/name).read_bytes());target.write_text(new,encoding='utf-8',newline='\n');(out/new_name).write_bytes(target.read_bytes());proof[new_name]=dict(origin=name,before_sha256=sha(T/name),after_sha256=sha(target));diff.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=name,tofile=new_name))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');j=dict(source_manifest_sha256=NEW,tools=proof,room_plan=str(new_plan),room_plan_sha256=sha(new_plan),original_plan_sha256=sha(original_plan),normal_PE_and_formal_packet_authority_lifecycle_assertions_retained=True,GUI_executed=False,qualification_granted=False,full_GUI_qualification_granted=False)
(out/'preparation.json').write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(preparation=str(out),tools=list(proof),GUI_executed=False),ensure_ascii=False))
