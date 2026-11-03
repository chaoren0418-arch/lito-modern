#Requires -Version 7.0
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'build-evidence.ps1')
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$fixture=Join-Path $project ('target/build-evidence-test-' + [Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $fixture
# Synthetic receipt fixtures only. No build, native player, network or SQL.
foreach($name in @('src','windows','.cargo','config','packaging','tools','target/i686-pc-windows-msvc/release')){
    $null=New-Item -ItemType Directory -Path (Join-Path $fixture $name) -Force
}
$utf8=[Text.UTF8Encoding]::new($false)
foreach($name in @('src/main.rs','windows/app.manifest','.cargo/config.toml','config/profile.example.json','packaging/profile.example.json','packaging/README.md',
    'Cargo.toml','Cargo.lock','build.rs','tools/build.ps1','tools/build-evidence.ps1','tools/package.ps1','tools/verify-independent.ps1')){
    [IO.File]::WriteAllText((Join-Path $fixture $name),'fixture content',$utf8)
}
$exe=Get-ModernPlayerPath $fixture 'Release'
[IO.File]::WriteAllText($exe,'fixture executable -- never launched',$utf8)
$inputs=@(Get-ModernBuildInputs $fixture)
$version=[pscustomobject]@{schema=1;application='LiToModern';version='fixture';build='release';status='development_candidate';profile_protocols=@('legacy_v37','modern_v47');extensions=@('player_healing_v1','armor_protection_v1','player_healing_armor_v1','armor_presentation_v1','player_healing_armor_presentation_v1');launches_native_bin=$false;launches_tw=$false}
$receipt=[pscustomobject]@{Schema=1;Mode='Release';Target='i686-pc-windows-msvc';Executable=[IO.Path]::GetRelativePath($fixture,$exe);Length=(Get-Item -LiteralPath $exe).Length;SHA256=(Get-FileHash -LiteralPath $exe).Hash;InputSHA256=(Get-ModernInputDigest $inputs);Inputs=$inputs;Version=$version}
$receiptPath=Join-Path $fixture 'target/player-release.build.json'
$originalReceipt=ConvertTo-Json -InputObject $receipt -Depth 8
function Restore-Receipt { [IO.File]::WriteAllText($receiptPath,$originalReceipt,$utf8) }
$cases=[Collections.Generic.List[string]]::new()
function Expect-Failure([string]$Name,[scriptblock]$Action,[string]$Pattern){
    $failure=$null
    try { & $Action | Out-Null } catch { $failure=$_ }
    if($null -eq $failure -or $failure.Exception.Message -notmatch $Pattern){throw "Expected rejection did not match: $Name"}
    $cases.Add($Name)
}
Expect-Failure 'missing receipt' { Assert-ModernBuildReceipt $fixture 'Release' } 'Missing successful'
Restore-Receipt
$valid=Assert-ModernBuildReceipt $fixture 'Release'
if($valid.SHA256 -cne $receipt.SHA256){throw 'Valid fixture receipt rejected'}
$cases.Add('matching inputs and executable')
$source=Join-Path $fixture 'src/main.rs'
[IO.File]::WriteAllText($source,'changed source',$utf8)
Expect-Failure 'edited source' { Assert-ModernBuildReceipt $fixture 'Release' } 'stale'
Expect-Failure 'change during compilation' { Write-ModernBuildReceipt $fixture 'Release' $inputs 'fixture.log' } 'changed while compiling'
[IO.File]::WriteAllText($source,'fixture content',$utf8)
$extra=Join-Path $fixture 'src/extra.rs'
[IO.File]::WriteAllText($extra,'extra source',$utf8)
Expect-Failure 'added source' { Assert-ModernBuildReceipt $fixture 'Release' } 'stale'
Remove-Item -LiteralPath $extra
$saved=Join-Path $fixture 'saved-main.rs'
Move-Item -LiteralPath $source -Destination $saved
Expect-Failure 'removed source' { Assert-ModernBuildReceipt $fixture 'Release' } 'stale'
Move-Item -LiteralPath $saved -Destination $source
[IO.File]::WriteAllText($exe,'changed executable',$utf8)
Expect-Failure 'replaced executable' { Assert-ModernBuildReceipt $fixture 'Release' } 'does not match executable'
[IO.File]::WriteAllText($exe,'fixture executable -- never launched',$utf8)
$receipt.Mode='TestRelease'
[IO.File]::WriteAllText($receiptPath,(ConvertTo-Json -InputObject $receipt -Depth 8),$utf8)
Expect-Failure 'test build is not player build' { Assert-ModernBuildReceipt $fixture 'Release' } 'does not match executable'
Restore-Receipt
$bad=$originalReceipt|ConvertFrom-Json
$bad.Inputs[0].SHA256='0'*64
[IO.File]::WriteAllText($receiptPath,(ConvertTo-Json -InputObject $bad -Depth 8),$utf8)
Expect-Failure 'receipt input list changed' { Assert-ModernBuildReceipt $fixture 'Release' } 'stale'
Restore-Receipt
$bad=$originalReceipt|ConvertFrom-Json
$bad.Version.profile_protocols=@('legacy_v37')
[IO.File]::WriteAllText($receiptPath,(ConvertTo-Json -InputObject $bad -Depth 8),$utf8)
Expect-Failure 'old player contract' { Assert-ModernBuildReceipt $fixture 'Release' } 'Unexpected player version'
Restore-Receipt
foreach($extensions in @(
    @('player_healing_v1'),
    @('player_healing_v1','armor_protection_v1','player_healing_armor_v1'),
    @('player_healing_v1','armor_protection_v1','player_healing_armor_v1','armor_presentation_v1','armor_presentation_v1'),
    @('player_healing_v1','armor_protection_v1','player_healing_armor_v1','armor_presentation_v1','unknown_v1'))) {
    $bad=$originalReceipt|ConvertFrom-Json
    $bad.Version.extensions=$extensions
    [IO.File]::WriteAllText($receiptPath,(ConvertTo-Json -InputObject $bad -Depth 8),$utf8)
    Expect-Failure ('incomplete or unknown extensions '+($extensions -join ',')) { Assert-ModernBuildReceipt $fixture 'Release' } 'Unexpected player version'
}
Restore-Receipt
$null=Assert-ModernBuildReceipt $fixture 'Release'
[pscustomobject]@{Passed=$cases.Count;Cases=$cases;Fixture=$fixture;ProcessesStarted=0;NetworkConnections=0}
