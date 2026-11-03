from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def pin(p):
 h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(8<<20),b''):h.update(b)
 return dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
rows=[json.loads(x)for x in(L/'r486c-final-lib-test-build.jsonl').read_text(encoding='utf-8-sig').splitlines()]
assert any(x.get('reason')=='build-finished'and x['success']for x in rows)
a=[x for x in rows if x.get('reason')=='compiler-artifact'and x['target']['name']=='lito_modern'and x['profile']['test']and x.get('executable')];assert len(a)==1
root=P/'target/r486c-final-test-compile';assert not root.exists();root.mkdir();exe=root/'r486c-lib-tests-not-executed.exe';shutil.copyfile(a[0]['executable'],exe)
sources=[]
for p in sorted((P/'src').rglob('*')):
 if p.is_file():
  dest=root/'source'/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);s=pin(p);assert pin(dest)['sha256']==s['sha256'];s['snapshot']=str(dest);sources.append(s)
assert len(sources)==602
out=dict(state='R486_FINAL_602_LIBRARY_TEST_SOURCES_COMPILE_PASS_ONLY',executable=pin(exe),source_pins=sources,build_json=pin(L/'r486c-final-lib-test-build.jsonl'),build_stderr=pin(L/'r486c-final-lib-test-build.stderr.log'),executions=0,full_test_suite_executed=False,PortalRecord_missing_field_compile_errors=0,initial_four_cases_record=pin(L/'r486-tile-test-candidate.json'),GUI=0,LOADED=False)
(L/'r486-final-test-compile.json').write_text(json.dumps(out,ensure_ascii=False,indent=2),encoding='utf-8');print('R486 final602libtest sources compiled;fixed exact EXE/source snapshots;0 additional executions')
