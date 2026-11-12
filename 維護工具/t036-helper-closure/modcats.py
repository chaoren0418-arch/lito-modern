# -*- coding: utf-8 -*-
"""Issue #307 靜態工具：每個模組（去掉行內測試模組與註解／字串後）的副作用分類，
以及指定模組對「帶危險分類模組」的具體引用路徑（唯讀）。

用法: python -I modcats.py <src/modern_lito> <mod,mod,...>
分類:
  WRITE   檔案寫入／刪除／改名／建目錄
  NET     socket
  PROC    子程序
  THREAD  建立執行緒
  WINDOW  建立視窗／訊息迴圈
  AUDIO   waveOut 裝置
  ENV     讀寫環境變數／目前目錄
  READ    檔案讀取／metadata
  WIN32   其他 unsafe／Win32 呼叫
  CLOCK   Instant／SystemTime／QPC
"""
import io, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fnclosure import Mod  # noqa: E402

CATS = [
    ("WRITE", r"File::create|fs::write|\.create\(true\)|\.create_new\(true\)|\.write\(true\)|\.append\(true\)|\.truncate\(true\)|create_dir|remove_file|remove_dir|fs::rename|fs::copy|set_permissions|set_len|WriteFile|DeleteFile|MoveFile"),
    ("NET", r"TcpStream|TcpListener|UdpSocket|std::net\b|rustls|WSA"),
    ("PROC", r"Command::new|std::process\b|CreateProcess|ShellExecute"),
    ("THREAD", r"thread::spawn|thread::Builder|thread::scope|CreateThread"),
    ("WINDOW", r"CreateWindowEx|RegisterClass|GetMessage|PeekMessage|DispatchMessage|ShowWindow|MessageBox"),
    ("AUDIO", r"waveOut|PlaySound|audio_output::|AudioOutput\b|PcmVoice"),
    ("ENV", r"env::var|env::set_var|env::remove_var|current_dir|current_exe|temp_dir|env::args"),
    ("READ", r"std::fs\b|\bfs::|File::open|OpenOptions|read_to_end|read_exact|read_dir|metadata\("),
    ("WIN32", r"\bunsafe\b|extern\s+\"system\"|\bwindows::"),
    ("CLOCK", r"Instant::now|SystemTime|QueryPerformance|GetTickCount"),
]
CATS = [(n, re.compile(p)) for n, p in CATS]
DANGER = {"WRITE", "NET", "PROC", "THREAD", "WINDOW", "AUDIO"}


def main():
    src, want = sys.argv[1], sys.argv[2].split(",")
    modules = {f[:-3] for f in os.listdir(src) if f.endswith(".rs")}
    cache = {}

    def mod(n):
        if n not in cache:
            cache[n] = Mod(n, os.path.join(src, n + ".rs"), modules)
        return cache[n]

    def cats(n):
        M = mod(n)
        out = {}
        for ln, line in enumerate(M.code.splitlines(), 1):
            for name, rx in CATS:
                if rx.search(line):
                    out.setdefault(name, []).append(ln)
        return out

    for n in want:
        if n not in modules:
            print("!! no module %s" % n)
            continue
        M = mod(n)
        own = cats(n)
        print("== %s  own: %s" % (n, ", ".join("%s(%d)" % (k, len(v)) for k, v in own.items()) or "-"))
        for k in sorted(set(own) & DANGER):
            print("   OWN %s lines: %s" % (k, own[k][:12]))
        for r in sorted(M.refs - {n}):
            rc = cats(r)
            d = sorted(set(rc) & DANGER)
            if not d:
                continue
            paths = sorted(set(re.findall(r"\b%s::[A-Za-z_0-9:]+" % re.escape(r), M.code)))
            print("   -> %s [%s] via %s" % (r, ",".join(d), ", ".join(paths)[:300]))


if __name__ == "__main__":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    main()
