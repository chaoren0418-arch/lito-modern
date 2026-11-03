"""Read-only WinMM device inventory; opens no device and emits no sound."""
import argparse
import ctypes
from ctypes import wintypes
import datetime
import hashlib
import json
from pathlib import Path


class Caps(ctypes.Structure):
    _fields_ = [('mid', wintypes.WORD), ('pid', wintypes.WORD),
                ('driver_version', wintypes.DWORD), ('name', wintypes.WCHAR * 32),
                ('formats', wintypes.DWORD), ('channels', wintypes.WORD),
                ('reserved', wintypes.WORD), ('support', wintypes.DWORD)]


def inspect():
    winmm = ctypes.WinDLL('winmm')
    winmm.waveOutGetNumDevs.restype = wintypes.UINT
    winmm.waveOutGetDevCapsW.argtypes = [ctypes.c_size_t, ctypes.POINTER(Caps), wintypes.UINT]
    winmm.waveOutGetDevCapsW.restype = wintypes.UINT
    count = winmm.waveOutGetNumDevs()
    if count >= 64:
        raise ValueError('device inventory exceeds bound')
    rows = []
    for index in range(count):
        caps = Caps()
        result = winmm.waveOutGetDevCapsW(index, ctypes.byref(caps), ctypes.sizeof(caps))
        rows.append(dict(index=index, result=result, name=caps.name if result == 0 else None,
                         channels=caps.channels if result == 0 else None,
                         formats_hex=hex(caps.formats), support_hex=hex(caps.support)))
    return dict(status='current_host_winmm_device_inventory_only',
                observed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                devices=count, rows=rows,
                inspector_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest().upper(),
                opens_device=False, emits_sound=False,
                native_engine_audio_playback_verified=False, physical_audibility_verified=False,
                device_reset_cleanup_verified=False, full_game_complete=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    result = inspect()
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'devices': result['devices']}, ensure_ascii=False))
