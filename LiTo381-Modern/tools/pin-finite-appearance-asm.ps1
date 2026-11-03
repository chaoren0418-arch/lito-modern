param([Parameter(Mandatory)][ValidateSet('r447','r448')][string]$Stage)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$asm='D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm'
$sha=(Get-FileHash $asm).Hash;if($sha -ne '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'){throw 'Original drift'}
$text=[IO.File]::ReadAllText($asm)
$names=if($Stage -eq 'r447'){@('sub_43DE10','sub_4E45F0','sub_4E5B50','sub_4E5C00','sub_4E8750','sub_4E34C0','sub_4E02C0','sub_4E63C0','sub_4D65F0','sub_4D66F0')}else{@('sub_43DE10','sub_4E5C00','sub_4E6EA0','sub_4E0200','sub_4E7000','sub_4CD9D0','sub_483770','sub_488860','sub_4C0F80','sub_4C0FA0','sub_4C18E0','sub_4BE460','sub_4BE360')}
$root=Join-Path $project ("target\$Stage-original-finite-"+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
$pins=@(foreach($name in $names){$matches=[regex]::Matches($text,"(?ms)^$name proc near.*?^$name endp[^\r\n]*");if($matches.Count -ne 1){throw "Original function count: $name"};$p=Join-Path $root "$name.asm";[IO.File]::WriteAllText($p,$matches[0].Value,[Text.UTF8Encoding]::new($false));if(!$text.Contains([IO.File]::ReadAllText($p))){throw 'Fragment mismatch'};[ordered]@{function=$name;path=$p;sha256=(Get-FileHash $p).Hash;bytes=(Get-Item $p).Length;complete_function=$true;exact_full_asm_match=$true}})
Write-ModernEvidenceJson -Path (Join-Path $logs "$Stage-original-source-pins.json") -Value ([ordered]@{original_disassembly=[ordered]@{path=$asm;sha256=$sha;bytes=(Get-Item $asm).Length};files=$pins;original_BIN_executed=$false})
Write-Output "$Stage : $($pins.Count) whole functions pinned and exact matched"
