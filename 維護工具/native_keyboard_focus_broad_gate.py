"""Reject other current canonical workloads before current broad runtime."""
from pathlib import Path
from datetime import datetime
import json, subprocess, time

SOURCE = 'AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D'

def read(path):
    for attempt in range(10):
        try:
            return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt == 9:
                raise
            time.sleep(.05)

def require_idle_runtime(root):
    queue = read(root/'工作佇列.json')
    assert queue['current_product_identity']['source_manifest_sha256'] == SOURCE
    active = queue.get('active_workloads', {})
    assert set(active) <= {'broad_default'}, 'another current workload is reserved; preserve it'
    if not active:
        assert not queue.get('source_and_corresponding_QA_frozen'), 'current source is reserved for another verification'
        return 'no_current_reserved_workloads'
    record = active['broad_default']
    assert record['source_manifest_sha256'] == SOURCE
    run = Path(record['run']).resolve(strict=True)
    assert run.is_relative_to(root/'驗證紀錄/current/native-keyboard-focus-default116')
    cohort = read(run/'cohort.json')
    assert cohort['status'] in ['executing_current_default116_profile','independently_reading_current_default116_profile']
    pid = cohort['pipeline_pid']
    command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
    result = subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    assert result.returncode == 0 and not result.stderr
    live = json.loads(result.stdout.decode('utf-8-sig'))
    assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(cohort['pipeline_creation_utc'])).total_seconds()) < .001
    return 'only_current_original_broad_pipeline_reserved'
