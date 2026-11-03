"""Install the prepared GUI readers only after both full library profiles qualify."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('prepared',type=Path);a=ap.parse_args();prepared=a.prepared.resolve(strict=True);assert prepared.is_relative_to(V/'native-quantity-enter-failure-tooling')
p=read(prepared/'preparation.json');w=read(R/'工作佇列.json');assert p['source_manifest_sha256']==w['current_product_identity']['source_manifest_sha256']and not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
lib=w['current_native_catalog_library'];assert w['current_full_default_library_verified']and lib['source_manifest_sha256']==p['source_manifest_sha256']and sha(lib['completion'])==lib['completion_sha256']
before=p['before_reader_sha256'];assert len(before)==2 and all(sha(T/n)==h and sha(prepared/n)==p['after_reader_sha256'][n]for n,h in before.items())
assert sha(p['room_plan'])==p['room_plan_sha256']
out=V/'native-quantity-enter-failure-tooling'/('GUI-readers-installed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before-tools.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(T/n,n)
with zipfile.ZipFile(out/'before-tools.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
diff=[]
for n in before:diff.extend(difflib.unified_diff((T/n).read_text('utf-8-sig').splitlines(True),(prepared/n).read_text('utf-8-sig').splitlines(True),fromfile=n+'/before',tofile=n+'/after'))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='\n');(out/'installer.py').write_bytes(Path(__file__).read_bytes())
for n in before:(T/n).write_bytes((prepared/n).read_bytes())
record=dict(status='GUI_readers_installed_full_current_library_precondition_verified',preparation=str(prepared/'preparation.json'),preparation_sha256=sha(prepared/'preparation.json'),source_manifest_sha256=p['source_manifest_sha256'],before_archive_sha256=sha(out/'before-tools.zip'),after_readers={n:sha(T/n)for n in before},GUI_executed=False,qualification_granted=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(installed=str(out),GUI_executed=False),ensure_ascii=False))
