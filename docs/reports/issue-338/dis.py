# Read-only linear disassembly of ranges in the unpacked LiTo image (never executed).
# usage: python -I dis.py <exe> <start_hex>:<len_hex>[:stop_at_ret] ...
import sys
import pefile, capstone

raw = open(sys.argv[1], "rb").read()
pe = pefile.PE(data=raw, fast_load=True)
base = pe.OPTIONAL_HEADER.ImageBase
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
for spec in sys.argv[2:]:
    parts = spec.split(":")
    va, n = int(parts[0], 16), int(parts[1], 16)
    stop = len(parts) > 2
    data = pe.get_data(va - base, n)
    print(f"=== {va:08X}")
    for x in md.disasm(data, va):
        print(f"{x.address:08X} {x.bytes.hex():<20} {x.mnemonic} {x.op_str}")
        if stop and x.mnemonic == "ret":
            break
