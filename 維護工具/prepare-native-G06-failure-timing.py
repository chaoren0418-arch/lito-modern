"""Compile a timestamp-only copy of the failed recovery test against its unchanged library."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
BASE=ROOT/'驗證紀錄/current/native-G06-controller/20261017-034558-407944'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
failure=read(BASE/'failure-before-repair/readback.json');assert failure['original_session_id']==80000 and failure['test_exit_code']==101
source=PROJECT/'tests/native_teleport_failure.rs';text=source.read_text(encoding='utf-8')
extra=r'''
fn stamp(root:&Path,role:&str,event:&str,start:Instant,detail:Value)->Result<()>{
 use std::io::Write;
 let mut log=std::fs::OpenOptions::new().create(true).append(true).open(root.join(format!("{role}-timing.jsonl")))?;
 writeln!(log,"{}",json!({"event":event,"elapsed_ns":start.elapsed().as_nanos(),"detail":detail}))?;
 Ok(())
}
fn timed_model(root:&Path,role:&str,stage:&str,start:Instant,world:&native_world::World,cancel:&AtomicBool)->Result<Model>{
 stamp(root,role,&format!("{stage}_load_start"),start,json!({"actors":world.actors.len()}))?;
 let t=Instant::now();let result=Model::load(Path::new(r"D:\lineage開發\0-客戶端"),world,cancel);
 stamp(root,role,&format!("{stage}_load_end"),start,json!({"load_us":t.elapsed().as_micros(),"ok":result.is_ok()}))?;
 result
}
'''
at='fn failure_player(';assert text.count(at)==1;text=text.replace(at,extra+'\n'+at)
line=next(s for s in text.splitlines() if 'let mut model=Model::load' in s)
text=text.replace(line,'  let mut model=timed_model(root,role,"initial",start,&c.snapshot().world,&cancel)?;')
line=next(s for s in text.splitlines() if 'let mut recovered_model=Model::load' in s)
text=text.replace(line,'  let mut recovered_model=timed_model(root,role,"recovery",start,&reentered.world,&cancel)?;')
old='let steps=walk(&mut c,&mut model,npc,&cancel,start)?;'
assert text.count(old)==1;text=text.replace(old,old+'stamp(root,role,"initial_walk_complete",start,json!({"steps":steps.len()}))?;')
old='  let continued=c.snapshot().clone();';assert text.count(old)==1
text=text.replace(old,'  stamp(root,role,"fresh_GAME_continued",start,json!({}))?;\n'+old)
old='  std::fs::write(root.join(format!("{role}-failure-complete")),b"done")?;'
assert text.count(old)==1;text=text.replace(old,'  stamp(root,role,"recovery_move_complete_wait_other",start,json!({}))?;\n'+old)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=BASE/('failure-timing-diagnostic-'+stamp);out.mkdir()
runtime=PROJECT/'target/current/native-teleport-failure-timing'/stamp;runtime.mkdir(parents=True)
probe=ROOT/'維護工具/oracle/NativeTeleportFailureTimingDiagnostic.rs';probe.write_text(text,encoding='utf-8',newline='\n')
assert bytes([0]) not in probe.read_bytes(), 'Generated diagnostic contains NUL'
(out/'probe.rs').write_bytes(probe.read_bytes());(out/'builder.py').write_bytes(Path(__file__).read_bytes())
prior=read(ROOT/'驗證紀錄/current/native-G06-controller/20261017-030505-804788/movement-timing-diagnostic-20261017-031911-251190/build.json')
for info in prior['library_inputs'].values():assert sha(info['path'])==info['sha256']
exe=runtime/'native-teleport-failure-timing.exe';args=prior['build_arguments'].copy()
args[args.index('--test')+1]=str(probe);args[args.index('--crate-name')+1]='native_teleport_failure_timing_diagnostic';args[args.index('-o')+1]=str(exe)
env=os.environ.copy();env.update(CARGO_MANIFEST_DIR=str(PROJECT),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
record=dict(status='compiling_failure_phase_timing_diagnostic',source_manifest_sha256=prior['source_manifest_sha256'],source=str(probe),source_sha256=sha(probe),original_test_source_sha256=sha(source),library_inputs=prior['library_inputs'],build_arguments=args,product_sources_changed=False,deadline_seconds_unchanged=True,original_failure_archive_sha256=failure['raw_archive_sha256'],performance_acceptance_claimed=False,whole_goal_complete=False)
save(out/'build.json',record);print(json.dumps(dict(output=str(out)),ensure_ascii=False),flush=True)
with (out/'build.stdout.log').open('wb') as stdout,(out/'build.stderr.log').open('wb') as stderr:
    p=subprocess.Popen(args,cwd=PROJECT,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
    record['compiler_pid']=p.pid;save(out/'build.json',record);code=p.wait()
record.update(build_exit_code=code,status='compiled_failure_phase_timing_diagnostic' if code==0 else 'failed_preserved')
if code==0:record.update(executable=str(exe),executable_sha256=sha(exe))
save(out/'build.json',record);assert code==0,'Retain compile failure before correction'
save(BASE/'latest-failure-timing-build.json',dict(build=str(out),executable=str(exe),executable_sha256=sha(exe)))
print(json.dumps(dict(status=record['status'],executable=str(exe)),ensure_ascii=False))
