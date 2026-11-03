# Local operator-only showcase. The sole player executable remains LiToModern.exe.
# Run in an interactive PowerShell terminal; type "stop" here after closing clients.
$ErrorActionPreference = 'Stop'
$project = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$litoRoot = (Resolve-Path -LiteralPath (Join-Path $project '..\..')).Path
$workspace = (Resolve-Path -LiteralPath (Join-Path $litoRoot '..')).Path
$server = (Resolve-Path -LiteralPath (Join-Path $litoRoot 'lito伺服器\LiTo381-Server')).Path
$hostRoot = Join-Path $project 'target\r407-dev-showcase-host'
$hostTest = Join-Path $hostRoot 'lito-modern-dev-host-tests-r407a.exe'
$player = Join-Path $project 'target\r405-vip-world-preview-revised\i686-pc-windows-msvc\debug\LiToModern.exe'
$java = 'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe'
$mysql = 'C:\Program Files\MySQL\MySQL Server 5.5'
$shared = (Resolve-Path -LiteralPath (Join-Path $workspace '0-客戶端')).Path
$effects = (Resolve-Path -LiteralPath (Join-Path $workspace '客戶端xml\effectlist2.xml')).Path
$tls = Join-Path $server 'target\modern-tls-test-fixtures'
$classRoot = Join-Path $hostRoot 'java'
$required = @($hostTest, $player, $java, $mysql, $tls, $effects,
    (Join-Path $classRoot 'test-classes'), (Join-Path $classRoot 'classes'),
    (Join-Path $classRoot 'lib'), (Join-Path $server 'db\modern_match_settlement.sql'))
foreach ($path in $required) {
    if (-not (Test-Path -LiteralPath $path)) { throw "Missing dev showcase dependency: $path" }
}
$hostHash = (Get-FileHash -LiteralPath $hostTest -Algorithm SHA256).Hash
$playerHash = (Get-FileHash -LiteralPath $player -Algorithm SHA256).Hash
if ($hostHash -ne 'A0840EEC3E573C5804230A132D4EFF451D87C7C6845514D317349220E1E07DC8') {
    throw 'R407 owned host executable changed; do not mix builds.'
}
if ($playerHash -ne '6994E91E8B3F368CAC9C486DA783E76DAFD48743D53C7D6D7609065DBC07A56C') {
    throw 'R405 player executable changed; do not mix builds.'
}
$env:LITO_MODERN_TEST_JAVA = $java
$env:LITO_MODERN_TEST_JAVA_CLASSPATH = "$(Join-Path $classRoot 'test-classes');$(Join-Path $classRoot 'classes');$(Join-Path $classRoot 'lib\*')"
$env:LITO_MODERN_TEST_SERVER_ROOT = $server
$env:LITO_MODERN_TLS_TEST_FIXTURES = $tls
$env:LITO_MODERN_MYSQL_HOME = $mysql
$env:LITO_SHARED_RESOURCE_ROOT = $shared
$env:LITO_EFFECT_CATALOG_PATH = $effects
$env:LITO_MODERN_R383_OFFICIAL_APP = $player
$env:LITO_MODERN_R383_OFFICIAL_SHA256 = $playerHash
Write-Host 'Starting owned local TLS/SQL/Map514 development host; it does not launch a client.'
Write-Host 'Wait for R407_DEV_READY, then use the printed R407_OFFICIAL_PROFILE with LiToModern.exe --profile.'
Write-Host 'When finished, close the client normally and type stop in this terminal.'
Push-Location -LiteralPath $project
try {
    & $hostTest r407_owned_dev_showcase_operator --ignored --nocapture --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw "Owned dev showcase exited with code $LASTEXITCODE" }
} finally {
    Pop-Location
}
