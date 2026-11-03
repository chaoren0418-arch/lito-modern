"""Run the owned original selector only after all owned compiler/GUI work ends."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,subprocess,sys
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
OUT=Path(sys.argv[1]).resolve(strict=True);assert OUT.is_relative_to(ROOT/'驗證紀錄/current/native-background-dispatch-original')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
prep=read(OUT/'preparation-readback.json');assert sha(OUT/'preparation-raw.zip')==prep['archive_sha256']
for n,h in prep['files'].items():assert sha(OUT/n)==h,n
pins=read(OUT/'compiled-inputs.json');assert len(pins)==1015
def stable():
    for n,h in pins.items():assert sha(ROOT/n)==h,n
stable()
command="$r='"+str(ROOT).replace("'","''")+"';$p=@(Get-CimInstance Win32_Process|Where-Object {$_.Name -in @('cargo.exe','rustc.exe') -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|native_|lito_modern-|oracle)' -or ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))});if($p.Count){exit 1}"
idle=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert idle.returncode==0,'Owned workload remains live'
assert not (OUT/'run-receipt.json').exists();(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
r={'status':'building_original_selector_oracle','owned_process':None,'process_history':[],
    'source_files':1015,'product_source_changed':False,'game_sessions':0,'whole_goal_complete':False}
persist=lambda:save(OUT/'run-receipt.json',r)
env=os.environ.copy();env.update(TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'))
def run(args,name):
    with (OUT/name).open('wb') as so,(OUT/(name+'.stderr')).open('wb') as se:
        c=subprocess.Popen(args,cwd=OUT,env=env,stdout=so,stderr=se,creationflags=subprocess.CREATE_NO_WINDOW)
        item={'pid':c.pid,'arguments':args,'observed_started_utc':datetime.now(timezone.utc).isoformat()}
        r['owned_process']=item;r['process_history'].append(item);persist()
        code=c.wait();item.update(exit_code=code,finished_utc=datetime.now(timezone.utc).isoformat());r['owned_process']=None;r[name+'_exit_code']=code;persist()
        assert code==0,(name,code)
persist();save(APP/'background-dispatch-run-current.json',{'path':str(OUT)})
try:
    run(read(OUT/'build-command.json')['args'],'build.log');stable()
    r['status']='executing_original_selector_with_recording_callees';persist()
    run([str(OUT/'oracle.exe'),str(OUT)],'native-summary.json');stable()
    s=read(OUT/'native-summary.json');assert s['cases']==263423 and s['guards_unchanged']
    r.update(status='original_selector_ran_not_yet_independently_qualified',native_summary=s,source_inputs_unchanged=True)
except BaseException as e:
    r.update(status='failed_preserved',error=repr(e));raise
finally:persist()
print(json.dumps({'status':r['status'],'path':str(OUT),'cases':263423},ensure_ascii=False))
