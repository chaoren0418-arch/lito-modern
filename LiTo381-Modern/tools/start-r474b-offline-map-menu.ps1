param([string[]]$Commands)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$m=Get-Content (Join-Path $project 'logs\continuation-20261003\r474-player-tool.json') -Raw|ConvertFrom-Json
foreach($pin in @($m.executable,$m.library,$m.binding,$m.source)){if((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256 -or (Get-Item -LiteralPath $pin.path).Length -ne $pin.bytes){throw 'R474 fixed offline tool pin drift'}}
$output=Join-Path $project ('target\r474-offline-map-flow\interactive-'+[guid]::NewGuid().ToString('N'))
Write-Host '離線地圖預覽：輸入 select 1 564，接著 enter。pan 240 120 移動視角；back 返回；quit 結束。'
Write-Host "CPU 圖像會儲存於 $output"
if($Commands){$Commands | & $m.executable.path $m.binding.path $output}else{& $m.executable.path $m.binding.path $output}
if($LASTEXITCODE -ne 0){throw 'Offline tool exited with error'}
