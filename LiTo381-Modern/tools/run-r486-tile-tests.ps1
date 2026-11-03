$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot;$logs=Join-Path $project 'logs\continuation-20261003'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r486a-lib-test-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($rows|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'lib test build incomplete'}
$artifact=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern' -and $_.profile.test -and $_.executable})
if($artifact.Count -ne 1){throw 'exact lib test artifact required'}
$root=Join-Path $project 'target\r486a-fixed-tile-tests';if(Test-Path -LiteralPath $root){throw 'test candidate already frozen'}
[IO.Directory]::CreateDirectory((Join-Path $root 'source'))|Out-Null
$exe=Join-Path $root 'r486a-lib-tests.exe';[IO.File]::Copy($artifact[0].executable,$exe,$false)
$sources=@(foreach($f in Get-ChildItem -LiteralPath (Join-Path $project 'src') -Recurse -File){$rel=[IO.Path]::GetRelativePath($project,$f.FullName);$dest=Join-Path (Join-Path $root 'source') $rel;[IO.Directory]::CreateDirectory((Split-Path -Parent $dest))|Out-Null;[IO.File]::Copy($f.FullName,$dest,$false);$pin=Pin $f.FullName;$pin.snapshot=$dest;$pin})
$names=@('variant_types_remaster_and_malformed_rle_are_not_clipped_or_guessed','supported_rle_type_only_payloads_are_truncated_not_unknown_types','inverse_brightness_rle_rejects_truncation_rectangle_skips_and_extra_bytes','cloud_brightness_uses_reference_light_gray_not_smoke_inverse_or_raw_rgb')
$runs=@(foreach($n in $names){$filter='modern_lito::tile::tests::'+$n;$out=Join-Path $logs ('r486-tile-'+$n+'.stdout.log');$err=Join-Path $logs ('r486-tile-'+$n+'.stderr.log');& $exe --exact $filter --test-threads=1 1> $out 2> $err;$code=$LASTEXITCODE;if($code -ne 0){throw ('related tile test failed: '+$n)};if((Get-Content -LiteralPath $out -Raw) -notmatch '1 passed; 0 failed'){throw 'exact one case required'};[ordered]@{filter=$filter;exit_code=$code;stdout=Pin $out;stderr=Pin $err}})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r486-tile-test-candidate.json') -Value ([ordered]@{state='R486_FIXED_LIB_TEST_CANDIDATE_FOUR_RELATED_TILE_CASES_PASS_ALL_LIB_TEST_SOURCES_COMPILED_ONLY';executable=Pin $exe;source_pins=$sources;build_json=Pin (Join-Path $logs 'r486a-lib-test-build.jsonl');build_stderr=Pin (Join-Path $logs 'r486a-lib-test-build.stderr.log');runs=$runs;test_executions=4;full_test_suite_executed=$false;PortalRecord_missing_field_compile_errors=0;original_R485_fixed_sources_unchanged=$true;GUI=0;LOADED=$false})
Write-Output 'R486 four related tile cases PASS; full library test compilation succeeded; full suite not executed'
