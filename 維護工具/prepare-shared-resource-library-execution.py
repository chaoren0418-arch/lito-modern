"""Derive exact installed-resource selection from its sealed audit, without running it."""
from pathlib import Path
from datetime import datetime, timezone
import ast, hashlib, json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
plan=V/'native-library-shared-resource-optin/plan-20261019-194354-677571/plan.json'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
p=read(plan);assert p['expected_tests']==72
names={x['test']for x in p['selected_tests']};assert len(names)==72
fixed=['shared-sprite-reference.png','shared-map509-static-reference.png','shared-map4-ground-reference.png',
       'shared-map4-static-reference.png','shared-map4-actor-reference.png','shared-map514-static-reference.png','shared-map514-actors-reference.png']
runner=(T/'run-native-rental-service-entry-return-library-resource-optin.py').read_text('utf-8-sig')
runner=runner.replace('native-rental-service-entry-return-library-resource-optin','native-library-shared-resource-optin-execution')
runner=runner.replace('PREFIX=\'modern_lito::native_disintegrate_tests::native77_probe::\'',
    'PLAN=Path('+repr(str(plan))+')\nPLAN_SHA='+repr(sha(plan))+'\nFIXED='+repr(fixed))
runner=runner.replace('import argparse, ctypes, hashlib, json, os, re, subprocess, zipfile',
    'import argparse, ctypes, hashlib, json, os, re, subprocess, zipfile, shutil')
runner=runner.replace("base=a.library.resolve(strict=True);", "plan=read(PLAN);assert sha(PLAN)==PLAN_SHA\nnames={x['test']for x in plan['selected_tests']};assert len(names)==72\nbase=a.library.resolve(strict=True);assert str(base)==plan['library'];")
runner=runner.replace("expected_tests_per_profile=10", "expected_tests_per_profile=72,plan=str(PLAN),plan_sha256=PLAN_SHA,explicit_body_review='72 sealed source functions; nine diagnostic PNG writers, Command::world is a model value, no live UI/network/device call; exact output preservation and isolated captures'")
runner=runner.replace("out.mkdir(parents=True);", "out.mkdir(parents=True);(out/'plan.json').write_bytes(PLAN.read_bytes());(out/'test-bodies.json').write_bytes(Path(plan['selected_bodies']).read_bytes());")
runner=runner.replace("try:\n    for profile", '''
resource=Path(r'D:\\lineage開發\\0-客戶端');xml=Path(r'D:\\lineage開發\\客戶端xml\\effectlist2.xml')
def metadata():
    result={}
    for root in [resource,xml.parent]:
        for x in root.rglob('*'):
            assert not (getattr(x.lstat(),'st_file_attributes',0)&0x400),str(x)
            if x.is_file():
                s=x.stat();result[str(x)]=[s.st_size,s.st_mtime_ns]
    return result
resources_before=metadata();save(out/'resource-metadata-before.json',resources_before);xml_sha=sha(xml)
env.update(LITO_SHARED_RESOURCE_ROOT=str(resource),LITO_EFFECT_CATALOG_PATH=str(xml),LITO_WORLD_MAP_WRITE_REFERENCE='1',LITO_WORLD_SCENE_WRITE_REFERENCE='1')
legacy=P/'target/modern-lito-render-tests'
assert legacy.resolve().is_relative_to(P/'target')
legacy.mkdir(exist_ok=True)
before={n:sha(legacy/n)for n in FIXED if(legacy/n).exists()}
with zipfile.ZipFile(out/'diagnostic-before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(legacy/n,n)
save(out/'diagnostic-before.json',before)
try:
    for profile''')
start=runner.index("        selected={x['test']:x for x in audit['tests']")
end=runner.index("        with(target/'tests.log')",start)
runner=runner[:start]+'''        selected={x['test']:x for x in audit['tests']if x['test']in names}
        assert set(selected)==names and len(selected)==72
        for row in selected.values():
            assert row['source_mapping']=='unique_source_attribute'and len(row['candidates'])==1
            src=row['candidates'][0];assert sha(R/src['source'])==src['source_sha256']
            assert 'LITO_SHARED_RESOURCE_ROOT'in src['reason']
        target=out/profile;target.mkdir()
        for option in ['--help','--list']:
            command=[str(exe),*sorted(names),'--exact','--ignored',option]
            query=subprocess.run(command,cwd=target,env=env,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
            assert query.returncode==0 and not query.stderr
            (target/('selection-help.txt'if option=='--help'else'selection-list.txt')).write_bytes(query.stdout)
            text=query.stdout.decode('utf-8-sig')
            if option=='--help':assert 'FILTERS' in text and '--exact'in text and '--ignored'in text
            else:assert {line[:-6]for line in text.splitlines()if line.endswith(': test')}==names
        captures=P/'target/current/diagnostic-output'/('shared-resource-'+out.name)/profile
        captures.mkdir(parents=True)
        for name,sub in [('LITO_MODERN_DISINTEGRATE_CAPTURE_DIR','disintegrate'),('LITO_MODERN_ROOF_CAPTURE_DIR','roof')]:
            path=captures/sub;path.mkdir();env[name]=str(path)
        command=[str(exe),*sorted(names),'--exact','--ignored','--test-threads=1']
'''+runner[end:]
runner=runner.replace("len(results)==10", "len(results)==72").replace("('ok','10','0','0','0')", "('ok','72','0','0','0')")
runner=runner.replace("passed=10", "passed=72").replace("q['inventory']-10", "q['inventory']-72")
runner=runner.replace("        check();save(out/'cohort.json',state)", '''        fresh=target/'diagnostic-PNG';fresh.mkdir()
        pngs={}
        for n in FIXED:
            src=legacy/n;assert src.is_file(),n
            shutil.copyfile(src,fresh/n);pngs[n]=sha(fresh/n)
        for src in captures.rglob('*.png'):
            rel=src.relative_to(captures);dest=fresh/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dest);pngs[rel.as_posix()]=sha(dest)
        row['diagnostic_PNGs']=pngs
        assert len(pngs)>7 and metadata()==resources_before and sha(xml)==xml_sha
        check();save(out/'cohort.json',state)''')
runner=runner.replace("finally:\n    state['finished_utc']", '''finally:
    with zipfile.ZipFile(out/'diagnostic-before.zip')as z:
        assert set(z.namelist())==set(before)
        for n in FIXED:
            if n in before:
                raw=z.read(n);assert hashlib.sha256(raw).hexdigest().upper()==before[n];(legacy/n).write_bytes(raw)
            elif(legacy/n).exists():
                assert(legacy/n).resolve().is_relative_to(legacy.resolve());(legacy/n).unlink()
    assert {n:sha(legacy/n)for n in FIXED if(legacy/n).exists()}==before
    state['resource_metadata_unchanged']=metadata()==resources_before
    state['diagnostic_historical_outputs_restored']=True
    state['finished_utc']''')
# Status vocabulary is distinct from native77; the standalone reader still verifies actual logs.
runner=runner.replace('native77','shared_resource72')
ast.parse(runner)
dest=T/'run-current-shared-resource-library.py';assert not dest.exists();dest.write_text(runner,encoding='utf-8',newline='\n')
out=V/'native-library-shared-resource-optin'/('tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'generator.py').write_bytes(Path(__file__).read_bytes())
(out/'review.json').write_text(json.dumps(dict(plan=str(plan),plan_sha256=sha(plan),runner=str(dest),runner_sha256=sha(dest),
    exact_names=72,fixed_diagnostic_outputs=fixed,resources_read_only=True,body_review_completed=True,
    help_and_list_runtime_validation_required=True,executed=False,GUI_verified=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),runner=str(dest),executed=False),ensure_ascii=False))
