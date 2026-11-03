function Write-ModernEvidenceJson {
    param([Parameter(Mandatory)][string]$Path,[Parameter(Mandatory)]$Value)
    $temporary=$Path+'.'+[Guid]::NewGuid().ToString('N')+'.tmp'
    try {
        [IO.File]::WriteAllText($temporary,(ConvertTo-Json -InputObject $Value -Depth 12),[Text.UTF8Encoding]::new($false))
        for($attempt=0;$attempt -lt 50;$attempt++) {
            try { [IO.File]::Move($temporary,$Path,$true); return }
            catch [IO.IOException] { if($attempt -eq 49){throw}; Start-Sleep -Milliseconds 50 }
        }
    } finally { if(Test-Path -LiteralPath $temporary){Remove-Item -LiteralPath $temporary} }
}
