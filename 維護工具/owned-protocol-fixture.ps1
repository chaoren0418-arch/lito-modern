function Assert-OwnedLitoProtocolFixture([string]$FixturePath,[string]$ActualOracle,[string]$Classes,[string]$OracleClass,[string]$Jdk,[string]$ReferenceJar,[string]$Output){
 $taskRoot=Split-Path -Parent $PSScriptRoot
 $fixture=Get-Content -LiteralPath $FixturePath -Raw|ConvertFrom-Json
 $current=Get-Content -LiteralPath $ActualOracle -Raw|ConvertFrom-Json
 if($current.jar_sha256 -ne (Get-FileHash -LiteralPath $ReferenceJar).Hash){throw 'Current oracle did not load the pinned reference JAR'}
 $historical=$ReferenceJar
 if($fixture.jar_sha256 -ne $current.jar_sha256){
  $historical=Join-Path $taskRoot 'LiTo381-Modern/target/current/server-reference/candidate-BBC59308.jar'
  if((Get-FileHash -LiteralPath $historical).Hash -ne $fixture.jar_sha256){throw 'Historical fixture source JAR is not retained at its declared SHA'}
 }
 $oldOracle=Join-Path $Output 'fixture-original-oracle.json'
 & (Join-Path $Jdk 'java.exe') '-Dfile.encoding=UTF-8' -cp ($Classes+';'+$historical) $OracleClass > $oldOracle 2> (Join-Path $Output 'fixture-original-oracle.stderr.txt')
 if($LASTEXITCODE -ne 0){throw 'Original fixture source oracle failed'}
 $comparison=Join-Path $Output 'fixture-current-parity.json'
 & python -X utf8 (Join-Path $PSScriptRoot 'compare-owned-protocol-fixture.py') $FixturePath $oldOracle $ActualOracle $ReferenceJar $comparison
 if($LASTEXITCODE -ne 0){throw 'Historical fixture provenance or current protocol payload mismatch'}
 return Get-Content -LiteralPath $comparison -Raw|ConvertFrom-Json
}
