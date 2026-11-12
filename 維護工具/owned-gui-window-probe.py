"""輸入／擷取前置檢查：確認「自己啟動的」客戶端視窗能不能被擷取、點擊與接收鍵盤輸入。

背景（T035 第 2 項，HISTORY L1411–L1417、NATIVE-LOGIN:14）：GUI 自動化曾回報
`FrameArrived timed out`、`coordinate input geometry is unavailable`，而且控制器提前
收尾時沒有留下 result.json。視窗最小化、隱藏、被 cloak、落在螢幕外，或輸入桌面不可用時，
都會出現這兩種錯誤。本工具在送出任何輸入之前先把這些狀態查出來，並且不論成功、失敗或
例外，都一定寫出 result.json。

本工具只做三件事：
  1. 核對目標程序是呼叫者自己啟動的（PID＋建立時間＋exe 位於本專案內）。
  2. 唯讀列出該程序的最上層視窗狀態，判定 capture／click／keyboard 是否就緒。
  3. 指定 --recover 時，只對該程序自己的主視窗做：還原最小化、移回工作區、嘗試設為前景。

本工具不送鍵盤或滑鼠輸入、不擷取畫面、不啟動或結束任何程序、不改系統設定；
status=ready 只代表「可以開始擷取與輸入」，不是 GUI 驗收通過。

用法：
  python -X utf8 維護工具/owned-gui-window-probe.py --pid 1234 \
      --creation-utc 2026-11-12T09:00:00.123456+00:00 \
      --output scratch/gui-probe/case-1 [--recover] [--wait-seconds 15]
  python -X utf8 維護工具/owned-gui-window-probe.py --self-test    # 純邏輯，不碰視窗

結束碼：0 就緒（含恢復後就緒）；2 未就緒；3 程序不存在或身分不符；1 其他錯誤。
"""
from datetime import datetime, timedelta, timezone
from pathlib import Path
import argparse
import json
import os
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = 'owned-gui-window-probe/1'
OUTPUT_ROOTS = ('scratch', 'LiTo381-Modern/target')
WS_CAPTION = 0x00C00000
SW_RESTORE = 9
SWP_NOZORDER, SWP_NOACTIVATE = 0x0004, 0x0010
EXIT_READY, EXIT_ERROR, EXIT_NOT_READY, EXIT_IDENTITY = 0, 1, 2, 3


# ---------------------------------------------------------------- 純邏輯
def fit_to_work_area(window, work):
    """與 src/modern_lito/window_presentation.rs 的 fit_to_work_area 相同。"""
    left, top, right, bottom = window
    work_left, work_top, work_right, work_bottom = work
    width, height = right - left, bottom - top
    work_width, work_height = work_right - work_left, work_bottom - work_top
    if width <= 0 or height <= 0 or work_width <= 0 or work_height <= 0:
        return None
    width, height = min(width, work_width), min(height, work_height)
    x = min(max(left, work_left), work_right - width)
    y = min(max(top, work_top), work_bottom - height)
    fitted = (x, y, x + width, y + height)
    return None if fitted == tuple(window) else fitted


def inside(inner, outer):
    return inner[0] >= outer[0] and inner[1] >= outer[1] and inner[2] <= outer[2] and inner[3] <= outer[3]


def assess(window, environment):
    """回傳 (capture_ready, click_geometry_ready, keyboard_input_ready, reasons)。

    window 為 None 表示找不到主視窗。reasons 只列觀測到的狀態，不推測原因。
    """
    reasons = []
    if not environment.get('input_desktop'):
        reasons.append('input_desktop_unavailable')
    if window is None:
        return False, False, False, reasons + ['no_main_window']
    if not window['visible']:
        reasons.append('hidden')
    if window['minimized']:
        reasons.append('minimized')
    if window['cloaked']:
        reasons.append('cloaked')
    rect, client, work = window['rect'], window['client_screen_rect'], window['work_area']
    if rect[2] <= rect[0] or rect[3] <= rect[1]:
        reasons.append('empty_rect')
    if work is None:
        reasons.append('outside_every_monitor')
    capture = not reasons
    click = capture
    if capture and (client[2] <= client[0] or client[3] <= client[1]):
        reasons.append('empty_client_area')
        click = False
    if capture and click and not inside(client, work):
        # 仍可擷取；超出工作區的控件沒有可點擊的螢幕座標。
        reasons.append('client_area_partly_outside_work_area')
        click = False
    keyboard = capture
    if not window['foreground']:
        # OS SendInput 的鍵盤事件只會送到前景視窗。
        reasons.append('not_foreground')
        keyboard = False
    return capture, click, keyboard, reasons


def choose_main(windows, window_class=None):
    """主視窗：沒有 owner、有標題列；可用類別名稱指定，否則取面積最大者。"""
    candidates = [w for w in windows if not w['owned'] and w['style'] & WS_CAPTION]
    if window_class:
        candidates = [w for w in candidates if w['class'] == window_class]
    if not candidates:
        return None
    area = lambda w: max(0, w['rect'][2] - w['rect'][0]) * max(0, w['rect'][3] - w['rect'][1])
    return max(candidates, key=lambda w: (w['visible'], area(w)))


def same_instant(a, b):
    return abs(a - b) < timedelta(milliseconds=1)


def parse_instant(text):
    value = datetime.fromisoformat(text.replace('Z', '+00:00'))
    if value.tzinfo is None:
        raise ValueError('creation time needs an explicit UTC offset')
    return value.astimezone(timezone.utc)


def resolve_output(text, root=ROOT):
    path = Path(text)
    path = (path if path.is_absolute() else root / path).resolve()
    if not any(path.is_relative_to((root / name).resolve()) for name in OUTPUT_ROOTS):
        raise ValueError('output must be inside ' + ' or '.join(OUTPUT_ROOTS) + ' of this project')
    return path


def write_result(directory, result):
    """先寫暫存檔再置換，讀取端不會看到寫到一半的 result.json。"""
    directory.mkdir(parents=True, exist_ok=True)
    temporary = directory / ('result.json.' + str(os.getpid()) + '.tmp')
    temporary.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
    os.replace(temporary, directory / 'result.json')


# ---------------------------------------------------------------- Win32
class Win32:
    """ctypes 包裝；只在實際檢查時建立，--self-test 不會載入。"""

    def __init__(self):
        import ctypes
        from ctypes import wintypes
        self.c, self.w = ctypes, wintypes
        u = self.user32 = ctypes.WinDLL('user32', use_last_error=True)
        k = self.kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)
        self.dwmapi = ctypes.WinDLL('dwmapi', use_last_error=True)
        H, B, D, I = wintypes.HWND, wintypes.BOOL, wintypes.DWORD, ctypes.c_int
        self.ENUM = ctypes.WINFUNCTYPE(B, H, wintypes.LPARAM)

        class MONITORINFO(ctypes.Structure):
            _fields_ = [('cbSize', D), ('rcMonitor', wintypes.RECT), ('rcWork', wintypes.RECT), ('dwFlags', D)]
        self.MONITORINFO = MONITORINFO
        signatures = [
            (u.EnumWindows, [self.ENUM, wintypes.LPARAM], B),
            (u.GetWindowThreadProcessId, [H, ctypes.POINTER(D)], D),
            (u.IsWindow, [H], B), (u.IsWindowVisible, [H], B), (u.IsIconic, [H], B), (u.IsZoomed, [H], B),
            (u.GetWindowRect, [H, ctypes.POINTER(wintypes.RECT)], B),
            (u.GetClientRect, [H, ctypes.POINTER(wintypes.RECT)], B),
            (u.ClientToScreen, [H, ctypes.POINTER(wintypes.POINT)], B),
            (u.GetClassNameW, [H, wintypes.LPWSTR, I], I),
            (u.GetWindowTextW, [H, wintypes.LPWSTR, I], I),
            (u.GetWindowLongPtrW, [H, I], ctypes.c_ssize_t),
            (u.GetWindow, [H, wintypes.UINT], H),
            (u.MonitorFromWindow, [H, D], wintypes.HANDLE),
            (u.GetMonitorInfoW, [wintypes.HANDLE, ctypes.POINTER(MONITORINFO)], B),
            (u.GetDpiForWindow, [H], wintypes.UINT),
            (u.GetForegroundWindow, [], H),
            (u.ShowWindowAsync, [H, I], B),
            (u.SetWindowPos, [H, H, I, I, I, I, wintypes.UINT], B),
            (u.SetForegroundWindow, [H], B),
            (u.SetThreadDpiAwarenessContext, [ctypes.c_void_p], ctypes.c_void_p),
            (u.OpenInputDesktop, [D, B, D], wintypes.HANDLE),
            (u.GetUserObjectInformationW, [wintypes.HANDLE, I, ctypes.c_void_p, D, ctypes.POINTER(D)], B),
            (u.CloseDesktop, [wintypes.HANDLE], B),
            (self.dwmapi.DwmGetWindowAttribute, [H, D, ctypes.c_void_p, D], ctypes.c_long),
            (k.OpenProcess, [D, B, D], wintypes.HANDLE),
            (k.CloseHandle, [wintypes.HANDLE], B),
            (k.GetProcessTimes, [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4, B),
            (k.GetExitCodeProcess, [wintypes.HANDLE, ctypes.POINTER(D)], B),
            (k.QueryFullProcessImageNameW, [wintypes.HANDLE, D, wintypes.LPWSTR, ctypes.POINTER(D)], B),
            (k.ProcessIdToSessionId, [D, ctypes.POINTER(D)], B),
            (k.WTSGetActiveConsoleSessionId, [], D),
        ]
        for function, argtypes, restype in signatures:
            function.argtypes, function.restype = argtypes, restype
        # 只影響本執行緒：讓座標是實際像素，不被 DPI 虛擬化。
        u.SetThreadDpiAwarenessContext(ctypes.c_void_p(-4))

    def process(self, pid):
        """回傳 dict；程序不存在時回傳 None。"""
        c, w, k = self.c, self.w, self.kernel32
        handle = k.OpenProcess(0x1000, False, pid)  # PROCESS_QUERY_LIMITED_INFORMATION
        if not handle:
            return None
        try:
            times = [w.FILETIME() for _ in range(4)]
            if not k.GetProcessTimes(handle, *[c.byref(t) for t in times]):
                raise c.WinError(c.get_last_error())
            ticks = times[0].dwLowDateTime | times[0].dwHighDateTime << 32
            created = datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)
            code = w.DWORD()
            k.GetExitCodeProcess(handle, c.byref(code))
            size = w.DWORD(32768)
            buffer = c.create_unicode_buffer(size.value)
            exe = buffer.value if k.QueryFullProcessImageNameW(handle, 0, buffer, c.byref(size)) else None
            session = w.DWORD()
            has_session = k.ProcessIdToSessionId(pid, c.byref(session))
            return dict(pid=pid, creation_utc=created.isoformat(), created=created, exe=exe,
                        running=code.value == 259, session_id=session.value if has_session else None)
        finally:
            k.CloseHandle(handle)

    def environment(self):
        c, w, u = self.c, self.w, self.user32
        name = None
        desktop = u.OpenInputDesktop(0, False, 0x0001)  # DESKTOP_READOBJECTS
        if desktop:
            try:
                buffer = c.create_unicode_buffer(256)
                needed = w.DWORD()
                if u.GetUserObjectInformationW(desktop, 2, buffer, c.sizeof(buffer), c.byref(needed)):
                    name = buffer.value
            finally:
                u.CloseDesktop(desktop)
        foreground = u.GetForegroundWindow()
        pid = w.DWORD()
        if foreground:
            u.GetWindowThreadProcessId(foreground, c.byref(pid))
        return dict(input_desktop=name, console_session_id=self.kernel32.WTSGetActiveConsoleSessionId(),
                    foreground_hwnd=int(foreground or 0), foreground_pid=pid.value or None)

    def windows(self, pid):
        c, w, u = self.c, self.w, self.user32
        handles = []

        def visit(hwnd, _):
            owner = w.DWORD()
            u.GetWindowThreadProcessId(hwnd, c.byref(owner))
            if owner.value == pid:
                handles.append(hwnd)
            return True
        callback = self.ENUM(visit)
        if not u.EnumWindows(callback, 0):
            raise c.WinError(c.get_last_error())
        foreground = int(u.GetForegroundWindow() or 0)
        return [self.window(hwnd, foreground) for hwnd in handles]

    def window(self, hwnd, foreground):
        c, w, u = self.c, self.w, self.user32
        text = c.create_unicode_buffer(512)
        u.GetClassNameW(hwnd, text, 512)
        class_name = text.value
        u.GetWindowTextW(hwnd, text, 512)
        title = text.value
        rect, client, origin = w.RECT(), w.RECT(), w.POINT(0, 0)
        u.GetWindowRect(hwnd, c.byref(rect))
        u.GetClientRect(hwnd, c.byref(client))
        u.ClientToScreen(hwnd, c.byref(origin))
        cloaked = w.DWORD()
        if self.dwmapi.DwmGetWindowAttribute(hwnd, 14, c.byref(cloaked), c.sizeof(cloaked)) != 0:  # DWMWA_CLOAKED
            cloaked = w.DWORD(0)
        work = monitor_rect = None
        monitor = u.MonitorFromWindow(hwnd, 0)  # MONITOR_DEFAULTTONULL
        if monitor:
            info = self.MONITORINFO()
            info.cbSize = c.sizeof(info)
            if u.GetMonitorInfoW(monitor, c.byref(info)):
                work = [info.rcWork.left, info.rcWork.top, info.rcWork.right, info.rcWork.bottom]
                monitor_rect = [info.rcMonitor.left, info.rcMonitor.top, info.rcMonitor.right, info.rcMonitor.bottom]
        return dict(
            hwnd=int(hwnd), **{'class': class_name}, title=title,
            visible=bool(u.IsWindowVisible(hwnd)), minimized=bool(u.IsIconic(hwnd)), maximized=bool(u.IsZoomed(hwnd)),
            cloaked=cloaked.value, owned=bool(u.GetWindow(hwnd, 4)),  # GW_OWNER
            style=u.GetWindowLongPtrW(hwnd, -16) & 0xFFFFFFFF, ex_style=u.GetWindowLongPtrW(hwnd, -20) & 0xFFFFFFFF,
            rect=[rect.left, rect.top, rect.right, rect.bottom],
            client_screen_rect=[origin.x, origin.y, origin.x + client.right, origin.y + client.bottom],
            dpi=int(u.GetDpiForWindow(hwnd)), work_area=work, monitor_rect=monitor_rect,
            foreground=int(hwnd) == foreground)

    def nearest_work_area(self, hwnd):
        c, u = self.c, self.user32
        info = self.MONITORINFO()
        info.cbSize = c.sizeof(info)
        monitor = u.MonitorFromWindow(hwnd, 2)  # MONITOR_DEFAULTTONEAREST
        if monitor and u.GetMonitorInfoW(monitor, c.byref(info)):
            return (info.rcWork.left, info.rcWork.top, info.rcWork.right, info.rcWork.bottom)
        return None

    def recover(self, window, reasons):
        """只處理傳入的那一個視窗；每個動作都記錄 API 回傳值。"""
        u, hwnd, actions = self.user32, window['hwnd'], []
        if 'minimized' in reasons:
            actions.append(dict(action='ShowWindowAsync(SW_RESTORE)', returned=bool(u.ShowWindowAsync(hwnd, SW_RESTORE))))
            time.sleep(0.3)
        if not window['maximized'] and ({'outside_every_monitor', 'client_area_partly_outside_work_area', 'minimized'} & set(reasons)):
            rect, work = self.w.RECT(), self.nearest_work_area(hwnd)
            if work and u.GetWindowRect(hwnd, self.c.byref(rect)) and not u.IsIconic(hwnd):
                fitted = fit_to_work_area((rect.left, rect.top, rect.right, rect.bottom), work)
                if fitted:
                    ok = u.SetWindowPos(hwnd, None, fitted[0], fitted[1], fitted[2] - fitted[0], fitted[3] - fitted[1],
                                        SWP_NOZORDER | SWP_NOACTIVATE)
                    actions.append(dict(action='SetWindowPos(work area)', rect=list(fitted), returned=bool(ok)))
        if 'not_foreground' in reasons or 'minimized' in reasons:
            # 呼叫端沒有前景權時 Windows 會拒絕；拒絕就如實記錄，不用其他手段繞過。
            actions.append(dict(action='SetForegroundWindow', returned=bool(u.SetForegroundWindow(hwnd))))
        return actions


# ---------------------------------------------------------------- 流程
def probe(arguments, result, api):
    expected = parse_instant(arguments.creation_utc)
    exe_root = (Path(arguments.exe_root).resolve() if arguments.exe_root else ROOT)
    deadline = time.monotonic() + arguments.wait_seconds
    attempts_left = arguments.recover_attempts if arguments.recover else 0
    recovered = False
    while True:
        process = api.process(arguments.pid)
        if process is None or not process['running']:
            result.update(status='process_exited', process=None if process is None else {k: v for k, v in process.items() if k != 'created'})
            return EXIT_IDENTITY
        created = process.pop('created')
        result['process'] = process
        if not same_instant(created, expected):
            result.update(status='identity_mismatch', error='PID 的建立時間與 --creation-utc 不符；不檢查也不操作這個程序')
            return EXIT_IDENTITY
        if not process['exe'] or not Path(process['exe']).resolve().is_relative_to(exe_root):
            result.update(status='identity_mismatch', error='程序的 exe 不在 ' + str(exe_root) + ' 內；只處理自己建置的客戶端')
            return EXIT_IDENTITY
        environment = api.environment()
        windows = api.windows(arguments.pid)
        main = choose_main(windows, arguments.window_class)
        capture, click, keyboard, reasons = assess(main, environment)
        result.update(environment=environment, windows=windows, main_window_hwnd=main['hwnd'] if main else None,
                      capture_ready=capture, click_geometry_ready=click, keyboard_input_ready=keyboard, reasons=reasons)
        if capture and click and keyboard:
            result['status'] = 'recovered' if recovered else 'ready'
            return EXIT_READY
        recoverable = main is not None and main['visible'] and 'input_desktop_unavailable' not in reasons and 'cloaked' not in reasons
        if recoverable and attempts_left > 0:
            attempts_left -= 1
            result['actions'].extend(api.recover(main, reasons))
            recovered = True
            time.sleep(0.5)
            continue
        if main is None and time.monotonic() < deadline:
            time.sleep(0.2)  # 視窗還沒建立
            continue
        result['status'] = 'not_ready'
        return EXIT_NOT_READY


def self_test():
    work = (0, 0, 1920, 1040)
    assert fit_to_work_area((100, 100, 700, 920), work) is None
    assert fit_to_work_area((-32000, -32000, -31400, -31180), work) == (0, 0, 600, 820)
    assert fit_to_work_area((1800, 900, 2400, 1720), work) == (1320, 220, 1920, 1040)
    assert fit_to_work_area((200, 50, 1400, 1690), work) == (200, 0, 1400, 1040)
    assert fit_to_work_area((-3000, -500, -2400, 320), (-2560, -200, 0, 1200)) == (-2560, -200, -1960, 620)
    assert fit_to_work_area((10, 10, 10, 500), work) is None and fit_to_work_area((1, 1, 5, 5), (0, 0, 0, 0)) is None
    env = dict(input_desktop='Default')
    base = dict(hwnd=1, visible=True, minimized=False, cloaked=0, owned=False, style=WS_CAPTION, foreground=True,
                rect=[100, 100, 700, 920], client_screen_rect=[108, 131, 692, 912], work_area=list(work), **{'class': 'A'})
    assert assess(base, env) == (True, True, True, [])
    assert assess(None, env) == (False, False, False, ['no_main_window'])
    assert assess(base, dict(input_desktop=None)) == (False, False, False, ['input_desktop_unavailable'])
    minimized = dict(base, minimized=True, foreground=False, rect=[-32000, -32000, -31840, -31972],
                     client_screen_rect=[-32000, -32000, -32000, -32000], work_area=None)
    assert assess(minimized, env) == (False, False, False, ['minimized', 'outside_every_monitor', 'not_foreground'])
    assert assess(dict(base, visible=False), env)[3] == ['hidden']
    assert assess(dict(base, cloaked=2), env)[:3] == (False, False, False)
    assert assess(dict(base, foreground=False), env) == (True, True, False, ['not_foreground'])
    tall = dict(base, rect=[200, 0, 1400, 1640], client_screen_rect=[208, 31, 1392, 1632])
    assert assess(tall, env) == (True, False, True, ['client_area_partly_outside_work_area'])
    assert assess(dict(base, rect=[5, 5, 5, 5]), env)[3] == ['empty_rect']
    small = dict(base, hwnd=2, rect=[0, 0, 10, 10])
    assert choose_main([small, base])['hwnd'] == 1
    assert choose_main([dict(base, owned=True), dict(base, style=0)]) is None
    assert choose_main([dict(base, visible=False), small])['hwnd'] == 2
    assert choose_main([base, dict(small, **{'class': 'B'})], 'B')['hwnd'] == 2
    now = datetime(2026, 11, 12, 9, 0, 0, 123456, tzinfo=timezone.utc)
    assert same_instant(parse_instant('2026-11-12T09:00:00.123456Z'), now)
    assert same_instant(parse_instant('2026-11-12T17:00:00.123456+08:00'), now)
    assert not same_instant(parse_instant('2026-11-12T09:00:00.125456+00:00'), now)
    for bad in ('2026-11-12T09:00:00', 'yesterday'):
        try:
            parse_instant(bad)
        except ValueError:
            continue
        raise AssertionError(bad)
    root = Path(__file__).resolve().parents[1]
    assert resolve_output('scratch/gui-probe/x', root) == (root / 'scratch/gui-probe/x').resolve()
    assert resolve_output(str(root / 'LiTo381-Modern/target/current/p'), root).name == 'p'
    for bad in ('docs/x', '../outside', str(root.parent / 'scratch')):
        try:
            resolve_output(bad, root)
        except ValueError:
            continue
        raise AssertionError(bad)

    class Fake:
        """沒有視窗、然後程序結束；用來確認流程不會卡住而且狀態正確。"""
        def __init__(self, rows): self.rows, self.recovered = list(rows), []
        def process(self, pid): return dict(pid=pid, creation_utc=now.isoformat(), created=now, exe=str(root / 'LiTo381-Modern/target/a.exe'), running=True, session_id=1)
        def environment(self): return dict(env)
        def windows(self, pid): return [self.rows.pop(0)] if len(self.rows) > 1 else list(self.rows)
        def recover(self, window, reasons): self.recovered.append(list(reasons)); return [dict(action='fake', returned=True)]
    make = lambda **extra: argparse.Namespace(pid=7, creation_utc=now.isoformat(), exe_root=None, wait_seconds=0, recover=False,
                                              recover_attempts=2, window_class=None, **extra)
    visible_minimized = dict(minimized, visible=True)
    result = dict(actions=[])
    assert probe(make(), result, Fake([base])) == EXIT_READY and result['status'] == 'ready'
    result = dict(actions=[])
    assert probe(make(), result, Fake([visible_minimized])) == EXIT_NOT_READY and result['status'] == 'not_ready' and not result['actions']
    result, fake = dict(actions=[]), Fake([visible_minimized, base])
    arguments = make()
    arguments.recover = True
    assert probe(arguments, result, fake) == EXIT_READY and result['status'] == 'recovered' and len(fake.recovered) == 1
    result, fake = dict(actions=[]), Fake([visible_minimized])
    assert probe(arguments, result, fake) == EXIT_NOT_READY and len(fake.recovered) == 2 and result['capture_ready'] is False
    result = dict(actions=[])
    arguments = make()
    arguments.creation_utc = (now + timedelta(seconds=1)).isoformat()
    assert probe(arguments, result, Fake([base])) == EXIT_IDENTITY and result['status'] == 'identity_mismatch' and 'windows' not in result
    result = dict(actions=[])
    arguments = make()
    arguments.exe_root = str(root / 'docs')
    assert probe(arguments, result, Fake([base])) == EXIT_IDENTITY and 'windows' not in result
    print('self-test ok')


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument('--self-test', action='store_true', help='只跑純邏輯檢查，不碰任何視窗或程序')
    parser.add_argument('--pid', type=int)
    parser.add_argument('--creation-utc', help='啟動該程序時記下的建立時間（ISO 8601，含時區）')
    parser.add_argument('--output', help='result.json 的資料夾；必須在本專案 scratch/ 或 LiTo381-Modern/target/ 內')
    parser.add_argument('--exe-root', help='exe 必須位於此資料夾內（預設：本專案根目錄）')
    parser.add_argument('--window-class', help='主視窗類別名稱，例如 LiToNativeLoginWindow')
    parser.add_argument('--wait-seconds', type=float, default=15.0, help='等待主視窗出現的上限')
    parser.add_argument('--recover', action='store_true', help='未就緒時還原／移回工作區／嘗試設為前景')
    parser.add_argument('--recover-attempts', type=int, default=2)
    arguments = parser.parse_args()
    if arguments.self_test:
        self_test()
        return EXIT_READY
    if arguments.pid is None or not arguments.creation_utc or not arguments.output:
        parser.error('--pid、--creation-utc、--output 都要指定')
    output = resolve_output(arguments.output)
    result = dict(schema=SCHEMA, status='error', started_utc=datetime.now(timezone.utc).isoformat(),
                  request=dict(pid=arguments.pid, creation_utc=arguments.creation_utc, recover=arguments.recover,
                               window_class=arguments.window_class, wait_seconds=arguments.wait_seconds),
                  process=None, environment=None, windows=None, main_window_hwnd=None,
                  capture_ready=None, click_geometry_ready=None, keyboard_input_ready=None, reasons=None,
                  actions=[], error=None,
                  input_sent=False, pixels_captured=False, sessions_observed=None, GUI_acceptance=False)
    code = EXIT_ERROR
    try:
        code = probe(arguments, result, Win32())
    except BaseException as error:  # 包含 KeyboardInterrupt：仍然要留下 result.json
        result.update(status='error', error=repr(error))
        code = EXIT_ERROR
        if not isinstance(error, Exception):
            raise
    finally:
        result.update(finished_utc=datetime.now(timezone.utc).isoformat(), exit_code=code)
        write_result(output, result)
        print(json.dumps(dict(result=str(output / 'result.json'), status=result['status'], reasons=result['reasons']), ensure_ascii=False), flush=True)
    return code


if __name__ == '__main__':
    sys.exit(main())
