param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$source='D:\380\天堂(Lineage 3.81C) (1)\LiTo'
$pins=@(
 @{name='LTtext.idx';bytes=9888;sha256='B8C0BB8C43AFFE59A90EBD2D26182269B6D60AF388C80963EAAEEDC8D8AE23C8'},
 @{name='LTtext.pak';bytes=292553;sha256='517D853E3FE91B504953FF4B9B2EED682A3612D063F2D18E75E39FD5580F342D'}
)
$root=Join-Path $project ('target\r429-native-catalog-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $root
foreach($pin in $pins){
 $src=Join-Path $source $pin.name
 if((Get-Item -LiteralPath $src).Length -ne $pin.bytes -or (Get-FileHash -LiteralPath $src).Hash -ne $pin.sha256){throw 'Original LTtext pin drift'}
 $dst=Join-Path $root $pin.name
 [IO.File]::Copy($src,$dst,$false)
 if((Get-FileHash -LiteralPath $src).Hash -ne $pin.sha256 -or (Get-FileHash -LiteralPath $dst).Hash -ne $pin.sha256){throw 'LTtext snapshot drift'}
}
$ref='D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收'
$loose=Join-Path $ref 'analysis\lttext-extracted-audit-0.7.593\LTlist.spr'
[IO.File]::Copy($loose,(Join-Path $root 'LTlist-reference.spr'),$false)
$asm=Join-Path $ref 'ida-client-oep-v2\full-disassembly.asm'
$pe=Join-Path $ref 'runtime\LiTo.unpacked-oep-current.exe'
if((Get-FileHash -LiteralPath $asm).Hash -ne '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A' -or (Get-FileHash -LiteralPath $pe).Hash -ne '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'){throw 'Original static evidence drift'}
$all=[IO.File]::ReadAllText($asm)
$functions=@('sub_42EA50','sub_43DDD0','sub_43DE10','sub_4406C0','sub_440010','sub_4CD9D0','sub_483770')
foreach($name in $functions){
 $m=[regex]::Match($all,'(?ms)^'+$name+' proc near\b.*?^'+$name+' endp\b')
 if(!$m.Success){throw ('Missing static function '+$name)}
 [IO.File]::WriteAllText((Join-Path $root ($name+'.asm')),$m.Value,[Text.UTF8Encoding]::new($false))
}
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r429-catalog-snapshot.json') -Value ([ordered]@{root=$root;original_root=$source;catalog_namespace='NATIVE_LITO_LTTEXT_V1';pins=$pins;reference_sha256=(Get-FileHash -LiteralPath (Join-Path $root 'LTlist-reference.spr')).Hash;reference_bytes=39135;disassembly_sha256=(Get-FileHash -LiteralPath $asm).Hash;pe_sha256=(Get-FileHash -LiteralPath $pe).Hash;static_route_functions=$functions;SourceAttachment_includes_LTtext=$false;Native_timer_ms_qualified=$false;LOADED=$false})
Write-Output $root
