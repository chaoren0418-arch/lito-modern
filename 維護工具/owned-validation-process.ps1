function Stop-OwnedLitoValidationChild([Diagnostics.Process]$Process,[string]$ScopeRoot){
 if($Process.HasExited){return}
 $taskRoot=[IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
 $scope=[IO.Path]::GetFullPath($ScopeRoot).Replace('/','\')
 if(!$scope.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned validation scope must stay in this rebuild project'}
 $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$Process.Id)
 if(!$live -and $Process.HasExited){return}
 $command=if($live){$live.CommandLine.Replace('/','\')}else{''}
 if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$Process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$command.Contains($scope)){
  throw 'Owned validation PID/creation/command mismatch; stop refused'
 }
 $Process.Kill();if(!$Process.WaitForExit(5000)){throw 'Verified owned validation child did not exit'}
}
