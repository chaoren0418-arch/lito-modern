function Complete-OwnedPeerValidation([string]$ReceiptPath) {
 $taskRoot=[IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
 $receipt=(Resolve-Path -LiteralPath $ReceiptPath).Path
 if(!$receipt.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Peer receipt outside owned rebuild project'}
 $folder=Split-Path -Parent $receipt;$receiptHash=(Get-FileHash -LiteralPath $receipt).Hash
 $comparisons=@(Get-ChildItem -LiteralPath $folder -Filter '*comparison.json' -File)
 if($comparisons.Count -eq 0){throw 'No independent peer comparisons'}
 $pins=[ordered]@{}
 foreach($file in $comparisons){
  $result=Get-Content -LiteralPath $file.FullName -Raw|ConvertFrom-Json
  if(!$result.receipt_sha256 -or $result.receipt_sha256 -ne $receiptHash){throw ('Comparison receipt binding mismatch: '+$file.Name)}
  $pins[$file.Name]=(Get-FileHash -LiteralPath $file.FullName).Hash
 }
 $marker=Join-Path $folder 'validation-completed.json'
 $value=[ordered]@{status='verified_owned_peer_comparison_bindings';receipt_sha256=$receiptHash;comparison_pins=$pins;binder_sha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'owned-peer-validation-binding.ps1')).Hash;runtime_receipt_rewritten=$false}
 if(Test-Path -LiteralPath $marker){
  $previous=Get-Content -LiteralPath $marker -Raw|ConvertFrom-Json -AsHashtable
  if($previous.receipt_sha256 -ne $receiptHash -or ($previous.comparison_pins|ConvertTo-Json -Compress) -ne ($pins|ConvertTo-Json -Compress)){throw 'Existing peer binding marker drift'}
 }else{$value|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $marker -Encoding utf8}
 if((Get-FileHash -LiteralPath $receipt).Hash -ne $receiptHash){throw 'Runtime receipt changed while completing validation'}
 return $marker
}
