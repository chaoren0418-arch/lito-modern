"""Bind only the new source identity; preserve full library execution assertions."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';OLD='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6';NEW='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
mapping={
'run-native-installed-resource-profile-library-default.py':'run-haste43-cancel-library-default.py',
'observe-native-installed-resource-profile-library-default.py':'observe-haste43-cancel-library-default.py',
'checkpoint-native-installed-resource-profile-library-completed.py':'checkpoint-haste43-cancel-library-completed.py',
'checkpoint-native-installed-resource-profile-library-active.py':'checkpoint-haste43-cancel-library-active.py'}
out=V/'native-haste43-cancel-tooling'/('library-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);proof={};diff=[]
for name,new_name in mapping.items():
    origin=T/name;old=origin.read_text('utf-8-sig');new=old
    if name.endswith('active.py'):
        at=new.index("work='來源'");end=new.index('\n',at)
        new=new[:at]+"work='來源'+cohort['source_manifest_sha256'][:8]+'原'+str(args.session_id)+'：完整library沿同一原程序執行，逐名、結果、PE、來源及終止待核對；正常兩版各19已限定核對，其他資源／API／控制器／廣域與GUI須依新來源資格接續。不重跑已核對範圍、不改JAR／時限，不要求使用者操作。'"+new[end:]
    else:
        assert new.count(OLD)==1;new=new.replace(OLD,NEW)
    ast.parse(new);target=T/new_name;assert not target.exists();(out/(name+'.before')).write_bytes(origin.read_bytes());target.write_text(new,encoding='utf-8',newline='\n');(out/new_name).write_bytes(target.read_bytes());proof[new_name]=dict(origin=name,before_sha256=sha(origin),after_sha256=sha(target));diff.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=name,tofile=new_name))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');j=dict(source_manifest_sha256=NEW,tools=proof,full_selection_counts_remain_inventory_driven=True,all_protocol_PE_archive_and_process_checks_retained=True,executed=False)
(out/'preparation.json').write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(preparation=str(out),new_tools=list(proof),executed=False),ensure_ascii=False))
