#Requires -Version 7.0
[CmdletBinding()]
param(
    [ValidatePattern('^r542(?:-[a-z0-9-]+)?$')][string]$EvidenceName='r542',
    [switch]$CompileOnly
)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$logs=Join-Path $project 'logs/continuation-20261003'
$output=Join-Path $project ("target/$EvidenceName-native-backend-candidate")
$existingCache=Join-Path $project 'target/r409-dev-ready'
$cache=if($CompileOnly){Join-Path $output 'cargo-cache'}else{$existingCache}
$mode=if($CompileOnly){'compile-only'}else{'compile-and-cli-tests'}
$binary='LiToModern-485F-Candidate'
function Pin([string]$Path){[ordered]@{path=[IO.Path]::GetFullPath($Path);bytes=(Get-Item -LiteralPath $Path).Length;sha256=(Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash}}
function Save([string]$Path,$Value){
    $stream=[IO.File]::Open($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
    try {$raw=[Text.Encoding]::UTF8.GetBytes((ConvertTo-Json -InputObject $Value -Depth 12)+"`n");$stream.Write($raw,0,$raw.Length)}finally{$stream.Dispose()}
}
function AssertPin($Expected){$actual=Pin $Expected.path;if($actual.bytes -ne $Expected.bytes -or $actual.sha256 -cne $Expected.sha256){throw "Pin drift: $($Expected.path)"}}
function Get-CandidateInputs {
    $inputs=@(Get-ModernBuildInputs $project)
    foreach($relative in @('tools/build-r542-native-candidate.ps1','docs/native-backend-485f-candidate.md','README.md')){
        $item=Get-Item -LiteralPath (Join-Path $project $relative)
        if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Reparse point in candidate inputs'}
        $inputs+=[pscustomobject]@{Path=[IO.Path]::GetRelativePath($project,$item.FullName);Length=$item.Length;SHA256=(Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash}
    }
    $inputs|Sort-Object Path -Unique
}
function Seed-DependencyCache {
    # Copies only: prior cache paths can be immutable evidence. Never hardlink or
    # modify them. Omit this package and all incremental/test package outputs.
    $root=Get-Item -LiteralPath $existingCache
    if(!$root.PSIsContainer -or ($root.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Unsafe dependency cache root'}
    $items=@(Get-ChildItem -LiteralPath $existingCache -Recurse -Force)
    if(@($items|Where-Object{$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'Reparse point in dependency cache'}
    $count=0;$bytes=[long]0
    foreach($item in @($items|Where-Object{!$_.PSIsContainer}|Sort-Object FullName)){
        $relative=[IO.Path]::GetRelativePath($existingCache,$item.FullName)
        $portable=$relative.Replace('\','/')
        if($portable -notmatch '^(?:debug|i686-pc-windows-msvc/debug)/(?:\.fingerprint|build|deps)/' -and $portable -notin @('.rustc_info.json','CACHEDIR.TAG')){continue}
        if($portable -match '(?:^|/)lito-modern-' -or $item.Name -match '^(?:liblito_modern|lito_modern|LiToModern)' -or $item.Name -eq '.cargo-lock'){continue}
        $to=Join-Path $cache $relative
        $null=[IO.Directory]::CreateDirectory((Split-Path -Parent $to))
        [IO.File]::Copy($item.FullName,$to,$false)
        [IO.File]::SetLastWriteTimeUtc($to,$item.LastWriteTimeUtc)
        $count++;$bytes+=$item.Length
    }
    Save (Join-Path $logs "$EvidenceName-cache-seed.json") ([ordered]@{
        source=$existingCache;destination=$cache;files=$count;bytes=$bytes;copies_only=$true;source_cache_writes=0;
        package_artifacts_copied=0;incremental_copied=0;timestamps_preserved=$true
    })
}
# No claim/READY/provider access. These are the same two immutable resource manifests used by R532.
$resources=@(
    [ordered]@{path=(Join-Path $logs 'r486-map-binding.json');bytes=208039;sha256='14805BC5D26A88EEAAE3B182A4A477D30AE7B0F26212B760F1CFA29B452DCA6E'},
    [ordered]@{path=(Join-Path $logs 'r506-native-menu-skin-binding.json');bytes=37911;sha256='27475914E201922FF0661EDA513B85E73C3E2CBF33FECB3CE71C7345CEF9D379'}
)
foreach($resource in $resources){AssertPin $resource}
if(Test-Path -LiteralPath $output){throw 'Evidence output exists; preserve it and select a fresh -EvidenceName'}
foreach($suffix in @('build.jsonl','build.stderr.log','test-build.jsonl','test-build.stderr.log','tests.stdout.log','tests.stderr.log','build-accounting.json','test-build-accounting.json','test-accounting.json','candidate.json','source-snapshot.json','source-drift.json','cache-seed.json','failure.json')){
    if(Test-Path -LiteralPath (Join-Path $logs "$EvidenceName-$suffix")){throw "Evidence log exists: $suffix"}
}
. (Join-Path $PSScriptRoot 'build-evidence.ps1')
$lease=[IO.File]::Open((Join-Path $project 'target/project-lease.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$stage='source-snapshot';$beforeDigest=$null;$buildAttempts=0
try {
if(Test-Path -LiteralPath $output){throw 'Evidence output collision after build lock; stop'}
$before=@(Get-CandidateInputs)
$beforeDigest=Get-ModernInputDigest $before
$null=[IO.Directory]::CreateDirectory($output)
$sourceRoot=Join-Path $output 'source'
$sourcePins=@()
foreach($inputPin in $before){
    $from=Join-Path $project $inputPin.Path
    $to=Join-Path $sourceRoot $inputPin.Path
    $null=[IO.Directory]::CreateDirectory((Split-Path -Parent $to))
    [IO.File]::Copy($from,$to,$false)
    AssertPin ([ordered]@{path=$to;bytes=$inputPin.Length;sha256=$inputPin.SHA256})
    $sourcePins+=([ordered]@{path=$inputPin.Path;bytes=$inputPin.Length;sha256=$inputPin.SHA256;snapshot=Pin $to})
}
Save (Join-Path $logs "$EvidenceName-source-snapshot.json") ([ordered]@{mode=$mode;input_digest=$beforeDigest;inputs=$sourcePins;source_count=@($before|Where-Object{$_.Path -like 'src*' -and $_.Path -like '*.rs'}).Count})
$scratch=Join-Path (Join-Path $project 'target/build-scratch') ("$EvidenceName-native-build-"+[guid]::NewGuid().ToString('N'))
$null=[IO.Directory]::CreateDirectory($scratch)
$env:TEMP=$scratch;$env:TMP=$scratch
    if($CompileOnly){
        $stage='dependency-cache-copy'
        Seed-DependencyCache
        Save (Join-Path $logs "$EvidenceName-test-build-accounting.json") ([ordered]@{status='NOT RUN';reason='explicit -CompileOnly';attempts=0;exit_code=$null;tests_compiled=0;command=@()})
        Save (Join-Path $logs "$EvidenceName-test-accounting.json") ([ordered]@{status='NOT RUN';reason='explicit -CompileOnly';attempts=0;exit_code=$null;tests_executed=0;old_PASS_replays=0;candidate_executions=0;command=@()})
    }
    $stage='pre-build-source-check'
    $preBuildDigest=Get-ModernInputDigest @(Get-CandidateInputs)
    if($beforeDigest -cne $preBuildDigest){throw 'Build inputs changed before Cargo; no build attempted'}
    foreach($resource in $resources){AssertPin $resource}
    $cargoVersion=(& cargo --version|Out-String).Trim()
    if($LASTEXITCODE -ne 0 -or ($CompileOnly -and $cargoVersion -notmatch '^cargo 1\.95\.0 ')){throw 'Required existing Cargo 1.95.0 unavailable; no build attempted'}
    $baseArgs=@('--offline','--locked','--manifest-path',(Join-Path $project 'Cargo.toml'),'--target','i686-pc-windows-msvc','--target-dir',$cache,'--bin',$binary)
    $stage='cargo-build';$buildAttempts=1
    & cargo build @baseArgs --verbose --message-format=json 1>(Join-Path $logs "$EvidenceName-build.jsonl") 2>(Join-Path $logs "$EvidenceName-build.stderr.log")
    $code=$LASTEXITCODE
    Save (Join-Path $logs "$EvidenceName-build-accounting.json") ([ordered]@{mode=$mode;attempts=1;exit_code=$code;command=@('cargo','build')+$baseArgs+@('--verbose','--message-format=json');cargo_version=$cargoVersion;binary=$binary;target='i686-pc-windows-msvc';scratch_retained=$scratch;source_digest=$beforeDigest;candidate_executions=0;GUI=0;network=0;JavaSQL=0;providers=0;READY_reads=0;claims=0;authority_calls=0;old_PASS_replays=0})
    $stage='post-build-source-check'
    $afterDigest=Get-ModernInputDigest @(Get-CandidateInputs)
    Save (Join-Path $logs "$EvidenceName-source-drift.json") ([ordered]@{before=$beforeDigest;immediately_before_cargo=$preBuildDigest;after_cargo=$afterDigest;unchanged=($beforeDigest -ceq $afterDigest)})
    if($beforeDigest -cne $afterDigest){throw 'Build inputs changed during Cargo; no candidate acceptance receipt'}
    foreach($resource in $resources){AssertPin $resource}
    if($code -ne 0){throw 'Candidate build failed; initial logs and source retained'}
    $stage='artifact-identification'
    $rows=@(Get-Content -LiteralPath (Join-Path $logs "$EvidenceName-build.jsonl") -Encoding UTF8|ForEach-Object{$_|ConvertFrom-Json})
    if($CompileOnly -and @($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.profile.test}).Count){throw 'Unexpected test artifact in compile-only Cargo output'}
    $binRows=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $binary -and !$_.profile.test -and $_.target.src_path -ceq (Join-Path $project 'src/bin/native_backend_485f_candidate.rs')})
    $libRows=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern' -and !$_.profile.test -and $_.target.src_path -ceq (Join-Path $project 'src/lib.rs')})
    if($binRows.Count -ne 1 -or $libRows.Count -ne 1){throw 'Exact Cargo binary and linked library artifact records required'}
    if($binRows[0].package_id -cne $libRows[0].package_id -or $binRows[0].package_id -notmatch '#lito-modern@0\.1\.0$'){throw 'Unexpected Cargo package identity'}
    $rlibs=@($libRows[0].filenames|Where-Object{$_ -like '*.rlib' -and $_ -like '*i686-pc-windows-msvc*'})
    if($rlibs.Count -ne 1){throw 'Exact linked target rlib required'}
    $candidateLinkCommand=$null
    if($CompileOnly){
        if($libRows[0].fresh -or $binRows[0].fresh){throw 'Current package must compile in the fresh target directory'}
        $crateName=$binary.Replace('-','_')
        $linkCommands=@(Get-Content -LiteralPath (Join-Path $logs "$EvidenceName-build.stderr.log") -Encoding UTF8|Where-Object{
            $_.Contains("--crate-name $crateName ") -and $_.Contains("lito_modern=$($rlibs[0])")
        })
        if($linkCommands.Count -ne 1){throw 'Exact candidate rustc --extern linked library command required'}
        $candidateLinkCommand=$linkCommands[0]
    }
    $artifactExePin=Pin $binRows[0].executable;$artifactLibPin=Pin $rlibs[0]
    $exe=Join-Path $output 'LiToModern-485F-Candidate-R542.exe'
    [IO.File]::Copy($binRows[0].executable,$exe,$false)
    $libRoot=Join-Path $output 'library';$null=[IO.Directory]::CreateDirectory($libRoot)
    $lib=Join-Path $libRoot 'liblito_modern.rlib';[IO.File]::Copy($rlibs[0],$lib,$false)
    AssertPin ([ordered]@{path=$exe;bytes=$artifactExePin.bytes;sha256=$artifactExePin.sha256})
    AssertPin ([ordered]@{path=$lib;bytes=$artifactLibPin.bytes;sha256=$artifactLibPin.sha256})
    if($CompileOnly){
        $stage='compile-only-receipt'
        if($beforeDigest -cne (Get-ModernInputDigest @(Get-CandidateInputs))){throw 'Build inputs changed while freezing outputs'}
        Save (Join-Path $logs "$EvidenceName-candidate.json") ([ordered]@{
            status='CURRENT_SOURCE_CANDIDATE_COMPILED_ONLY_REVIEW';mode=$mode;task_state='review';product_accepted=$false;
            input_digest=$beforeDigest;executable=Pin $exe;linked_library=Pin $lib;Cargo_linked_library=$artifactLibPin;
            library_fresh=$libRows[0].fresh;binary_fresh=$binRows[0].fresh;compiler_artifacts=@($libRows[0],$binRows[0]);
            candidate_rustc_link_command=$candidateLinkCommand;
            binary_name=$binary;binary_source=Pin (Join-Path $project 'src/bin/native_backend_485f_candidate.rs');default_main=Pin (Join-Path $project 'src/main.rs');cargo_manifest=Pin (Join-Path $project 'Cargo.toml');cargo_lock=Pin (Join-Path $project 'Cargo.lock');resources=$resources;
            build_command=@('cargo','build')+$baseArgs+@('--verbose','--message-format=json');test_compile_status='NOT RUN';tests_status='NOT RUN';test_compile_command=@();test_command=@();tests_compiled=0;tests_executed=0;
            custom_resource_env_required=$false;resource_project_root=$project;candidate_executions=0;GUI=0;network=0;JavaSQL=0;providers=0;READY_reads=0;claims=0;authority_calls=0;old_PASS_replays=0;default_run='LiToModern';published=$false
        })
        Write-Output 'Current-source candidate frozen for review; tests NOT RUN; no candidate launch or READY claim.'
        return
    }
    $stage='normal-mode-cli-tests'
    & cargo test @baseArgs --no-run --message-format=json 1>(Join-Path $logs "$EvidenceName-test-build.jsonl") 2>(Join-Path $logs "$EvidenceName-test-build.stderr.log")
    $code=$LASTEXITCODE
    Save (Join-Path $logs "$EvidenceName-test-build-accounting.json") ([ordered]@{attempts=1;exit_code=$code;binary=$binary;tests_executed=0;old_PASS_replays=0})
    if($code -ne 0){throw 'New candidate CPU test compile failed; preserve initial outcome'}
    $testRows=@(Get-Content -LiteralPath (Join-Path $logs "$EvidenceName-test-build.jsonl") -Encoding UTF8|ForEach-Object{$_|ConvertFrom-Json})
    $testBins=@($testRows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $binary -and $_.profile.test})
    if($testBins.Count -ne 1){throw 'Exact candidate unit-test artifact required'}
    $test=Join-Path $output 'r542-new-cli-tests.exe';[IO.File]::Copy($testBins[0].executable,$test,$false)
    & $test r542_candidate_checks --nocapture 1>(Join-Path $logs "$EvidenceName-tests.stdout.log") 2>(Join-Path $logs "$EvidenceName-tests.stderr.log")
    $code=$LASTEXITCODE
    Save (Join-Path $logs "$EvidenceName-test-accounting.json") ([ordered]@{attempts=1;exit_code=$code;filter='r542_candidate_checks';new_tests_expected=2;old_tests_executed=0;old_PASS_replays=0;Accepted_load_calls=0;READY_reads=0;claims=0;network=0;GUI=0;JavaSQL=0})
    if($code -ne 0){throw 'New CLI CPU checks failed; preserve result, do not replay passed checks'}
    $after=@(Get-CandidateInputs)
    if($beforeDigest -cne (Get-ModernInputDigest $after)){throw 'Build inputs changed during candidate build'}
    Save (Join-Path $logs "$EvidenceName-candidate.json") ([ordered]@{
        status='REPOSITORY_EXPLICIT_485F_CANDIDATE_COMPILED_CPU_CLI_ONLY';executable=Pin $exe;linked_library=Pin $lib;Cargo_linked_library=Pin $rlibs[0];library_fresh=$libRows[0].fresh;test_executable=Pin $test;
        binary_name=$binary;binary_source=Pin (Join-Path $project 'src/bin/native_backend_485f_candidate.rs');default_main=Pin (Join-Path $project 'src/main.rs');cargo_manifest=Pin (Join-Path $project 'Cargo.toml');resources=$resources;
        build_command=@('cargo','build')+$baseArgs+@('--verbose','--message-format=json');test_compile_command=@('cargo','test')+$baseArgs+@('--no-run');test_command=@($test,'r542_candidate_checks','--nocapture');
        custom_resource_env_required=$false;resource_project_root=$project;new_CPU_tests=2;candidate_executions=0;GUI=0;network=0;JavaSQL=0;READY_reads=0;claims=0;old_PASS_replays=0;default_run='LiToModern';published=$false
    })
    Write-Output "R542 named candidate frozen; two new CLI CPU checks only, no candidate launch or READY claim."
} catch {
    $failure=[ordered]@{status='blocked';stage=$stage;message=$_.Exception.Message;build_attempts=$buildAttempts;source_digest_before=$beforeDigest;retry_performed=$false;
        missing_input='Resolution of the preserved first failure at the recorded stage';supplier='Modern build owner / controller review';resume_when='A concrete explanation and new authorized correction resolve the recorded failure; no identical retry';consumer='modern-current-native-candidate-compile-20261007'}
    $failurePath=Join-Path $logs "$EvidenceName-failure.json"
    if(!(Test-Path -LiteralPath $failurePath)){Save $failurePath $failure}
    throw
} finally {$lease.Dispose();$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
