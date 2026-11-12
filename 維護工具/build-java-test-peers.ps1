# T036-9 (#314): compile the read-only lito-launcher-server Java sources (main + test peers)
# into THIS project's target directory, so the Java-peer / TLS ignored tests have a classpath
# without writing anything into the server repository (its own target/ stays absent).
#
# Compile only. This script never starts java.exe, never prepares TLS fixtures, never runs a
# test and never touches a database. Fixture preparation and test execution belong to the
# execution tickets (see docs/reports/issue-314.md).
#
# Output (all under <project>/LiTo381-Modern/target/java-peer/<label>/):
#   server-src/       only with -ServerRef: extracted src/main/java + src/test/java of that ref
#   classes/          server src/main/java
#   test-classes/     server src/test/java (Modern*TestPeer, *WirePeer, fixtures generators)
#   classpath.txt     value for LITO_MODERN_TEST_JAVA_CLASSPATH (single line)
#   build.json        provenance: server HEAD, dirty flag, source counts, javac version, jar SHA256
[CmdletBinding()]
param(
    [string]$ServerRoot,
    [string]$JdkHome = 'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot',
    [string]$MavenRepo = 'D:\dev-cache\maven\repository',
    [string]$Label = 'current',
    # Optional git ref of the server repository (for example origin/main). When set, src/main/java
    # and src/test/java of that ref are extracted with "git archive" into the output directory and
    # compiled from there; the server working tree is neither checked out nor modified.
    [string]$ServerRef,
    # With -ServerRef only: also extract src/main/resources and src/test/resources of that ref.
    [switch]$IncludeResources,
    [switch]$CheckOnly
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$project = Join-Path $taskRoot 'LiTo381-Modern'
if (-not (Test-Path -LiteralPath (Join-Path $project 'Cargo.toml') -PathType Leaf)) { throw 'LiTo381-Modern/Cargo.toml not found next to this script' }
if ($Label -notmatch '^[A-Za-z0-9._-]{1,64}$') { throw 'Label must be [A-Za-z0-9._-]{1,64}' }

if ([string]::IsNullOrWhiteSpace($ServerRoot)) {
    # The server repo is the parent workspace of the original checkout; a git worktree under
    # scratch/wt/<name> sits deeper, so walk up until the sibling directory is found.
    # Directory name spelled by code point so this file stays ASCII (Windows PowerShell 5.1
    # reads BOM-less scripts in the ANSI code page).
    $serverDirectory = 'lito' + [char]0x4F3A + [char]0x670D + [char]0x5668 + '\LiTo381-Server'
    $probe = $taskRoot
    for ($i = 0; $i -lt 6 -and $probe; $i++) {
        $candidate = Join-Path (Split-Path -Parent $probe) $serverDirectory
        if (Test-Path -LiteralPath (Join-Path $candidate 'pom.xml') -PathType Leaf) { $ServerRoot = $candidate; break }
        $probe = Split-Path -Parent $probe
    }
}
if ([string]::IsNullOrWhiteSpace($ServerRoot)) { throw 'Server root not found; pass -ServerRoot <...\LiTo381-Server>' }
$ServerRoot = (Resolve-Path -LiteralPath $ServerRoot).Path
foreach ($member in 'pom.xml', 'src\main\java', 'src\test\java') {
    if (-not (Test-Path -LiteralPath (Join-Path $ServerRoot $member))) { throw "Server root lacks $member" }
}

$javac = Join-Path $JdkHome 'bin\javac.exe'
if (-not (Test-Path -LiteralPath $javac -PathType Leaf)) { throw "javac not found: $javac" }
$javacVersion = (& $javac -version 2>&1 | Out-String).Trim()
if ($javacVersion -notmatch '^javac 17\.') { throw "JDK 17 required, got: $javacVersion" }

# Dependency jars: versions pinned by the server pom.xml (mysql-connector-j 8.0.33, HikariCP 5.1.0,
# junit-jupiter 5.10.3) plus their transitive jars as resolved in the local Maven repository.
$jarNames = @(
    'com\mysql\mysql-connector-j\8.0.33\mysql-connector-j-8.0.33.jar',
    'com\google\protobuf\protobuf-java\3.21.9\protobuf-java-3.21.9.jar',
    'com\zaxxer\HikariCP\5.1.0\HikariCP-5.1.0.jar',
    'org\slf4j\slf4j-api\1.7.36\slf4j-api-1.7.36.jar',
    'org\junit\jupiter\junit-jupiter-api\5.10.3\junit-jupiter-api-5.10.3.jar',
    'org\junit\jupiter\junit-jupiter-params\5.10.3\junit-jupiter-params-5.10.3.jar',
    'org\junit\jupiter\junit-jupiter-engine\5.10.3\junit-jupiter-engine-5.10.3.jar',
    'org\junit\platform\junit-platform-commons\1.10.3\junit-platform-commons-1.10.3.jar',
    'org\junit\platform\junit-platform-engine\1.10.3\junit-platform-engine-1.10.3.jar',
    'org\opentest4j\opentest4j\1.3.0\opentest4j-1.3.0.jar',
    'org\apiguardian\apiguardian-api\1.1.2\apiguardian-api-1.1.2.jar'
)
$jars = foreach ($name in $jarNames) {
    $path = Join-Path $MavenRepo $name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing dependency jar: $path" }
    $path
}

$outRoot = [IO.Path]::GetFullPath((Join-Path $project ('target\java-peer\' + $Label)))
$allowed = [IO.Path]::GetFullPath((Join-Path $project 'target')) + [IO.Path]::DirectorySeparatorChar
if (-not $outRoot.StartsWith($allowed, [StringComparison]::OrdinalIgnoreCase)) { throw 'Output must stay under LiTo381-Modern/target' }
if ($outRoot.StartsWith($ServerRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Output must not be inside the server repository' }

$serverRepo = (& git -C $ServerRoot rev-parse --show-toplevel 2>$null)
$serverHead = (& git -C $ServerRoot rev-parse HEAD 2>$null)
$serverDirty = @(& git -C $ServerRoot -c core.quotepath=false status --porcelain -- 'src' 'pom.xml' 2>$null)
$sourceCommit = $serverHead
$sourceRoot = $ServerRoot
if (-not [string]::IsNullOrWhiteSpace($ServerRef)) {
    $sourceCommit = (& git -C $ServerRoot rev-parse --verify ($ServerRef + '^{commit}') 2>$null)
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($sourceCommit)) { throw "Unknown server ref: $ServerRef" }
    $sourceRoot = Join-Path $outRoot 'server-src'
} elseif ($IncludeResources) { throw '-IncludeResources requires -ServerRef' }

function Get-JavaSources([string]$Root, [string]$Relative) {
    $directory = Join-Path $Root $Relative
    if (-not (Test-Path -LiteralPath $directory -PathType Container)) { return @() }
    @(Get-ChildItem -LiteralPath $directory -Recurse -File -Filter '*.java')
}

if ($CheckOnly) {
    [pscustomobject]@{
        ServerRoot = $ServerRoot; ServerHead = $serverHead; ServerSrcDirtyEntries = $serverDirty.Count
        ServerRef = $ServerRef; SourceCommit = $sourceCommit
        WorkingTreeMainSources = (Get-JavaSources $ServerRoot 'src\main\java').Count
        WorkingTreeTestSources = (Get-JavaSources $ServerRoot 'src\test\java').Count; Jars = $jars.Count
        Javac = $javacVersion; OutRoot = $outRoot; OutRootExists = (Test-Path -LiteralPath $outRoot)
    }
    return
}

if (Test-Path -LiteralPath $outRoot) { throw "Output already exists (pick another -Label, never overwritten): $outRoot" }
$classes = Join-Path $outRoot 'classes'
$testClasses = Join-Path $outRoot 'test-classes'
$tmp = Join-Path $outRoot 'tmp'
$null = New-Item -ItemType Directory -Path $classes, $testClasses, $tmp

if ($sourceRoot -ne $ServerRoot) {
    # Run from the server root: git archive then limits itself to that subtree and stores member
    # paths relative to it (all ASCII), so the non-ASCII workspace prefix never enters the tar.
    $null = New-Item -ItemType Directory -Path $sourceRoot
    $archive = Join-Path $outRoot 'server-src.tar'
    $members = @('src/main/java', 'src/test/java')
    if ($IncludeResources) { $members += @('src/main/resources', 'src/test/resources') }
    # Newer server commits add javac-only legacy API stubs as an extra source root (pom.xml:
    # build-helper "compile-only-legacy-stubs"); include it when that commit has one.
    & git -C $ServerRoot cat-file -e ($sourceCommit + ':./compile-stubs') 2>$null
    if ($LASTEXITCODE -eq 0) { $members += 'compile-stubs' }
    & git -C $ServerRoot archive --format=tar -o $archive $sourceCommit -- @members
    if ($LASTEXITCODE -ne 0) { throw "git archive failed with exit code $LASTEXITCODE" }
    & (Join-Path $env:SystemRoot 'System32\tar.exe') -xf $archive -C $sourceRoot
    if ($LASTEXITCODE -ne 0) { throw "tar extraction failed with exit code $LASTEXITCODE" }
    Remove-Item -LiteralPath $archive -Confirm:$false
}
$mainSources = @(Get-JavaSources $sourceRoot 'src\main\java') + @(Get-JavaSources $sourceRoot 'compile-stubs')
$testSources = @(Get-JavaSources $sourceRoot 'src\test\java')
if ($mainSources.Count -eq 0 -or $testSources.Count -eq 0) { throw 'No Java sources found' }

# javac 17 reads @argfiles in the platform code page, and the workspace path is non-ASCII.
# Run javac with the server root as working directory and list sources by relative (ASCII)
# path; absolute non-ASCII paths are only passed as direct process arguments.
function Write-ArgFile([string]$Path, [IO.FileInfo[]]$Files) {
    $lines = foreach ($file in $Files) {
        $relative = $file.FullName.Substring($sourceRoot.Length + 1)
        if ($relative -match '[^\x20-\x7E]') { throw "Non-ASCII source path: $relative" }
        '"' + $relative.Replace('\', '/') + '"'
    }
    [IO.File]::WriteAllLines($Path, [string[]]$lines, [Text.ASCIIEncoding]::new())
}
$mainArgs = Join-Path $outRoot 'main-sources.txt'
$testArgs = Join-Path $outRoot 'test-sources.txt'
Write-ArgFile $mainArgs $mainSources
Write-ArgFile $testArgs $testSources

$common = @(('-J-Djava.io.tmpdir=' + $tmp), '-J-Dfile.encoding=UTF-8', '-encoding', 'UTF-8', '-proc:none', '-nowarn', '--release', '17')
Push-Location -LiteralPath $sourceRoot
try {
    & $javac @common -cp ($jars -join ';') -d $classes ('@' + $mainArgs)
    if ($LASTEXITCODE -ne 0) { throw "javac (main) failed with exit code $LASTEXITCODE" }
    & $javac @common -cp (@($classes) + $jars -join ';') -d $testClasses ('@' + $testArgs)
    if ($LASTEXITCODE -ne 0) { throw "javac (test) failed with exit code $LASTEXITCODE" }
} finally { Pop-Location }

# Resources stay in the server repo and are referenced read-only on the classpath.
# Working-tree mode: resources stay in the server repo and are referenced read-only.
# Ref mode: only resources extracted with -IncludeResources are referenced (never the working
# tree's, which may belong to a different commit).
$resources = @(@('src\test\resources', 'src\main\resources') | ForEach-Object { Join-Path $sourceRoot $_ } | Where-Object { Test-Path -LiteralPath $_ -PathType Container })
$classpath = @($testClasses, $classes) + $resources + $jars
[IO.File]::WriteAllText((Join-Path $outRoot 'classpath.txt'), ($classpath -join ';'), [Text.UTF8Encoding]::new($false))

$provenance = [ordered]@{
    created_utc = [DateTime]::UtcNow.ToString('o')
    server_root = $ServerRoot
    server_repo = $serverRepo
    server_head = $serverHead
    server_ref = $ServerRef
    source_commit = $sourceCommit
    source_root = $sourceRoot
    resources_on_classpath = $resources
    server_src_dirty_entries = $serverDirty.Count
    javac = $javacVersion
    main_sources = $mainSources.Count
    test_sources = $testSources.Count
    main_class_files = @(Get-ChildItem -LiteralPath $classes -Recurse -File -Filter '*.class').Count
    test_class_files = @(Get-ChildItem -LiteralPath $testClasses -Recurse -File -Filter '*.class').Count
    jars = @($jars | ForEach-Object { [ordered]@{ path = $_; sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash } })
    executed_java = $false
}
[IO.File]::WriteAllText((Join-Path $outRoot 'build.json'), ($provenance | ConvertTo-Json -Depth 4), [Text.UTF8Encoding]::new($false))
[pscustomobject]@{ OutRoot = $outRoot; MainClasses = $provenance.main_class_files; TestClasses = $provenance.test_class_files; SourceCommit = $sourceCommit }
