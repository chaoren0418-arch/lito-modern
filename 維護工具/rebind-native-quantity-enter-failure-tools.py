"""Bind current readers to the reviewed three-file repair; retain every old tool."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
OLD='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
NEW='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==NEW and not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
history=w['qualification_before_quantity_enter_failure_retirement'];assert history['source_manifest_sha256']==OLD and history['negative_baseline']['baseline_counterexamples_verified']
prior=read(V/'native-room-form-label-tooling/rebind-20261020-012828-930975/tooling.json');assert prior['source_manifest_sha256']==OLD
names=set(prior['tools'])|{'qualify-native-room-form-labels-GUI-room-service.py','qualify-current-rental-return-library-subset.py'};assert len(names)==51
prepared=T/'qualify-native-quantity-enter-failure-retirement-normal.py';assert sha(prepared)=='9927DA773B49311DFD606ABC00CB4D4676FC1703379E6D66599A23B5100AD4A6'
before={n:(T/n).read_bytes()for n in names};staged={};diff=[]
for n,b in before.items():
    text=b.decode('utf-8-sig')
    # The prepared normal reader deliberately retains OLD as its before-source guard.
    updated=prepared.read_text('utf-8-sig')if n=='qualify-native-installed-resource-profile-normal.py'else text.replace(OLD,NEW).replace(OLD[:8],NEW[:8])
    ast.parse(updated);staged[n]=updated.encode('utf-8')
    diff.extend(difflib.unified_diff(text.splitlines(True),updated.splitlines(True),fromfile=n+'/before',tofile=n+'/after'))
out=V/'native-quantity-enter-failure-tooling'/('rebind-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
with zipfile.ZipFile(out/'before-tools.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,b in before.items():z.writestr(n,b)
with zipfile.ZipFile(out/'before-tools.zip')as z:assert set(z.namelist())==names and all(z.read(n)==b for n,b in before.items())
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='\n');(out/'generator.py').write_bytes(Path(__file__).read_bytes())
assert all((T/n).read_bytes()==b for n,b in before.items())and not read(R/'工作佇列.json')['active_workloads']
for n,b in staged.items():(T/n).write_bytes(b)
record=dict(source_manifest_sha256=NEW,prior_source_manifest_sha256=OLD,before_archive_sha256=sha(out/'before-tools.zip'),normal_reader_before_source_guard_retained=True,normal_reader_verifies_exact_three_file_repair_and_baseline_counterexamples=True,original_counts_protocol_and_clock_assertions_preserved=True,GUI_quantity_Enter_reader_pending=True,executed=False,tools={n:dict(before_sha256=hashlib.sha256(before[n]).hexdigest().upper(),after_sha256=sha(T/n),changed=before[n]!=staged[n])for n in sorted(names)})
(out/'tooling.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(tooling=str(out),rebound_tools=len(names),source_manifest_sha256=NEW,executed=False),ensure_ascii=False))
