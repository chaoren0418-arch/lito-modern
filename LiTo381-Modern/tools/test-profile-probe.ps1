#Requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Executable,
    [Parameter(Mandatory)][string]$Java,
    [Parameter(Mandatory)][string]$ClassPath,
    [Parameter(Mandatory)][string]$Fixtures,
    [ValidateSet('Debug','Release')][string]$Mode='Debug'
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'build-evidence.ps1')
foreach($path in @($Executable,$Java,$Fixtures)) {
    if(![IO.Path]::IsPathFullyQualified($path) -or !(Test-Path -LiteralPath $path)){throw 'Explicit existing test paths required'}
}
$version=Get-ModernPlayerVersion $Executable $Mode
$exeHash=(Get-FileHash -LiteralPath $Executable).Hash
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$fixture=Join-Path $project ('target/profile-probe-cli-test-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $fixture
$rootDer=[Convert]::ToBase64String([IO.File]::ReadAllBytes((Join-Path $Fixtures 'root.der')))
$cases=[Collections.Generic.List[object]]::new()
$utf8=[Text.UTF8Encoding]::new($false)

function New-OwnedProcess([string]$File, [string[]]$Arguments) {
    $start=[Diagnostics.ProcessStartInfo]::new()
    $start.FileName=$File
    foreach($value in $Arguments){$start.ArgumentList.Add($value)}
    $start.UseShellExecute=$false
    $start.CreateNoWindow=$true
    $start.WindowStyle=[Diagnostics.ProcessWindowStyle]::Hidden
    $start.RedirectStandardOutput=$true
    $start.RedirectStandardError=$true
    $start.Environment.Remove('LITO_MODERN_PROBE_ACCOUNT')|Out-Null
    $start.Environment.Remove('LITO_MODERN_PROBE_PASSWORD')|Out-Null
    $process=[Diagnostics.Process]::new()
    $process.StartInfo=$start
    if(!$process.Start()){$process.Dispose();throw 'Cannot start owned test process'}
    $process
}
function Retire-OwnedProcess($Process) {
    if($null -eq $Process){return}
    try {
        if(!$Process.HasExited){$Process.Kill();$Process.WaitForExit()}
    } finally {$Process.Dispose()}
}
function Invoke-Probe([string[]]$Arguments,[int]$ExpectedExit,[string]$ExpectedError) {
    $process=New-OwnedProcess $Executable $Arguments
    try {
        $stdout=$process.StandardOutput.ReadToEndAsync()
        $stderr=$process.StandardError.ReadToEndAsync()
        if(!$process.WaitForExit(10000)){throw 'Owned probe deadline; interactive dialog or network stall is not a pass'}
        $output=$stdout.GetAwaiter().GetResult()
        $errors=$stderr.GetAwaiter().GetResult()
        if($process.ExitCode -ne $ExpectedExit -or $output.Length -gt 4096 -or ![string]::IsNullOrWhiteSpace($errors)){throw 'Unexpected probe process result'}
        if($output -match 'private-secret|certificate_der_base64|never-opened-probe'){throw 'Probe leaked fixture values'}
        $report=$output|ConvertFrom-Json
        if($report -is [array] -or $report.schema -ne 1 -or $report.operation -cne 'profile_probe' -or
            $report.success -ne ($ExpectedExit -eq 0) -or @($report.not_checked).Count -ne 4){throw 'Invalid probe report'}
        foreach($name in @('authentication','source_databases','shared_resources','gameplay')){
            if($name -cnotin $report.not_checked){throw 'Probe overstated its validation scope'}
        }
        if($ExpectedError -and $report.error.code -cne $ExpectedError){throw 'Wrong probe error code'}
        $report
    } finally {Retire-OwnedProcess $process}
}
function Invoke-PeerCase([string]$Mode,[bool]$Legacy,[string]$Extension,[int]$ExpectedExit) {
    $peer=New-OwnedProcess $Java @('-cp',$ClassPath,'local.lito381.server.ModernProfileProbeTestPeer','serve',$Fixtures,$Mode,'5000')
    try {
        $stderr=$peer.StandardError.ReadToEndAsync()
        $ready=$peer.StandardOutput.ReadLineAsync()
        if(!$ready.Wait(5000)){throw 'Owned peer readiness deadline'}
        $line=$ready.GetAwaiter().GetResult()
        if($line -notmatch '^READY ([1-9][0-9]*)$'){throw 'Invalid owned peer readiness'}
        $port=[int]$Matches[1]
        $profile=@{schema=$(if($Legacy){1}else{2});profile_id='probe-cli';display_name='private-secret-label';
            shared_resource_root=(Join-Path $fixture 'never-opened-probe-resources');endpoint=('127.0.0.1:'+$port);
            tls_server_name='localhost';trust=@{kind='private_ca';certificate_der_base64=$rootDer};connect_timeout_ms=1000;exchange_timeout_ms=2000}
        if(!$Legacy){$profile.gameplay_protocol='modern_v47'}
        if($Extension){$profile.gameplay_extension=$Extension}
        $file=Join-Path $fixture ('profile-'+$cases.Count+'.json')
        [IO.File]::WriteAllText($file,($profile|ConvertTo-Json -Depth 4),$utf8)
        $before=(Get-FileHash -LiteralPath $file).Hash
        $report=Invoke-Probe @('--probe-profile',$file) $ExpectedExit ''
        if((Get-FileHash -LiteralPath $file).Hash -cne $before){throw 'Probe modified its profile'}
        if($ExpectedExit -eq 0){
            if(!$report.verified_tls_and_protocol -or !$report.connection_closed -or
                $report.negotiated.version -ne $(if($Legacy){37}else{47}) -or
                $report.configured_extension -cne $(if($Extension){$Extension}else{$null})){throw 'Wrong negotiated contract'}
        } elseif($report.stage -cne 'negotiation' -or $null -ne $report.negotiated){throw 'Rejected peer was treated as negotiated'}
        if(!$peer.WaitForExit(6000) -or $peer.ExitCode -ne 0){throw 'Owned peer did not verify clean close without AUTH'}
        $closed=$peer.StandardOutput.ReadToEnd()
        if($closed.Trim() -cne ('CLOSED '+$Mode+' no-auth-or-game-data') -or
            ![string]::IsNullOrWhiteSpace($stderr.GetAwaiter().GetResult())){throw 'Peer did not confirm probe-only traffic'}
        $cases.Add([pscustomobject]@{Mode=$Mode;Legacy=$Legacy;Extension=$Extension;Exit=$ExpectedExit;NoAuth=$true;ProfileUnchanged=$true})
    } finally {Retire-OwnedProcess $peer}
}

Invoke-PeerCase 'full' $true '' 0
Invoke-PeerCase 'full' $false '' 0
Invoke-PeerCase 'tls12' $false '' 0
foreach($extension in $version.extensions){Invoke-PeerCase 'full' $false $extension 0}
Invoke-PeerCase 'legacy' $false '' 2
Invoke-PeerCase 'no-extensions' $false 'armor_presentation_v1' 2

foreach($arguments in @(@('--probe-profile'),@('--probe-profile','relative-private-secret.json'),@('--probe-profile',(Join-Path $fixture 'missing.json'),'private-secret-extra'))) {
    $null=Invoke-Probe $arguments 2 'invalid_arguments'
    $cases.Add([pscustomobject]@{Mode='invalid-arguments';Exit=2;Redacted=$true})
}
$missing=Join-Path $fixture 'missing-private-secret.json'
$null=Invoke-Probe @('--probe-profile',$missing) 2 'profile_read_failed'
$cases.Add([pscustomobject]@{Mode='missing-file';Exit=2;Redacted=$true})
$invalid=Join-Path $fixture 'invalid-private-secret.json'
[IO.File]::WriteAllText($invalid,'{"password":"private-secret"}',$utf8)
$null=Invoke-Probe @('--probe-profile',$invalid) 2 'invalid_document'
$cases.Add([pscustomobject]@{Mode='invalid-document';Exit=2;Redacted=$true})
if(Test-Path -LiteralPath (Join-Path $fixture 'never-opened-probe-resources')){throw 'Probe created resource directory'}
if((Get-FileHash -LiteralPath $Executable).Hash -cne $exeHash){throw 'Probe executable changed'}
[pscustomobject]@{Passed=$cases.Count;Cases=$cases;ExecutableSHA256=$exeHash;Fixture=$fixture;Scope=("Actual isolated $Mode EXE CLI/TLS only; no native UI, account, SQL or game-resource validation")}|ConvertTo-Json -Depth 5
