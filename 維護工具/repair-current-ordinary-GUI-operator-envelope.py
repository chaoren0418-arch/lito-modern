"""Increase only future ordinary-room GUI orchestration envelope after failure."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return (json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
failure=w['current_normal_GUI_ordinary_operator_deadline_failure'];assert failure['original_session_id']==54314 and failure['original_identities_terminal'] and not failure['GUI_qualification_granted']
assert sha(failure['raw_archive'])==failure['raw_archive_sha256']
source=w['current_product_identity']['source_manifest_sha256'];assert source==failure['source_manifest_sha256']
inputs=read(Path(w['current_native_catalog_library']['run'])/'compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
changes={
 'prepare-native-installed-resource-profile-sky-gui.py':[
 ('gui&&profile.equals("rental-hotel-return-reconnect")?1_800_000_000_000L:', 'gui&&(profile.equals("rental-hotel-return-reconnect")||profile.equals("room-admin"))?1_800_000_000_000L:'),
 ('rental_overall_seconds=900,rental_hotel_return_overall_seconds=1800,','rental_overall_seconds=900,ordinary_room_overall_seconds=1800,rental_hotel_return_overall_seconds=1800,')],
 'start-native-installed-resource-profile-sky-gui-case.py':[
 ("else policy['rental_overall_seconds'] if hotel or ordinary", "else policy['ordinary_room_overall_seconds'] if ordinary and 'ordinary_room_overall_seconds' in policy else policy['rental_overall_seconds'] if hotel or ordinary")]
}
before={n:(T/n).read_bytes() for n in changes};staged={};diff=[]
for n,replacements in changes.items():
    old=before[n].decode('utf-8');new=old
    for a,b in replacements:assert new.count(a)==1,(n,a);new=new.replace(a,b)
    compile(new,str(T/n),'exec');staged[n]=new.encode('utf-8');diff.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=n+'/before',tofile=n+'/after'))
out=V/'native-room-form-label-tooling'/('ordinary-operator-envelope-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before-tools.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,b in before.items():z.writestr(n,b)
with zipfile.ZipFile(out/'before-tools.zip') as z:assert all(z.read(n)==b for n,b in before.items())
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');(out/'repair.py').write_bytes(Path(__file__).read_bytes())
assert all((T/n).read_bytes()==b for n,b in before.items())
for n,b in staged.items():(T/n).write_bytes(b)
assert all(sha(R/n)==h for n,h in inputs.items())
record=dict(source_manifest_sha256=source,trigger=failure['case'],only_future_ordinary_room_GUI_operator_envelope_changed=True,old_overall_seconds=900,new_overall_seconds=1800,accept_socket_wait_seconds=900,cleanup_seconds=30,gameplay_or_protocol_clocks_changed=False,normal_source_or_products_modified=False,qualification_granted=False,before_archive_sha256=sha(out/'before-tools.zip'),before_tools={n:hashlib.sha256(b).hexdigest().upper() for n,b in before.items()},after_tools={n:sha(T/n) for n in changes},review_sha256=sha(out/'review.diff'))
(out/'readback.json').write_bytes(data(record));print(json.dumps(dict(repair=str(out),qualification_granted=False),ensure_ascii=False))
