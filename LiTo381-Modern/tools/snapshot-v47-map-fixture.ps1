$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$handoffPath=Join-Path (Split-Path (Split-Path $project)) 'lito伺服器\Modern-map-fixture交接.json'
$handoff=Get-Content -LiteralPath $handoffPath -Raw|ConvertFrom-Json
$expectedManifest='E729F8D380604DF7C7FB1CBEC430D3F3473E2A5CA5D6FDEB068148BA78F1A60F'
if($handoff.manifest_sha256 -ne $expectedManifest -or (Get-FileHash -LiteralPath $handoff.manifest).Hash -ne $expectedManifest){throw 'Published v47 map manifest changed'}
$manifest=Get-Content -LiteralPath $handoff.manifest -Raw|ConvertFrom-Json
if(-not $manifest.immutable -or $manifest.files.Count -ne 10 -or $manifest.dependencies.Count -ne 6){throw 'Unexpected immutable fixture manifest shape'}
$destination=Join-Path $project 'target\r420-map-v47-fixture'
if(Test-Path -LiteralPath $destination){throw 'Do not overwrite immutable local fixture snapshot'}
function Check-File([string]$Path,$Entry){
    $hash=(Get-FileHash -LiteralPath $Path).Hash
    if($hash -ne $Entry.sha256 -or ($Entry.PSObject.Properties.Name -contains 'bytes' -and (Get-Item -LiteralPath $Path).Length -ne $Entry.bytes)){throw ('Published immutable input differs: '+$Path)}
}
foreach($entry in $manifest.files){Check-File (Join-Path $handoff.immutable_root $entry.relative_path) $entry}
foreach($entry in $manifest.dependencies){Check-File $entry.path $entry}
$nativeManifestPath=$manifest.dependencies[1].path
$nativeRoot=Split-Path $nativeManifestPath
$native=Get-Content -LiteralPath $nativeManifestPath -Raw|ConvertFrom-Json
if(-not $native.immutable){throw 'Candidate dependency snapshot is not immutable'}
$selected=@($native.files|Where-Object{$_.path.StartsWith('test-classes/',[StringComparison]::Ordinal) -or $_.path.StartsWith('candidate/lib/',[StringComparison]::Ordinal) -or $_.path -eq 'candidate/LiTo381-Server-candidate.jar'})
if($selected.Count -lt 1000 -or @($selected|Where-Object{$_.path -eq 'candidate/LiTo381-Server-candidate.jar'}).Count -ne 1){throw 'Incomplete candidate/test classpath dependency'}
foreach($entry in $selected){Check-File (Join-Path $nativeRoot $entry.path) $entry}
$checks=[Collections.Generic.List[object]]::new()
function Copy-Pinned([string]$Source,[string]$Target,$Entry){
    $null=New-Item -ItemType Directory -Path (Split-Path $Target) -Force
    Copy-Item -LiteralPath $Source -Destination $Target
    Check-File $Target $Entry
    $checks.Add([ordered]@{source=$Source;snapshot=$Target;sha256=$Entry.sha256;bytes=(Get-Item -LiteralPath $Target).Length})
}
foreach($entry in $manifest.files){Copy-Pinned (Join-Path $handoff.immutable_root $entry.relative_path) (Join-Path $destination ('map-prefix\'+$entry.relative_path)) $entry}
foreach($entry in $selected){Copy-Pinned (Join-Path $nativeRoot $entry.path) (Join-Path $destination ('application-snapshot\'+$entry.path)) $entry}
$junit=@(foreach($entry in $manifest.dependencies|Where-Object{$_.path.EndsWith('.jar',[StringComparison]::OrdinalIgnoreCase)}){
    $target=Join-Path $destination ('junit\'+(Split-Path $entry.path -Leaf))
    Copy-Pinned $entry.path $target $entry
    $target
})
$prior=Get-Content -LiteralPath (Join-Path $project 'logs\continuation-20261003\fixture-v3-snapshot.json') -Raw|ConvertFrom-Json
if(-not $prior.consumer_verified -or $prior.class_checks.Count -ne 14){throw 'Modern fourteen-class dependency not qualified'}
foreach($entry in $prior.class_checks){if((Get-FileHash -LiteralPath $entry.snapshot).Hash -ne $entry.sha256){throw 'Existing immutable Modern14 changed'}}
$classpath=@((Join-Path $destination 'map-prefix\classes'),$prior.classpath_prefix,(Join-Path $destination 'application-snapshot\test-classes'),(Join-Path $destination 'application-snapshot\candidate\LiTo381-Server-candidate.jar'),(Join-Path $destination 'application-snapshot\candidate\lib\*'))+$junit
$classpathPath=Join-Path $destination 'consumer-classpath.txt'
[IO.File]::WriteAllText($classpathPath,($classpath -join ';'),[Text.UTF8Encoding]::new($false))
$evidence=[ordered]@{snapshotted_at=[DateTimeOffset]::Now.ToString('o');handoff=$handoffPath;handoff_sha256=(Get-FileHash -LiteralPath $handoffPath).Hash;producer_manifest=$handoff.manifest;producer_manifest_sha256=$expectedManifest;dependency_manifest=$nativeManifestPath;dependency_manifest_sha256=$manifest.dependencies[1].sha256;local_root=$destination;file_checks=$checks.ToArray();files=$checks.Count;classpath_order=$classpath;classpath_file=$classpathPath;classpath_sha256=(Get-FileHash -LiteralPath $classpathPath).Hash;entrypoint=$handoff.entrypoint;scenarios=$handoff.scenarios;game_deadlines=$handoff.game_deadlines_unchanged;producer_scope='v47 HELLO/startup only, authentication_calls0; no consumer three rounds or GUI yet';consumer_scope='pending';server_jar_scope='Separate immutable test dependency candidate JAR only; authoritative preserved root Server JAR not replaced';prior14_manifest=(Join-Path $project 'logs\continuation-20261003\fixture-v3-snapshot.json')}
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\v47-map-fixture-snapshot.json') -Value $evidence
[pscustomobject]@{files=$checks.Count;entrypoint=$handoff.entrypoint;local_root=$destination}
