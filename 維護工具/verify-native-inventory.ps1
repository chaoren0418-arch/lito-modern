#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
. (Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1')
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-'+$BuildProfile
$outputBase=Join-Path $taskRoot '驗證紀錄/current/native-inventory'
$output=Join-Path $outputBase $stamp
$runtime=Join-Path $project ('target/current/native-inventory/'+$stamp)
$classes=Join-Path $runtime 'oracle'
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
New-Item -ItemType Directory -Path $output,$runtime,$classes -Force|Out-Null
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄\current\reference-snapshot.json') -Raw|ConvertFrom-Json
$sha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if($sha -ne $pin.sha256){throw 'Pinned JAR mismatch'}
$sourcePins=@{}
foreach($name in @('維護工具/verify-native-inventory.ps1','維護工具/owned-validation-process.ps1','維護工具/owned-protocol-fixture.ps1','維護工具/compare-owned-protocol-fixture.py','驗證紀錄/current/reference-snapshot.json','驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/source-inputs.json','LiTo381-Modern/tests/fixtures/native-inventory-current.json','LiTo381-Modern/tests/fixtures/native-inventory-icons-current.json','LiTo381-Modern/tests/oracle/NativeGameHarness.java','LiTo381-Modern/tests/oracle/NativeInventoryOracle.java')){
 $path=Join-Path $taskRoot $name;$sourcePins[$name]=(Get-FileHash -LiteralPath $path).Hash;$destination=Join-Path $output ('source/'+$name);New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force|Out-Null;Copy-Item -LiteralPath $path -Destination $destination
}
$inputPins=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
function Assert-CurrentProtocolInputs {
 foreach($name in $inputPins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $inputPins[$name]){throw ('Protocol compilation input drift: '+$name)}}
 foreach($name in $sourcePins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $sourcePins[$name]){throw ('Protocol tool/input drift: '+$name)}}
}
Assert-CurrentProtocolInputs
$mainBefore=(Get-FileHash -LiteralPath $exe).Hash
$parity=$null
function Quote([string]$text){if($text.Contains('"')){throw 'Invalid process argument'};'"'+$text+'"'}
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target\current\scratch'
New-Item -ItemType Directory -Path $scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$receipts=[System.Collections.Generic.List[object]]::new()
Push-Location $project
try {
    & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeInventoryOracle.java' 'tests/oracle/NativeGameHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Game harness compile failed'}
    & (Join-Path $jdk 'java.exe') -cp ($classes+';'+$snapshot) local.lito381.server.NativeInventoryOracle > (Join-Path $output 'oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Game oracle failed'}
    $parity=Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-inventory-current.json') -ActualOracle (Join-Path $output 'oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeInventoryOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $output
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null;$mainBefore=(Get-FileHash -LiteralPath $exe).Hash}
    foreach($mode in @('inventory','inventory-reselect')){
        $modeRoot=Join-Path $runtime $mode;New-Item -ItemType Directory -Path $modeRoot -Force|Out-Null
        $readyPath=Join-Path $modeRoot 'ready.json';$resultPath=Join-Path $modeRoot 'result.json'
        # Fixed files belong only to this isolated test; never traverse/delete a tree.
        foreach($file in @($readyPath,$resultPath)){if(Test-Path -LiteralPath $file){Remove-Item -LiteralPath $file -Force}}
        $server=$null;$client=$null
        try {
            $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-cp',(Quote ($classes+';'+$snapshot)),'local.lito381.server.NativeGameHarness',(Quote $modeRoot),$mode) -WorkingDirectory $modeRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output ($mode+'-server.log')) -RedirectStandardError (Join-Path $output ($mode+'-server.stderr.txt'))
            $until=[DateTime]::UtcNow.AddSeconds(15);$ready=$null
            while(!$ready){
                if($server.HasExited -or [DateTime]::UtcNow -ge $until){throw "Game harness failed readiness: $mode"}
                if(Test-Path -LiteralPath $readyPath){try{$ready=Get-Content -LiteralPath $readyPath -Raw|ConvertFrom-Json}catch{}}
                if(!$ready){Start-Sleep -Milliseconds 20}
            }
            if($ready.jar_sha256 -ne $sha){throw 'Session loaded wrong JAR'}
            $info=[System.Diagnostics.ProcessStartInfo]::new($exe)
            $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
            $info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
            $info.StandardInputEncoding=[System.Text.UTF8Encoding]::new($false)
            $info.StandardOutputEncoding=[System.Text.UTF8Encoding]::new($false)
            $info.ArgumentList.Add('--probe-native-lobby');$info.ArgumentList.Add('127.0.0.1:'+$ready.port)
            $client=[System.Diagnostics.Process]::Start($info)
            $outTask=$client.StandardOutput.ReadToEndAsync();$errTask=$client.StandardError.ReadToEndAsync()
            # These are local MemoryAccountRepository fixture credentials, not player credentials.
            $fixtureInput=[ordered]@{account='重建帳號';password='測試密碼';retry_password=$null;source_index=1;lobby_mode=$mode}
            $client.StandardInput.Write(($fixtureInput|ConvertTo-Json -Compress));$client.StandardInput.Close()
            if(!$client.WaitForExit(45000)){throw 'Native GAME client timed out'}
            $clientText=$outTask.GetAwaiter().GetResult();$clientText|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.json')) -Encoding utf8
            $errTask.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.stderr.txt')) -Encoding utf8
            if(!$server.WaitForExit(8000)){throw 'Deployed session did not close'}
            if($client.ExitCode -ne 0 -or $server.ExitCode -ne 0){throw "Lobby test failed $mode; inspect mode logs"}
            $response=$clientText|ConvertFrom-Json;$reference=Get-Content -LiteralPath $resultPath -Raw|ConvertFrom-Json
            if($response.game.phase -ne 'lobby_ready' -or !$response.worker_quiescent -or $response.shutdown -ne 'closed' -or $response.pointer_bits -ne 64){throw 'Inventory client did not settle'}
            $expected=if($mode -eq 'inventory-reselect'){2}else{1}
            if($response.lobby_history.Count -ne $expected -or $response.game.directory_generation -ne $expected){throw 'Inventory selection count'}
            if(!$response.inventory_shutdown_cleared){throw 'Shutdown retained inventory owner'}
            $names=$response.inventory_names
            if(!$names.worker_quiescent -or !$names.cancelled_catalog_empty -or $names.views.Count -ne $expected){throw 'Inventory name worker lifecycle'}
            if($names.sources.Count -ne 2 -or $names.sources[0].archive -cne 'Text' -or $names.sources[1].archive -cne 'LiTo/LTtext'){throw 'Inventory name namespace'}
            if($names.sources[0].sha256 -cne 'F06C94F47271FCB01F1D5AF63725DFA9A230B5F6BE3C0DD1D53CD37628ADB7DF' -or $names.sources[1].sha256 -cne '73243064A856FADE75DE110C5D25F6B1F19ED51C3BA7B1219BBE44ED7E84006B'){throw 'Inventory description source changed'}
            foreach($rows in $names.views){
                if($rows.Count -ne 2){throw 'Inventory name rows'}
                foreach($row in $rows){
                    $wire=@($reference.inventory | Where-Object { $_.id -eq $row.id })
                    if($wire.Count -ne 1 -or $row.count -ne $wire[0].count -or !$row.resolved){throw 'Inventory name identity/count'}
                    $expectedName=switch -CaseSensitive ($wire[0].name){'$26 (2)' {'治癒藥水 (2)'} 'Magic Gem (4)' {'Magic Gem (4)'} default {throw 'Unexpected name fixture'}}
                    if($row.name -cne $expectedName -or $row.text -cne ($expectedName+' × '+$wire[0].count)){throw 'Inventory translated presentation differs'}
                }
            }
            $icons=$response.inventory_icons
            $iconFixture=Get-Content -LiteralPath 'tests/fixtures/native-inventory-icons-current.json' -Raw|ConvertFrom-Json
            if(!$icons.worker_quiescent -or !$icons.cancelled_cache_empty -or $icons.views.Count -ne $expected){throw 'Inventory icon worker lifecycle'}
            foreach($rows in $icons.views){
                if($rows.Count -ne 2){throw 'Inventory icon rows'}
                foreach($row in $rows){
                    $wire=@($reference.inventory | Where-Object { $_.id -eq $row.object })
                    if($wire.Count -ne 1 -or $row.graphic -ne $wire[0].icon -or $row.icon.id -ne $row.graphic){throw 'Inventory icon identity'}
                    $pin=@($iconFixture.samples | Where-Object { $_.id -eq $row.graphic })
                    if($pin.Count -ne 1){throw 'Inventory icon fixture absent'}
                    foreach($field in @('cell_sha256','width','height','covered_pixels','covered_zero')){if($row.icon.$field -cne $pin[0].$field){throw "Inventory icon mismatch: $field"}}
                    foreach($field in @('archive','member','bytes','sha256')){if($row.icon.source.$field -cne $pin[0].source.$field){throw "Inventory icon source mismatch: $field"}}
                    if(($row.icon.offset -join ',') -cne ($pin[0].offset -join ',')){throw 'Inventory icon offset mismatch'}
                }
            }
            if($expected -eq 2 -and !$response.inventory_return_cleared){throw 'Return retained inventory owner'}
            $priorSelection=0
            foreach($view in $response.lobby_history){
                $inv=$view.inventory
                if($inv.owner.connection -ne 2 -or $inv.owner.selection -le $priorSelection -or $inv.owner.character -ne $view.selected_character -or $inv.batches -ne 1){throw 'Inventory character ownership'}
                $priorSelection=$inv.owner.selection
                if(@($inv.items.PSObject.Properties).Count -ne 2){throw 'Inventory rows missing or duplicated'}
                foreach($item in $reference.inventory){
                    $actual=$inv.items.PSObject.Properties[[string]$item.id].Value
                    if($null -eq $actual){throw 'Invented inventory object identity'}
                    foreach($field in @('id','use_type','charge','icon','blessing','count','identified','name')){if($actual.$field -cne $item.$field){throw "Inventory authority differs: $field"}}
                    if(($actual.detail -join ',') -ne ($item.detail -join ',')){throw 'Inventory detail changed'}
                }
            }
            if($expected -eq 2 -and $response.game.channel.id -ne 3){throw 'Reselection did not return to lobby'}
            if($reference.status -ne 'verified' -or $reference.jar_sha256 -ne $sha -or !$reference.actual_client_sessions -or $reference.sql_access -or $reference.state_persistent -or (Test-Path -LiteralPath (Join-Path $modeRoot 'state.properties'))){throw 'Isolation/session evidence failed'}
            $receipts.Add([ordered]@{mode=$mode;client=$response;server=$reference;verified=$true})
            Write-Output "$mode native inventory bootstrap/owner verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-OwnedLitoValidationChild -Process $owned -ScopeRoot $project};$owned.Dispose()}}
        }
    }
    Assert-CurrentProtocolInputs
    if((Get-FileHash -LiteralPath $exe).Hash -ne $mainBefore -or (Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Protocol main/current-reference drift'}
    [ordered]@{schema=2;profile=$BuildProfile;output=$output;runtime=$runtime;immutable_receipt=(Join-Path $output 'receipt.json');source_pins=$sourcePins;compilation_inputs_checked=$inputPins.Count;fixture_parity=$parity;status='verified_native_character_inventory';jar_sha256=$sha;exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;cases=$receipts;sql_access=$false;game_endpoint_connected=$true;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8
    $canonical=Join-Path $outputBase 'receipt.json';if(Test-Path -LiteralPath $canonical){Copy-Item -LiteralPath $canonical -Destination (Join-Path $output 'previous-canonical-receipt.json')};Copy-Item -LiteralPath (Join-Path $output 'receipt.json') -Destination $canonical
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
