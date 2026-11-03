"""Rebind GUI plans/readers only; no GUI or source qualification is carried over."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
OLD='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D';NEW='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
out=V/'native-text-index-order-tooling'/('GUI-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
old_plan=V/'native-haste43-cancel-tooling/GUI-20261020-115815-159424/room-positive-plan.json';plan=read(old_plan);assert plan['source_manifest_sha256']==OLD;plan.update(source_manifest_sha256=NEW,reprepared_from=str(old_plan),reprepared_from_sha256=sha(old_plan),GUI_executed=False);new_plan=out/'room-positive-plan.json';new_plan.write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
names=['prepare-haste43-cancel-sky-gui.py','checkpoint-haste43-cancel-sky-gui.py','qualify-haste43-cancel-GUI-room-service.py','qualify-haste43-cancel-GUI-rental-entry.py','qualify-haste43-cancel-GUI-hotel-entry.py','qualify-haste43-cancel-GUI-hotel-return.py','prepare-haste43-cancel-Enter-negative-plan.py','qualify-haste43-cancel-Enter-negatives.py','prepare-haste43-cancel-EOF-plan.py','qualify-haste43-cancel-EOF-cleanup.py'];rows=[]
for name in names:
    origin=T/name;before=origin.read_text('utf-8-sig');assert OLD in before;after=before.replace(OLD,NEW).replace('haste43-cancel','text-index-order')
    if name=='qualify-haste43-cancel-GUI-room-service.py':
        line=next(l for l in after.splitlines(True) if l.startswith('plan_path=Path('));after=after.replace(line,'plan_path=Path(r"'+str(new_plan)+'")\n');assert after.count(sha(old_plan))==1;after=after.replace(sha(old_plan),sha(new_plan))
    target=T/name.replace('haste43-cancel','text-index-order');assert not target.exists();ast.parse(after);(out/name).write_bytes(origin.read_bytes());target.write_text(after,encoding='utf-8',newline='\n');(out/(target.name+'.diff')).write_text(''.join(difflib.unified_diff(before.splitlines(True),after.splitlines(True),fromfile=name,tofile=target.name)),encoding='utf-8');rows.append(dict(origin=str(origin),before_sha256=sha(origin),current=str(target),current_sha256=sha(target)))
(out/'preparation.json').write_text(json.dumps(dict(source_manifest_sha256=NEW,tools=rows,room_plan=str(new_plan),room_plan_sha256=sha(new_plan),original_assertions_retained=True,GUI_executed=False,qualification_granted=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'preparer.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(preparation=str(out/'preparation.json'),tools=len(rows),GUI_executed=False),ensure_ascii=False))
