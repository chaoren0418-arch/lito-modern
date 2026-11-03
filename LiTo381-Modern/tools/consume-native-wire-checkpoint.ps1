param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$checkpointPath='D:\lito-dev-runtime\20261003-supervision\server\slice2-118f67ce278e46d7a1306bbe7eee43d8\native-resource-wire-d394ff9813a1415ca71b8e61229add9a\checkpoint.json'
$checkpointBytes=[IO.File]::ReadAllBytes($checkpointPath)
$checkpoint=[Text.Encoding]::UTF8.GetString($checkpointBytes)|ConvertFrom-Json
if((Get-FileHash -LiteralPath $checkpoint.jar).Hash -ne $checkpoint.sha256){throw 'Candidate JAR drift'}
$snapshot=Join-Path $logs ('native-wire-checkpoint-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $snapshot
[IO.File]::WriteAllBytes((Join-Path $snapshot 'checkpoint.json'),$checkpointBytes)
$javap='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe'
$declaration=@(& $javap -constants -p -classpath $checkpoint.jar 'local.lito381.server.ModernWorldProtocol' 2>&1)
if($LASTEXITCODE -ne 0 -or !($declaration -match 'MAX_FRAME_BYTES = 131072;')){throw 'Fixed candidate WORLD bound not established'}
$declarationPath=Join-Path $snapshot 'ModernWorldProtocol.javap-constants.txt'
[IO.File]::WriteAllText($declarationPath,($declaration -join "`r`n"),[Text.UTF8Encoding]::new($false))
$pins=@()
$textByName=@{}
foreach($entry in $checkpoint.sources) {
    $bytes=[IO.File]::ReadAllBytes($entry.path)
    if($bytes.Length -gt 2MB){throw 'Candidate source budget exceeded'}
    $hash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes))
    if($hash -ne $entry.sha256){throw ('Candidate source drift: '+$entry.path)}
    $name=Split-Path -Leaf $entry.path
    $copy=Join-Path $snapshot $name
    [IO.File]::WriteAllBytes($copy,$bytes)
    $pins += [ordered]@{source=$entry.path;snapshot=$copy;sha256=$hash;bytes=$bytes.Length}
    $textByName[$name]=[Text.Encoding]::UTF8.GetString($bytes)
}
function Find-Literal([string]$name,[string]$literal) {
    $lines=$textByName[$name] -split '\r?\n'
    $numbers=@(for($i=0;$i -lt $lines.Count;$i++){if($lines[$i].Contains($literal)){$i+1}})
    if(!$numbers.Count){throw "Candidate contract changed: $name $literal"}
    [ordered]@{snapshot=(Join-Path $snapshot $name);lines=$numbers;literal=$literal}
}
$evidence=@(
    (Find-Literal 'ModernWorldResourceProtocol.java' 'REQUEST_BYTES=156,PLAN_BYTES=252,RESULT_PREFIX_BYTES=288'),
    (Find-Literal 'ModernWorldResourceProtocol.java' 'ModernWorldProtocol.MAX_FRAME_BYTES+RESULT_PREFIX_BYTES'),
    (Find-Literal 'ModernWorldResourceProtocol.java' '16+RESULT_PREFIX_BYTES+108,MAX_RESULT_BYTES'),
    (Find-Literal 'ModernWorldResourceProtocol.java' 'role%2==1?16L:512L'),
    (Find-Literal 'ModernSessionExtensions.java' '(bits&255)!=0&&(base&DEPENDENCIES)!=DEPENDENCIES'),
    (Find-Literal 'ModernWorldService.java' 'v1 admits navigation only until combat proofs gain a Native envelope'),
    (Find-Literal 'ModernWorldService.java' 'Native binding rejects legacy WORLD route'),
    (Find-Literal 'ModernApplicationSession.java' 'message instanceof ModernApplicationProtocol.BattleWorldRequest'),
    (Find-Literal 'ModernApplicationSession.java' 'message instanceof ModernApplicationProtocol.NativeWorldRequest')
)
$result=[ordered]@{
    consumed_at=[DateTimeOffset]::Now.ToString('o')
    checkpoint=[ordered]@{source=$checkpointPath;snapshot=(Join-Path $snapshot 'checkpoint.json');sha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($checkpointBytes));producer_updated_at=$checkpoint.updated_at;producer_validation=$checkpoint.validation}
    candidate_jar=[ordered]@{path=$checkpoint.jar;sha256=$checkpoint.sha256;hash_matched=$true;executed_by_Modern=$false;authority_replaced=$false}
    fixed_base_class_constants=[ordered]@{tool=$javap;tool_sha256=(Get-FileHash -LiteralPath $javap).Hash;declaration=$declarationPath;declaration_sha256=(Get-FileHash -LiteralPath $declarationPath).Hash;WORLD_full_maximum=131072;class_executed=$false}
    source_pins=$pins;evidence=$evidence
    static_findings=[ordered]@{
        request_full_bytes=156;request_payload_bytes=140;result_minimum_full_bytes=412;result_maximum_full_bytes=131360
        source_plan_bytes=252;result_prefix_bytes=288;archive_pin_bytes=44
        Native_index_codec_limit_bytes=16777216;Native_pak_codec_limit_bytes=536870912
        R425_consumer_index_limit_bytes=8388608;R425_consumer_pak_limit_bytes=268435456
        budget_difference='Current offline consumer is stricter than Java record limits; all four real archives fit both. Keep consumer bounded limits explicit; do not infer arbitrary512MiB archive pixel support from codec negotiation'
        dependency_masks='Java separates Native-only mask from existing bits255 full Battle dependencies; Rust implementation pending fixed vectors'
        navigation_scope='Candidate explicitly says navigation only until combat proofs gain Native envelope; this is not canonical66/67 formal PLAYER or combat integration'
        dispatch='Java application session contains separate BattleWorldRequest and NativeWorldRequest branches; standalone86/87 envelope remains legacy WORLD body'
    }
    not_validated=@('No HEX manifest received','No cross-language encode/decode test','No candidate execution/TLS/SQL here','No Native formal PLAYER/canonical integration','No native mode addition or GUI acceptance')
    related_draft_audit=(Join-Path $logs 'native-wire-v1-compatibility-audit.json')
    qualified_modes=16;total_modes=40;gui=0
}
$output=Join-Path $logs 'native-wire-compiled-checkpoint-consumer.json'
Write-ModernEvidenceJson -Path $output -Value $result
$auditPath=$result.related_draft_audit
$audit=Get-Content -LiteralPath $auditPath -Raw|ConvertFrom-Json
$audit|Add-Member -NotePropertyName compiled_checkpoint -NotePropertyValue ([ordered]@{consumer_evidence=$output;sha256=(Get-FileHash -LiteralPath $output).Hash;request_length='Static source confirms156 full; draft should clarify payload140';result_bounds='Static source confirms412..131360; fixed cross-language vectors still pending';canonical_gap='Candidate explicitly navigation-only; formal66/67 integration not completed'}) -Force
Write-ModernEvidenceJson -Path $auditPath -Value $audit
$statusPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$status=Get-Content -LiteralPath $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o')
$status.current_task='Native wire草案與compiled checkpoint靜態核對完成，JAR/7sources pins相符；156 full/140 payload、result412..131360及分mask依賴明確。候選明列navigation-only，正式canonical66/67和戰鬥proof待協調；固定正負HEX尚未到齊。新原版user大廳觀察已獨立記錄。'
if($status.evidence_paths -notcontains $output){$status.evidence_paths=@($status.evidence_paths)+@($output)}
foreach($path in @($output,$PSCommandPath)){if($status.changed_files -notcontains $path){$status.changed_files=@($status.changed_files)+@($path)}}
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output ([ordered]@{output=$output;snapshot=$snapshot;sources=$pins.Count;static_evidence=$evidence.Count;cross_language_test=$false}|ConvertTo-Json)
