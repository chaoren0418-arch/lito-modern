"""Seal the original reader rejection before supporting observed keyboard input."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
case=V/'native-installed-resource-profile-sky-gui/case-20261020-110317-890181-debug-ordinary-room-service'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
c=read(case/'cohort.json');assert c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending' and c['recorded_owned_processes_terminal']
assert not(case/'qualified-room-service-keyboard').exists()
events=read(case/'Sky-events.json')
for role in ['a','b']:
    e=next(e for e in events if e['role']==role and e['label']=='source-choice')
    assert e['action']==dict(method='press_key',args={'key':'End'}) and '來源伺服器 Value: 測試乙 ID: 203' in e['tree']
e=next(e for e in events if e['role']=='b' and e['label']=='join-room-secret');assert e['action']==dict(method='set_value',args={'element_index':21,'value':'QA42'}) and 'ID: 206' in e['focused_element']
out=case/('reader-input-binding-rejection-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
reader=R/'維護工具/qualify-native-room-form-labels-GUI-room-service.py'
files={x.relative_to(case).as_posix():x for x in case.iterdir() if x.is_file()}
for x in(case/'captures').rglob('*'):
    if x.is_file():files[x.relative_to(case).as_posix()]=x
files.update({'reader-before.py':reader,'preserver.py':Path(__file__)})
hashes={n:sha(x) for n,x in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,x in files.items():z.write(x,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='reader_rejected_keyboard_End_instead_of_set_value_and_alternate_password_event_label',source_manifest_sha256=c['source_manifest_sha256'],original_session=13723,original_terminal_chunk='e506d3',original_exit_code=0,reader_terminal_chunk='fb074f',reader_exit_code=1,reader_error="KeyError: 'value' at source-choice",reader_before_sha256=sha(reader),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,normal_source_or_products_modified=False,GUI_qualification_granted=False)
(out/'evidence.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(evidence=str(out/'evidence.json')),ensure_ascii=False))
