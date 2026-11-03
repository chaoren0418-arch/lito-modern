param([switch]$UseFixtureV3)
$ErrorActionPreference='Stop'
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$fixture=Join-Path $project 'target\relocation-baseline-20261003\fixture'
$lib=Join-Path $project 'target\r409-fixed-owned-host\lito-modern-r409-host-tests.exe'
if((Get-FileHash -LiteralPath $lib).Hash -ne '2EE895F6893CD98576C7CE59F6E74CED1826FE0E673D14A1A7F726A370A55B11'){throw 'Fixed R409 operator host changed'}
$app=Join-Path $project 'target\r408-source-selector\i686-pc-windows-msvc\debug\LiToModern.exe'
if((Get-FileHash -LiteralPath $app).Hash -ne '58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'){throw 'Fixed R408 player changed'}
$runtime=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r409-showcase-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $runtime
$env:LITO_MODERN_TEST_JAVA='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe'
$env:LITO_MODERN_TEST_JAVA_CLASSPATH="$fixture\java\test-classes;$fixture\java\classes;$fixture\java\lib\*"
if($UseFixtureV3){
    $pin=Get-Content (Join-Path $project 'logs\continuation-20261003\fixture-v3-snapshot.json') -Raw|ConvertFrom-Json
    foreach($entry in $pin.class_checks){if((Get-FileHash -LiteralPath $entry.snapshot).Hash -ne $entry.sha256){throw 'Frozen Modern fixture v3 drifted'}}
    $env:LITO_MODERN_TEST_JAVA_CLASSPATH="$($pin.classpath_prefix);$($pin.dependency_glob);"+$env:LITO_MODERN_TEST_JAVA_CLASSPATH
}
$env:LITO_MODERN_TEST_SERVER_ROOT=$fixture
$env:LITO_MODERN_TLS_TEST_FIXTURES=Join-Path (Split-Path (Split-Path $project)) 'lito伺服器\LiTo381-Server\target\modern-tls-test-fixtures'
$env:LITO_MODERN_MYSQL_HOME='C:\Program Files\MySQL\MySQL Server 5.5'
$env:LITO_SHARED_RESOURCE_ROOT='D:\lineage開發\0-客戶端'
$env:LITO_EFFECT_CATALOG_PATH='D:\lineage開發\客戶端xml\effectlist2.xml'
$env:LITO_MODERN_R383_OFFICIAL_APP=$app
$env:LITO_MODERN_R383_OFFICIAL_SHA256='58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'
$env:JAVA_TOOL_OPTIONS='-Djava.io.tmpdir="'+$runtime+'"'
Write-Output "R409_RUNTIME=$runtime"
Push-Location $project
try { & $lib r407_owned_dev_showcase_operator --ignored --nocapture --test-threads=1 2>&1 | Tee-Object -FilePath (Join-Path $project 'logs\continuation-20261003\r409-showcase-operator.log'); if($LASTEXITCODE -ne 0){throw 'Owned showcase failed'} }
finally { Pop-Location }
