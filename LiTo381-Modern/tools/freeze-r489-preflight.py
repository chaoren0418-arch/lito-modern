from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r489a-preflight-library';O.mkdir(exist_ok=False)
def pin(p):
 h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(8<<20),b''):h.update(b)
 return dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
build=[json.loads(x)for x in (L/'r489a-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];assert [x['success']for x in build if x['reason']=='build-finished']==[True]
artifact=[Path(f)for b in build if b['reason']=='compiler-artifact'and b['target']['name']=='lito_modern'for f in b['filenames']if f.endswith('.rlib')];assert len(artifact)==1
dest=O/'liblito_modern.rlib';shutil.copyfile(artifact[0],dest);assert pin(dest)['sha256']==pin(artifact[0])['sha256'];sources=[]
for p in sorted((P/'src').rglob('*')):
 if p.is_file():
  dst=O/'source'/p.relative_to(P);dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dst);q=pin(dst);q['source_origin']=str(p);sources.append(q)
assert len(sources)==604
out=dict(state='R489A_SUCCESSFUL_PREFLIGHT_LIBRARY_NOT_LINKED_OR_EXECUTED_SUPERSEDED_BY_SIGNED_DISPATCH_CORRECTION',library=pin(dest),sources=sources,build_log=pin(L/'r489a-library-build.jsonl'),stderr=pin(L/'r489a-library-build.stderr.log'),reason='Final source review found native dispatch uses signed JL for phase<3. Correct u32 phase inspection to signed i32 comparison before any player/consumer link or test. No failing product test.',executions=0,GUI=0,LOADED=False)
(L/'r489-preflight-build.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('R489a successful preflight library and604source snapshot retained;0 links/executions')
