"""Only invoke after the agent observes the original tool handle's actual exit."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
ap=argparse.ArgumentParser();ap.add_argument('normal',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);args=ap.parse_args()
NORMAL=args.normal.resolve(strict=True);assert NORMAL.is_relative_to(APP)
assert args.terminal_exit_code==0
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
g=read(NORMAL/'GUI-current.json');n=read(NORMAL/'qualified.json');pins=read(NORMAL/'source-inputs.json')
assert n['status']=='qualified_unique_entry_floor_port_normal_builds' and len(pins)==1005
assert g['status']=='qualified_current_lighting_TW14_GUI_cases' and not g['source_and_corresponding_tests_frozen']
for name,value in pins.items():assert sha(ROOT/name)==value,name
for m in n['profiles'].values():assert sha(m['path'])==m['sha256']
assert not (NORMAL/'GUI-process-completion-observation.json').exists()
root_literal=str(ROOT).replace("'","''")
command="""[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false)
$taskRoot='ROOT_LITERAL'
$taskSnapshot=@(Get-CimInstance Win32_Process)
function Describe($p){[pscustomobject]@{pid=$p.ProcessId;name=$p.Name;created=$p.CreationDate.ToUniversalTime().ToString('o');path=$p.ExecutablePath;command=$p.CommandLine}}
$taskOriginal=@($taskSnapshot|Where-Object {$_.ProcessId -eq EXPECTED_PID}|ForEach-Object {Describe $_})
$taskOwned=@($taskSnapshot|Where-Object {
 $_.Name -in @('cargo.exe','rustc.exe') -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($taskRoot)) -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -and
 ($_.Name -match '^(LiToModern|lito_modern-|native_|qualified-client|actual-client)' -or
 ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))
}|ForEach-Object {Describe $_})
[pscustomobject]@{observed_utc=[DateTime]::UtcNow.ToString('o');original_pid_candidates=$taskOriginal;owned_workloads=$taskOwned}|ConvertTo-Json -Depth 6 -Compress
""".replace('ROOT_LITERAL',root_literal).replace('EXPECTED_PID',str(g['pipeline_pid']))
child=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert child.returncode==0 and not child.stderr,(child.returncode,child.stderr)
snapshot=json.loads(child.stdout.decode('utf-8-sig'))
assert isinstance(snapshot['original_pid_candidates'],list) and isinstance(snapshot['owned_workloads'],list)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
raw_path=NORMAL/('GUI-terminal-CIM-'+stamp+'.json');raw_path.write_bytes(child.stdout)
assert read(raw_path)==snapshot
absent=not any(p['created']==g['pipeline_creation_utc'] for p in snapshot['original_pid_candidates'])
assert absent and snapshot['owned_workloads']==[],'Original identity or owned workloads remain live; raw observation retained.'
observation={'status':'actual_tool_terminal_and_CIM_owned_process_exit_observed','observed_utc':snapshot['observed_utc'],
 'session_id':args.session_id,'terminal_exit_code':args.terminal_exit_code,'actual_terminal_tool_observed':True,
 'pipeline_pid':g['pipeline_pid'],'expected_creation_utc':g['pipeline_creation_utc'],
 'original_pipeline_identity_absent':True,'original_pid_candidates':snapshot['original_pid_candidates'],
 'owned_workloads':[],'raw_CIM_observation':str(raw_path),'raw_CIM_sha256':sha(raw_path),
 'source_files':1005,'whole_goal_complete':False,
 'observation_basis':'Calling agent observed this original unified tool handle exit before supplying its terminal exit code.'}
save(NORMAL/'GUI-process-completion-observation.json',observation)
assert read(NORMAL/'GUI-process-completion-observation.json')==observation
print(json.dumps(observation,ensure_ascii=False))
