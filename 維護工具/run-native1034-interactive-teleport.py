"""Owned passive C02 authority for an interactive OS SendInput teleport test.

This controller starts no UI automation and never changes server authority.
All clicks/keys must use the computer-use sky API in the calling chat.
"""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ctypes,hashlib,json,os,subprocess,time,zipfile
from ctypes import wintypes
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
APP=ROOT/'驗證紀錄/current/native1034-interactive-teleport'
APP.mkdir(exist_ok=True)
NORMAL=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950'
JDK=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
RUST=Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
def read(p):
    for i in range(10):
        try:return json.loads(p.read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('--profile',choices=['Debug','Release'],default='Release');ap.add_argument('--lease-seconds',type=int,default=1800);a=ap.parse_args();assert 60<=a.lease_seconds<=1800
qualifications={p:next(NORMAL.glob(p+'-qualified-*/qualified.json')) for p in ['Debug','Release']}
rows={p:read(v) for p,v in qualifications.items()}
assert all(v['passed']==57 and v['failed']==0 and v['ignored']==0 and v['source_manifest_sha256']=='86E65C7B9D9D77FF73FF154B59F28465B75BA1B3C1D1C6EF315D4990584B45AE' for v in rows.values())
pins=read(NORMAL/'compiled-inputs.json');assert len(pins)==1035
q={'profiles':{p:v['normal'] for p,v in rows.items()}}

def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
    for v in q['profiles'].values():assert sha(Path(v['path']))==v['sha256']
stable()
jar=PROJECT/'target/current/server-reference/formal-server-C02AAAE8.jar';assert sha(jar)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
formal=Path(r'D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar');assert sha(formal)==sha(jar)
catalog=PROJECT/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin';assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
quote=str(ROOT).replace("'","''")
probe="$r='"+quote+"';$p=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -in @('cargo.exe','rustc.exe')) -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|native_|qualified-client|actual-client)' -or $_.ExecutablePath -match '\\\\deps\\\\.*\\.exe$'))});if($p.Count){exit 1}"
assert subprocess.run(['powershell.exe','-NoProfile','-Command',probe],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW).returncode==0,'Owned load must be idle'
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=APP/stamp;runtime=PROJECT/'target/current/native-SendInput-teleport'/stamp
out.mkdir(parents=True,exist_ok=False);runtime.mkdir(parents=True,exist_ok=False);classes=runtime/'classes';classes.mkdir()
sources=[PROJECT/'tests/oracle'/n for n in ['NativePeerHarness.java','NativeTeleportHarness.java']]
for p in sources:(out/p.name).write_bytes(p.read_bytes())
original=(out/'NativeTeleportHarness.java').read_bytes();(out/'NativeTeleportHarness.original.java.txt').write_bytes(original)
adapted=original
for old,new in [(b'gui?240000:60000',b'gui?1800000:60000'),(b'gui?300_000_000_000L:120_000_000_000L',b'gui?1_800_000_000_000L:120_000_000_000L')]:
    assert adapted.count(old)==1;adapted=adapted.replace(old,new,1)
assert adapted.replace(b'gui?1800000:60000',b'gui?240000:60000',1).replace(b'gui?1_800_000_000_000L:120_000_000_000L',b'gui?300_000_000_000L:120_000_000_000L',1)==original
(out/'NativeTeleportHarness.java').write_bytes(adapted)
(out/'controller.py').write_bytes(Path(__file__).read_bytes());(out/'source-inputs.json').write_bytes((NORMAL/'compiled-inputs.json').read_bytes());(out/'normal-qualified.json').write_bytes(qualifications[a.profile].read_bytes())
receipt={'status':'compiling_owned_C02_test_harness','profile':a.profile,'source_files':1034,'compiled_inputs':1035,'source_manifest_sha256':rows[a.profile]['source_manifest_sha256'],'runtime':str(runtime),'output':str(out),'normal_qualification':str(qualifications[a.profile]),'main_sha256':q['profiles'][a.profile]['sha256'],'jar_sha256':sha(jar),'catalog_sha256':sha(catalog),'resource_profile':'tw13081901-v1','extended_item_details':True,'authority_mutations':0,'test_harness_changes':'Only owned-copy socket accept and observation lease extended to30minutes; canonical test sources and C02 JAR unchanged.','owned_process':None,'OS_SendInput_verified':False,'human_physical_device_verified':False,'whole_goal_complete':False}
persist=lambda:save(out/'receipt.json',receipt)
env=os.environ.copy();env.update(TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'),PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',RUSTC=str(RUST/'rustc.exe'),RUSTDOC=str(RUST/'rustdoc.exe'));env['PATH']=str(RUST)+os.pathsep+str(JDK)+os.pathsep+env['PATH']
compile_args=[str(JDK/'javac.exe'),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',str(jar),'-d',str(classes),*[str(out/p.name) for p in sources]]
with (out/'javac.stdout.log').open('wb') as stdout,(out/'javac.stderr.log').open('wb') as stderr:r=subprocess.run(compile_args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
receipt['javac_exit_code']=r.returncode;persist();assert r.returncode==0
args=[str(JDK/'java.exe'),'-XX:-UsePerfData','-Djava.io.tmpdir='+str(PROJECT/'target/current/scratch'),'-Dfile.encoding=UTF-8','-Xverify:all','-Dlito.resourceProfile=tw13081901-v1','-Dlito.extendedItemDetails=true','-Dlito.commonCatalog='+str(catalog),'-cp',str(classes)+';'+str(jar),'local.lito381.server.NativeTeleportHarness',str(runtime),'gui-cross-endpoint']
save(out/'server-command.json',{'args':args,'cwd':str(runtime),'canonical_sources':{str(p.relative_to(ROOT)):sha(p) for p in sources},'compiled_sources':{p.name:sha(out/p.name) for p in sources}})
started=time.monotonic()
with (out/'server.stdout.log').open('wb') as stdout,(out/'server.stderr.log').open('wb') as stderr:
    child=subprocess.Popen(args,cwd=runtime,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    times=[wintypes.FILETIME() for _ in range(4)];get=ctypes.WinDLL('kernel32',use_last_error=True).GetProcessTimes;get.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;get.restype=wintypes.BOOL;assert get(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(v) for v in times])
    ticks=(times[0].dwHighDateTime<<32)|times[0].dwLowDateTime;created=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
    receipt.update(status='waiting_owned_server_ready',owned_process={'pid':child.pid,'creation_utc':created.isoformat(),'args':args});persist()
    def stop_owned():
        if child.poll() is not None:return
        ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$p=Get-CimInstance Win32_Process -Filter 'ProcessId="+str(child.pid)+"';if($p){[pscustomobject]@{path=$p.ExecutablePath;created=$p.CreationDate.ToUniversalTime().ToString('o');command=$p.CommandLine}|ConvertTo-Json -Compress}"
        check=subprocess.run(['powershell.exe','-NoProfile','-Command',ps],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert check.returncode==0
        actual=json.loads(check.stdout.decode('utf-8-sig'));assert Path(actual['path']).resolve()==Path(args[0]).resolve();assert abs((datetime.fromisoformat(actual['created'])-created).total_seconds())<.001;assert actual['command'].strip()==subprocess.list2cmdline(args)
        child.kill();child.wait(timeout=10)
    try:
        ready=runtime/'ready.json'
        while not ready.exists():
            assert child.poll() is None,'Server exited before ready'
            assert time.monotonic()-started<30,'Ready deadline';time.sleep(.1)
        state=read(ready);assert state['jar_sha256'].upper()==sha(jar)
        save(out/'ready.json',state);save(APP/'SendInput-teleport-current.json',{'output':str(out),'runtime':str(runtime),'server_pid':child.pid,'creation_utc':created.isoformat(),'profile':a.profile,'status':'server_ready_interactive_input_not_yet_started'})
        receipt.update(status='ready_for_interactive_OS_SendInput',ready=state);persist();print(json.dumps({'output':str(out),'runtime':str(runtime),'ready':state,'server_pid':child.pid},ensure_ascii=False),flush=True)
        while child.poll() is None:
            if (out/'abort-owned-server.json').exists() or time.monotonic()-started>a.lease_seconds:
                stop_owned();raise RuntimeError('Owned server aborted or interactive lease expired')
            time.sleep(.2)
        receipt.update(server_exit_code=child.returncode,owned_process=None);assert child.returncode==0
        result=read(runtime/'result.json');assert result['actual_client_sessions']==6 and result['authority_mutations']==0 and result['memberships_empty'] and result['staged_loadouts_empty'];stable();assert sha(formal)==sha(jar)==receipt['jar_sha256']
        receipt.update(status='owned_authority_closed_input_evidence_qualification_pending',server=result);persist()
    except BaseException as error:
        stop_owned();receipt.update(owned_process=None,server_exit_code=child.returncode)
        receipt.update(status='failed_preserved',error=repr(error));persist();raise
    finally:
        receipt['elapsed_seconds']=round(time.monotonic()-started,3);persist()
        for name in ['result.json','server-trace.jsonl','staged-trace.jsonl']:
            p=runtime/name
            if p.exists():(out/name).write_bytes(p.read_bytes())
        files={p.name:p for p in out.iterdir() if p.is_file() and p.name!='controller-raw.zip'}
        hashes={n:sha(p) for n,p in files.items()}
        with zipfile.ZipFile(out/'controller-raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
            for n,p in files.items():z.write(p,n)
        with zipfile.ZipFile(out/'controller-raw.zip') as z:
            for n,h in hashes.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
        save(out/'controller-archive-readback.json',{'files':hashes,'sha256':sha(out/'controller-raw.zip'),'verified':True})
print(json.dumps({'status':receipt['status'],'output':str(out)},ensure_ascii=False),flush=True)
