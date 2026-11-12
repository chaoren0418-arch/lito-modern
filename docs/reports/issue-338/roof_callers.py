# Read-only static inspection of the unpacked LiTo image (never executed).
# usage: python -I roof_callers.py <exe> <out.json> <hex target> [<hex target>...]
import sys, json, hashlib, struct
import pefile, capstone

exe, out = sys.argv[1], sys.argv[2]
targets = [int(v, 16) for v in sys.argv[3:]]
raw = open(exe, "rb").read()
pe = pefile.PE(data=raw, fast_load=True)
base = pe.OPTIONAL_HEADER.ImageBase
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
result = {"image": {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest().upper()},
          "image_base": hex(base), "targets": {}}
code = []
for s in pe.sections:
    if s.Characteristics & 0x20000000:
        code.append((base + s.VirtualAddress, s.get_data()))


def read(va, n):
    for start, data in code:
        if start <= va < start + len(data):
            return data[va - start: va - start + n]
    return b""


def function_start(va):
    # Walk back to the nearest padding/ret boundary followed by a prologue.
    for back in range(0, 0x4000):
        a = va - back
        b = read(a - 1, 4)
        if len(b) < 4:
            break
        if b[0] in (0xCC, 0x90, 0xC3) or read(a - 3, 1) == b"\xc2":
            if b[1:4] == b"\x55\x8b\xec" or b[1] in (0x53, 0x56, 0x57, 0x83, 0x81, 0x6A, 0x64, 0xA1, 0x8B):
                return a
    return None


for target in targets:
    callers = []
    for start, data in code:
        i = 0
        while True:
            i = data.find(b"\xe8", i)
            if i < 0 or i + 5 > len(data):
                break
            rel = struct.unpack_from("<i", data, i + 1)[0]
            if (start + i + 5 + rel) & 0xFFFFFFFF == target:
                site = start + i
                before = read(site - 48, 48)
                # Linear decode of the preceding window is ambiguous; keep raw bytes too.
                tail = []
                for skip in range(0, 16):
                    ins = list(md.disasm(before[skip:], site - 48 + skip))
                    if ins and ins[-1].address + ins[-1].size == site:
                        tail = [f"{x.address:08X} {x.mnemonic} {x.op_str}" for x in ins[-10:]]
                        break
                callers.append({"site": f"{site:08X}", "function_guess": (lambda f: f and f"{f:08X}")(function_start(site)),
                                "before_hex": before.hex(), "before": tail})
            i += 1
    body = read(target, 0x260)
    ins = []
    for x in md.disasm(body, target):
        ins.append(f"{x.address:08X} {x.mnemonic} {x.op_str}")
        if x.mnemonic in ("ret", "retn") and len(ins) > 8:
            break
    result["targets"][f"{target:08X}"] = {"callers": callers, "head": ins}
open(out, "w", encoding="utf-8").write(json.dumps(result, indent=1))
for t, v in result["targets"].items():
    print(t, "callers", [(c["site"], c["function_guess"]) for c in v["callers"]])
