# Read-only: list E8/E9 rel32 sites that target the given addresses in the unpacked
# LiTo image (never executed). A byte-pattern hit is only reported when a linear
# decode at that offset yields a call/jmp to the target.
# usage: python -I calls.py <exe> <hex addr> ...
import sys, struct
import pefile, capstone

raw = open(sys.argv[1], "rb").read()
pe = pefile.PE(data=raw, fast_load=True)
base = pe.OPTIONAL_HEADER.ImageBase
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
targets = [int(a, 16) for a in sys.argv[2:]]
hits = {t: [] for t in targets}
for s in pe.sections:
    if not s.Characteristics & 0x20000000:
        continue
    data = s.get_data()
    start = base + s.VirtualAddress
    for i in range(len(data) - 5):
        if data[i] not in (0xE8, 0xE9):
            continue
        dest = (start + i + 5 + struct.unpack_from("<i", data, i + 1)[0]) & 0xFFFFFFFF
        if dest in hits:
            ins = next(md.disasm(data[i:i + 5], start + i), None)
            if ins is not None:
                hits[dest].append((start + i, ins.mnemonic))
for t in targets:
    print(f"=== {t:08X}: {len(hits[t])}")
    for site, kind in hits[t]:
        print(f"{site:08X} {kind}")
