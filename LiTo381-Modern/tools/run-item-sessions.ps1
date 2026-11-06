[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$ReferenceJar,
    [ValidateSet('item-heal-194','item-heal-197','item-stream','item-stream-no-reagent','item-stream-no-mana')]
    [string]$Mode='item-heal-194'
)
# Windows PowerShell 5.1. Only this process's two memory sessions are started.
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$repo=Split-Path -Parent $project
$root=Join-Path $project ('target/current/native-real-peers/issue-179-'+$Mode)
$classes=Join-Path $project 'target/current/issue-179/revision2/classes'
$jdk='C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin'
$utf8=New-Object Text.UTF8Encoding($false)
$pin='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
$ReferenceJar=(Resolve-Path -LiteralPath $ReferenceJar).Path
if((Get-FileHash -LiteralPath $ReferenceJar).Hash -ne $pin){throw 'E149 JAR identity mismatch'}
New-Item -ItemType Directory -Force $root,$classes,(Join-Path $project 'target/current/scratch') | Out-Null
# Fixed owned coordination files only, never a recursive cleanup.
foreach($name in @('ready.json','result.json','action-finished','damage-paused','a-action-endpoint','b-action-endpoint','a-rendered-1','b-rendered-1','a-error.txt','b-error.txt','a-client.json','b-client.json','summary.json')) {
    $path=Join-Path $root $name
    if(Test-Path -LiteralPath $path){Remove-Item -LiteralPath $path}
}
$env:CARGO_HOME='D:/dev-cache/cargo';$env:RUSTUP_HOME='D:/dev-cache/rustup'
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP
$env:CARGO_TERM_COLOR='never'
$common=@('--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current')
function Start-Captured([string]$exe,[string]$arguments) {
    $p=New-Object Diagnostics.Process
    $p.StartInfo.FileName=$exe;$p.StartInfo.Arguments=$arguments;$p.StartInfo.WorkingDirectory=$project
    $p.StartInfo.UseShellExecute=$false;$p.StartInfo.CreateNoWindow=$true
    $p.StartInfo.RedirectStandardOutput=$true;$p.StartInfo.RedirectStandardError=$true
    $p.StartInfo.StandardOutputEncoding=$utf8;$p.StartInfo.StandardErrorEncoding=$utf8
    $p.Start() | Out-Null
    [pscustomobject]@{Process=$p;Out=$p.StandardOutput.ReadToEndAsync();Err=$p.StandardError.ReadToEndAsync()}
}
function Finish-Captured($job,[string]$name,[int]$timeout) {
    if(!$job.Process.WaitForExit($timeout)){throw "$name timed out"}
    $log=$job.Out.GetAwaiter().GetResult()+"`n"+$job.Err.GetAwaiter().GetResult()
    [IO.File]::WriteAllText((Join-Path $root ($name+'.log')),$log,$utf8)
    [pscustomobject]@{ExitCode=$job.Process.ExitCode;Log=$log}
}
function Quote([string]$s){if($s.Contains('"')){throw 'Invalid path'};'"'+$s+'"'}
$server=$null;$client=$null;$compiler=$null
try {
    $compiler=Start-Captured (Join-Path $jdk 'javac.exe') ('-encoding UTF-8 -cp '+(Quote $ReferenceJar)+' -d '+(Quote $classes)+' tests/oracle/NativePeerHarness.java')
    $compiled=Finish-Captured $compiler 'javac' 60000
    if($compiled.ExitCode -ne 0){throw $compiled.Log}
    $compiler.Process.Dispose();$compiler=$null
    $compiler=Start-Captured 'D:/dev-cache/cargo/bin/cargo.exe' (('test '+($common -join ' '))+' --test native_item_sessions --no-run')
    $compiled=Finish-Captured $compiler 'compile' 600000
    if($compiled.ExitCode -ne 0){throw $compiled.Log}
    $env:LITO_NATIVE_PEER_ROOT=$root;$env:LITO_NATIVE_PEER_PROFILE=$Mode
    $server=Start-Captured (Join-Path $jdk 'java.exe') ('-Dfile.encoding=UTF-8 -cp '+(Quote ($classes+';'+$ReferenceJar))+' local.lito381.server.NativePeerHarness '+(Quote $root)+' '+$Mode)
    $until=[DateTime]::UtcNow.AddSeconds(120);$ready=$null
    while(!$ready) {
        if($server.Process.HasExited -or [DateTime]::UtcNow -gt $until){throw 'E149 memory harness readiness failed'}
        $path=Join-Path $root 'ready.json'
        if(Test-Path -LiteralPath $path){try{$ready=[IO.File]::ReadAllText($path,$utf8)|ConvertFrom-Json}catch{}}
        if(!$ready){Start-Sleep -Milliseconds 50}
    }
    if($ready.jar_sha256 -ne $pin){throw 'Loaded JAR differs'}
    $env:LITO_ITEM_TEST_PEER='127.0.0.1:'+$ready.port
    $client=Start-Captured 'D:/dev-cache/cargo/bin/cargo.exe' (('test '+($common -join ' '))+' --test native_item_sessions formal_item_session -- --ignored --exact --nocapture')
    $result=Finish-Captured $client 'test' 240000
    $reference=Finish-Captured $server 'server' 15000
    $tokens=$null;$errors=$null
    $ast=[Management.Automation.Language.Parser]::ParseFile((Join-Path $repo '維護工具/run-local-test-suite.ps1'),[ref]$tokens,[ref]$errors)
    if($errors.Count){throw 'Test gate parse failed'}
    $function=$ast.Find({param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -eq 'Measure-TestResult'},$true)
    Invoke-Expression $function.Extent.Text
    $gate=Measure-TestResult -Output $result.Log -ExitCode $result.ExitCode -Kind Exact
    if($reference.Log -match 'material policy read failed|policy provider unavailable'){throw 'Formal material source unavailable; refusal is not qualified'}
    $serverResult=[IO.File]::ReadAllText((Join-Path $root 'result.json'),$utf8)|ConvertFrom-Json
    $isolated=$serverResult.actual_client_sessions -eq 4 -and $serverResult.memberships_empty -and !$serverResult.sql_access -and !$serverResult.state_persistent -and $serverResult.authority_mutations -eq 0 -and $serverResult.jar_sha256 -eq $pin
    $overall=if(!$isolated){'FAIL: session-isolation'}elseif($reference.ExitCode -ne 0){'FAIL: server-exit'}else{$gate.Status}
    $summary=[ordered]@{Mode=$Mode;Jar=$pin;TestExitCode=$result.ExitCode;ServerExitCode=$reference.ExitCode;IsolationValid=$isolated;Gate=$overall;
        Result=([regex]::Matches($result.Log,'(?m)^test result:.*$')|ForEach-Object Value);Evidence=$root}
    [IO.File]::WriteAllText((Join-Path $root 'summary.json'),($summary|ConvertTo-Json),$utf8)
    $summary | ConvertTo-Json | Write-Output
    if($overall -ne 'PASS'){throw 'Formal item session gate failed; inspect evidence'}
} finally {
    # Process handles originate exclusively from Start-Captured in this runner.
    foreach($job in @($client,$server,$compiler)) {
        if($job) {
            if(!$job.Process.HasExited){$job.Process.Kill();$job.Process.WaitForExit()}
            $name=if($job -eq $server){'server-final'}elseif($job -eq $client){'test-final'}else{'compile-final'}
            [IO.File]::WriteAllText((Join-Path $root ($name+'.log')),($job.Out.GetAwaiter().GetResult()+"`n"+$job.Err.GetAwaiter().GetResult()),$utf8)
            $job.Process.Dispose()
        }
    }
}
