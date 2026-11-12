# Read-only: list code locations embedding an absolute address (never executes the image).
# usage: python -I xref.py <exe> <hex addr> ...
import sys, struct
import pefile, capstone

raw = open(sys.argv[1], "rb").read()
pe = pefile.PE(data=raw, fast_load=True)
base = pe.OPTIONAL_HEADER.ImageBase
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
for arg in sys.argv[2:]:
    addr = int(arg, 16)
    needle = struct.pack("<I", addr)
    print(f"=== {addr:08X}")
    for s in pe.sections:
        if not s.Characteristics & 0x20000000:
            continue
        data = s.get_data()
        start = base + s.VirtualAddress
        i = 0
        while True:
            i = data.find(needle, i)
            if i < 0:
                break
            # find an instruction that ends right after or contains the immediate
            shown = False
            for back in range(1, 8):
                ins = next(md.disasm(data[i - back:i - back + 16], start + i - back), None)
                if ins and ins.address + ins.size >= start + i + 4 and arg.lower().lstrip("0") in ins.op_str.replace("0x", ""):
                    print(f"{ins.address:08X} {ins.mnemonic} {ins.op_str}")
                    shown = True
                    break
            if not shown:
                print(f"{start + i:08X} (raw immediate)")
            i += 1
