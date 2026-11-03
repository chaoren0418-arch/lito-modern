"""Seal a failed readback and original reader before changing its oracle."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--chunk',required=True);ap.add_argument('--reason',required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-haste43-live-candidate')
c=read(run/'cohort.json');assert c['recorded_children_terminal'] and not(run/'qualified-self-peer-formal75').exists()
out=run/('reader-rejection-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
reader=R/'維護工具/qualify-current-haste43-live-candidate.py';files={'reader-before.py':reader,'cohort.json':run/'cohort.json','preserver.py':Path(__file__)}
for profile in ['Debug','Release']:
    for path in(run/profile/'case').rglob('*'):
        if path.is_file():files[path.relative_to(run).as_posix()]=path
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
j=dict(source_manifest_sha256=c['source_manifest_sha256'],reader_terminal_chunk=a.chunk,reader_exit_code=1,reason=a.reason,reader_before_sha256=sha(reader),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,qualification_granted=False,normal_product_modified=False)
(out/'evidence.json').write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(evidence=str(out/'evidence.json')),ensure_ascii=False))
