"""Extend only the eight-session GUI operator envelope after its preserved failure."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
failure=w['current_normal_GUI_hotel_return_failure'];assert failure['observed_actual_exit_code']==1 and failure['actual_masked_login_frames']==6 and not failure['GUI_qualification_granted']
source=w['current_product_identity']['source_manifest_sha256'];assert source=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
inputs=read(Path(w['current_native_catalog_library']['run'])/'compiled-inputs.json')
assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
changes={
'prepare-native-installed-resource-profile-sky-gui.py':[
('gui&&profile.endsWith("expiry")?8_000_000_000_000L:gui?900_000_000_000L:', 'gui&&profile.endsWith("expiry")?8_000_000_000_000L:gui&&profile.equals("rental-hotel-return-reconnect")?1_800_000_000_000L:gui?900_000_000_000L:'),
('rental_overall_seconds=900,rental_natural_expiry_seconds=8000,', 'rental_overall_seconds=900,rental_hotel_return_overall_seconds=1800,rental_natural_expiry_seconds=8000,')],
'start-native-installed-resource-profile-sky-gui-case.py':[
("else policy['rental_overall_seconds'] if hotel or ordinary", "else policy['rental_hotel_return_overall_seconds'] if a.scenario=='rental-hotel-return-reconnect' else policy['rental_overall_seconds'] if hotel or ordinary")]
}
before={n:(T/n).read_bytes()for n in changes};staged={};diffs=[]
for n,replacements in changes.items():
    old=before[n].decode('utf-8');new=old
    for a,b in replacements:assert new.count(a)==1,(n,a);new=new.replace(a,b)
    staged[n]=new.encode('utf-8');compile(new,str(T/n),'exec')
    diffs.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=n+'/before',tofile=n+'/after'))
out=V/'native-room-form-label-tooling'/('hotel-operator-wait-repair-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before-tools.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,b in before.items():z.writestr(n,b)
with zipfile.ZipFile(out/'before-tools.zip')as z:assert all(z.read(n)==b for n,b in before.items())
(out/'review.diff').write_text(''.join(diffs),encoding='utf-8',newline='')
assert all((T/n).read_bytes()==b for n,b in before.items())
for n,b in staged.items():(T/n).write_bytes(b)
assert all(sha(R/n)==h for n,h in inputs.items())
(out/'repair.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(source_manifest_sha256=source,trigger=failure['case'],review=str(out/'review.diff'),review_sha256=sha(out/'review.diff'),before_tools={n:hashlib.sha256(b).hexdigest().upper()for n,b in before.items()},after_tools={n:sha(T/n)for n in changes},before_archive_sha256=sha(out/'before-tools.zip'),only_eight_session_GUI_operator_envelope_changed=True,normal_products_and_all1049_inputs_unchanged=True,gameplay_or_protocol_clocks_changed=False,authority_or_constructor_changes=False,per_connection_socket_wait_changed=False,GUI_tests_rerun=False,qualification_granted=False))
print(json.dumps(dict(repair=str(out),next_action='prepare fresh GUI fixture, then rerun Release hotel return only',normal_products_unchanged=True),ensure_ascii=False))
