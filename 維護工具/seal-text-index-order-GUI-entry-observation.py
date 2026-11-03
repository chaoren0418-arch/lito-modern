"""Retain an actual rejected GUI readback without granting entry qualification."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, zipfile
R=Path(__file__).resolve().parents[1]
C=R/'驗證紀錄/current/native-installed-resource-profile-sky-gui/case-20261020-141316-623300-release-rental-entry'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
c=read(C/'cohort.json');assert c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending'
assert all(x['exit_code']==0 for x in c['processes'])
e=read(C/'Sky-events.json');a={x['label']:x for x in e if x['role']=='a'}
assert '正在載入伺服器指定的世界資源' in a['world-focus']['tree'] and '地圖 500' not in a['world-focus']['tree']
assert '地圖 500' in a['world-after-walk']['tree']
out=C/'reader-rejection-world-not-ready';out.mkdir()
files={p.name:p for p in C.iterdir() if p.is_file()}
files['qualifier.py']=R/'維護工具/qualify-text-index-order-GUI-rental-entry.py'
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
rec=dict(source_manifest_sha256=c['source_manifest_sha256'],case=str(C),profile='Release',original_session_id=98757,
 terminal_tool_chunk='df4a71',original_exit_code=0,reader_tool_chunk='eae5f3',reader_exit_code=1,
 failure='world-focus was observed before map 500 load completed; strict original assertion rejected it',
 product_defect_proven=False,qualification_granted=False,GUI_full_flow_verified=False,whole_goal_complete=False,
 action='Retain all raw events and unchanged reader. Rerun fresh case, observe map 500 before movement.',
 raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files={n:sha(p) for n,p in files.items()})
(out/'evidence.json').write_bytes(data(rec));(out/'seal.py').write_bytes(Path(__file__).read_bytes())
work='Release租裝入口原98757 runtime退出0，但獨立reader原eae5f3退出1：A world-focus仍在載入，未通過原地圖500斷言；原始紀錄封存，reader及產品不改，重備新批次等待地圖500後才移動。完整GUI仍待。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);v.setdefault('current_GUI_readback_rejections',[]).append(rec)
        v.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=data(v)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),qualification_granted=False),ensure_ascii=False))
