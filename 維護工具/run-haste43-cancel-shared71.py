"""Run the 71 current shared_resource71 ignored resource cases from qualified library PEs.

This does not execute historical TLS/SQL tests or grant GUI/audio-device scope.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, re, subprocess, zipfile, shutil

R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';P=R/'LiTo381-Modern'
SOURCE='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
LEGACY={'modern_lito::disintegrate_effect::tests::shared_one_shots_match_csharp_composites_and_release_owned_assets', 'modern_lito::world_scene::tests::real_disintegrate_layers_share_world_depth_and_survive_target_removal', 'modern_lito::world_runtime::npc_fire_storm_runtime_tests::actual_npc70_runtime_preloads_once_and_renders_one_owned_group_without_replay', 'modern_lito::disintegrate_assets::tests::all_four_shared_sprites_match_independent_csharp_pixels', 'modern_lito::tw_action_catalog::tests::shared_tw_pack_has_real_ultimate_definitions_without_base_catalog_fallback'}
FIXED=['shared-sprite-reference.png', 'shared-map509-static-reference.png', 'shared-map4-ground-reference.png', 'shared-map4-static-reference.png', 'shared-map4-actor-reference.png', 'shared-map514-static-reference.png', 'shared-map514-actors-reference.png']
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
now=lambda:datetime.now(timezone.utc).isoformat()
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.GetCurrentProcess.restype=wintypes.HANDLE
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def birth(h):
    t=[wintypes.FILETIME()for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(x)for x in t])
    ticks=t[0].dwLowDateTime|(t[0].dwHighDateTime<<32)
    return(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat()
ap=argparse.ArgumentParser();ap.add_argument('library',type=Path);ap.add_argument('--plan',type=Path,required=True);a=ap.parse_args()
PLAN=a.plan.resolve(strict=True);assert PLAN.is_relative_to(V/'native-installed-resource-profile-shared-resource-plan')
PLAN_SHA=sha(PLAN);plan=read(PLAN)
assert plan['source_manifest_sha256']==SOURCE and plan['expected_tests']==71
review_path=PLAN.parent/'review.json';review=read(review_path)
assert review['plan_sha256']==PLAN_SHA and review['review_completed']and review['source_manifest_sha256']==SOURCE
assert sha(PLAN.parent/'test-bodies.json')==plan['selected_bodies_sha256']
assert sha(PLAN.parent/'shared-helper-bodies.json')==plan['shared_helpers_sha256']
names={x['test']for x in plan['selected_tests']};assert len(names)==71
base=a.library.resolve(strict=True);assert str(base)==plan['library'];assert base.is_relative_to(V/'native-installed-resource-profile-library-default')
completion=read(base/'completion-observation.json');queue=read(R/'工作佇列.json')
assert completion['source_manifest_sha256']==SOURCE and completion['observed_terminal_exit_code']==0
assert completion['recorded_original_owned_processes_all_terminal']
assert queue['current_full_default_library_verified'] and queue['current_native_catalog_library']['run']==str(base)
assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
inputs=read(base/'compiled-inputs.json');assert len(inputs)==1049
def check():
    for n,h in inputs.items():assert sha(R/n)==h,n
    assert sha(base/'completion-observation.json')==state['library_completion_sha256']
out=V/'native-installed-resource-profile-shared-resource-optin'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True);(out/'plan.json').write_bytes(PLAN.read_bytes());(out/'test-bodies.json').write_bytes(Path(plan['selected_bodies']).read_bytes());(out/'shared-helper-bodies.json').write_bytes(Path(plan['shared_helpers']).read_bytes());(out/'review.json').write_bytes(review_path.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
state=dict(status='preparing_explicit_shared_resource71_resource_optin',source_manifest_sha256=SOURCE,
    library=str(base),library_completion_sha256=sha(base/'completion-observation.json'),
    pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),profiles={},owned_process=None,
    expected_tests_per_profile=71,review_sha256=sha(review_path),plan=str(PLAN),plan_sha256=PLAN_SHA,explicit_body_review='71 current functions and three shared helpers; nine diagnostic PNG writers, Command::world is a model value, no live UI/network/device call; exact output preservation and isolated captures',default_library_results_changed=False,historical_TLS_SQL_executed=False,
    resources_read_only=True,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
env=os.environ.copy()
for n in list(env):
    if n.startswith('LITO_'):del env[n]
env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))

resource=Path(r'D:\lineage開發\0-客戶端');xml=Path(r'D:\lineage開發\客戶端xml\effectlist2.xml')
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
    for profile in ['Debug','Release']:
        check();d=base/profile;qp=d/'full-default-qualified/qualified.json';q=read(qp)
        assert q['source_manifest_sha256']==SOURCE and q['failed']==q['Rust_exit_code']==0
        product=read(d/'product-retention.json');exe=Path(product['product']).resolve(strict=True)
        assert exe.is_relative_to(P/'target/current') and sha(exe)==product['product_sha256']==q['actual_test_sha256']
        audit=read(d/'inventory-audit/ignored-source-reasons.json')
        available={x['test']:x for x in audit['tests']if len(x.get('candidates',[]))==1 and 'LITO_SHARED_RESOURCE_ROOT'in x['candidates'][0]['reason']}
        assert len(available)==76 and LEGACY<=set(available)and names==set(available)-LEGACY
        selected={x['test']:x for x in audit['tests']if x['test']in names}
        assert set(selected)==names and len(selected)==71
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
        with(target/'tests.log').open('wb')as stdout,(target/'stderr.log').open('wb')as stderr:
            child=subprocess.Popen(command,cwd=target,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            row=dict(pid=child.pid,creation_utc=birth(wintypes.HANDLE(int(child._handle))),command=command,
                product_sha256=sha(exe),qualification=str(qp),qualification_sha256=sha(qp),
                expected_tests=selected,exit_code=None)
            state.update(status='running_explicit_shared_resource71_resource_optin',active_profile=profile,owned_process=row)
            state['profiles'][profile]=row;save(out/'cohort.json',state)
            code=child.wait();row.update(exit_code=code,finished_utc=now());state['owned_process']=None
        save(out/'cohort.json',state)
        log=(target/'tests.log').read_text('utf-8-sig')
        results=re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$',log,re.M)
        assert len(results)==71 and {n for n,_ in results}==set(selected)
        totals=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
        assert len(totals)==1 and totals[0][3:5]==('0','0')
        row.update(passed=int(totals[0][1]),failed=int(totals[0][2]),ignored=int(totals[0][3]),filtered_out=int(totals[0][5]),
            results=dict(results),stdout_sha256=sha(target/'tests.log'),stderr_sha256=sha(target/'stderr.log'))
        assert row['filtered_out']==q['inventory']-71
        fresh=target/'diagnostic-PNG';fresh.mkdir()
        pngs={}
        for n in FIXED:
            src=legacy/n
            if not src.is_file():continue
            shutil.copyfile(src,fresh/n);pngs[n]=sha(fresh/n)
        for src in captures.rglob('*.png'):
            rel=src.relative_to(captures);dest=fresh/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dest);pngs[rel.as_posix()]=sha(dest)
        row['diagnostic_PNGs']=pngs
        assert metadata()==resources_before and sha(xml)==xml_sha
        save(out/'cohort.json',state)
        assert all(result=='ok'for _,result in results)and code==0 and row['passed']==71 and row['failed']==0
        assert set(FIXED)<=set(pngs)and len(pngs)>7
        check();save(out/'cohort.json',state)
        print(json.dumps(dict(profile=profile,passed=71,failed=0,ignored=0),ensure_ascii=False),flush=True)
    state.update(status='executed_both_current_shared_resource71_resource_optin_selections',active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved',error=repr(error));raise
finally:
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
    state['finished_utc']=now();save(out/'cohort.json',state)
print(json.dumps(dict(output=str(out),status=state['status']),ensure_ascii=False),flush=True)
