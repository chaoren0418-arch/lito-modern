"""Repair only the preserved history metadata aliases; keep live job identity intact."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
source='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041';old='D9B0F468FA65E186B246269A6F00B8D6B4EB76742049E3FBCC744357A780022D'
assert w['current_product_identity']['source_manifest_sha256']==source
out=R/'驗證紀錄/current/native-room-form-label-tooling'/('history-key-recovery-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
record=dict(status='preserved_D9_history_keys_recovered_from_existing_pinned_fields',source_manifest_sha256=source,prior_source_manifest_sha256=old,reason='Application generator retained defect under hotel_room_form_label_defect and omitted explicit historical hotel-return alias. Underlying D9 evidence remains present and independently hash-checked.',product_source_modified=False,qualifications_promoted=False,active_workloads_changed=False,whole_goal_complete=False)
staged={}
for n in before:
 p=R/n
 if p.suffix=='.json':
  v=read(p);assert v['current_product_identity']['source_manifest_sha256']==source
  h=v['qualification_before_room_form_label_repair'];assert h['source_manifest_sha256']==old and 'GUI_hotel_return' not in h
  defect=h['hotel_room_form_label_defect'];assert defect['source_manifest_sha256']==old and sha(defect['raw_archive'])==defect['raw_archive_sha256']
  hotels=v['current_normal_GUI_hotel_return_keyboard'];assert set(hotels)=={'Debug','Release'}
  for q in hotels.values():assert q['source_manifest_sha256']==old and sha(q['qualification'])==q['qualification_sha256']
  h.update(GUI_hotel_return=hotels,room_form_label_defect=defect,history_key_recovery=str(out/'evidence.json'))
  v['last_goal_turn_evidence']=str(out/'evidence.json');staged[n]=data(v)
 else:staged[n]=p.read_bytes()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
(out/'evidence.json').write_bytes(data(record));(out/'recovery.py').write_bytes(Path(__file__).read_bytes())
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
assert read(R/'工作佇列.json')['active_workload_sessions']==w['active_workload_sessions']
print(json.dumps(dict(evidence=str(out/'evidence.json'),active_sessions=w['active_workload_sessions']),ensure_ascii=False))
