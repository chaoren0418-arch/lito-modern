param([Parameter(Mandatory)][ValidateSet('r449','r450','r451')][string]$Stage)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$base=Get-Content (Join-Path $logs 'r448-original-source-pins.json') -Raw|ConvertFrom-Json
$asm=$base.original_disassembly;if((Get-FileHash $asm.path).Hash -ne $asm.sha256){throw 'Original ASM drift'};$text=[IO.File]::ReadAllText($asm.path)
$names=if($Stage -eq 'r449'){@($base.files.function)+@((Get-Content (Join-Path $logs 'r447-original-source-pins.json') -Raw|ConvertFrom-Json).files.function)}else{@('sub_4E0200','sub_4E7000','sub_4CD9D0','sub_483770','sub_488860','sub_4B8FC0','sub_483A80','sub_482D90','sub_4E1410')}
$root=Join-Path $project ("target\$Stage-original-finite-"+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
$pins=@(foreach($name in $names|Select-Object -Unique){$found=[regex]::Matches($text,"(?ms)^$name proc near.*?^$name endp[^\r\n]*");if($found.Count -ne 1){throw 'Function count'};$p=Join-Path $root "$name.asm";[IO.File]::WriteAllText($p,$found[0].Value,[Text.UTF8Encoding]::new($false));if(!$text.Contains([IO.File]::ReadAllText($p))){throw 'Function copy mismatch'};[ordered]@{function=$name;path=$p;sha256=(Get-FileHash $p).Hash;bytes=(Get-Item $p).Length;complete_function=$true;exact_full_asm_match=$true}})
Write-ModernEvidenceJson -Path (Join-Path $logs "$Stage-original-source-pins.json") -Value ([ordered]@{original_disassembly=$asm;files=$pins;original_BIN_executed=$false})
Write-Output "$Stage : $($pins.Count) full original functions pinned"
