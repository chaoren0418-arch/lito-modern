$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot;$logs=Join-Path $project 'logs\continuation-20261003';$root=Join-Path $project 'target\r485-format-study\csharp-reference'
[IO.Directory]::CreateDirectory($root)|Out-Null
[IO.File]::Copy((Join-Path $PSScriptRoot 'r485-tile-reference.cs'),(Join-Path $root 'Program.cs'),$false)
[IO.File]::Copy((Join-Path $project 'target\r485-format-study\authority\L1Til.cs'),(Join-Path $root 'L1Til.cs'),$false)
[IO.File]::WriteAllText((Join-Path $root 'Reference.csproj'),'<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><AllowUnsafeBlocks>true</AllowUnsafeBlocks><NuGetAudit>false</NuGetAudit></PropertyGroup></Project>')
[IO.File]::WriteAllText((Join-Path $root 'NuGet.Config'),'<configuration><packageSources><clear /></packageSources></configuration>')
dotnet build (Join-Path $root 'Reference.csproj') --configfile (Join-Path $root 'NuGet.Config') --nologo 1> (Join-Path $logs 'r485-csharp-build.stdout.log') 2> (Join-Path $logs 'r485-csharp-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'Reference compile failed; source/logs retained'}
dotnet (Join-Path $root 'bin\Debug\net10.0\Reference.dll') (Join-Path $logs 'r485-tile-reference-input.json') (Join-Path $logs 'r485-tile-reference.json') 1> (Join-Path $logs 'r485-csharp.stdout.log') 2> (Join-Path $logs 'r485-csharp.stderr.log')
if($LASTEXITCODE -ne 0){throw 'Reference failed; source/logs retained'}
Get-Content -LiteralPath (Join-Path $logs 'r485-csharp.stdout.log')
