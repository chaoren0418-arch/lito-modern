# Read-only static disassembly helper. Usage: python -I -B dis.py <va_hex> <len_hex> | xref <va_hex> | str <va_hex> | find <hexbytes>
import sys, struct, pefile, capstone
PE = r"D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe"
pe = pefile.PE(PE, fast_load=True)
base = pe.OPTIONAL_HEADER.ImageBase
data = pe.get_memory_mapped_image()
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
def rd(va, n): return data[va - base: va - base + n]
cmd = sys.argv[1]
if cmd == "xref":
    # scan .text-like range for call/jmp rel32 and absolute imm32 references
    tgt = int(sys.argv[2], 16)
    lo, hi = 0x401000, int(sys.argv[3], 16) if len(sys.argv) > 3 else 0x640000
    blob = rd(lo, hi - lo)
    pat = struct.pack("<I", tgt)
    i = blob.find(pat)
    while i >= 0:
        print("abs %08X" % (lo + i)); i = blob.find(pat, i + 1)
    for i in range(len(blob) - 5):
        if blob[i] in (0xE8, 0xE9):
            rel = struct.unpack_from("<i", blob, i + 1)[0]
            if lo + i + 5 + rel == tgt: print("%s %08X" % ("call" if blob[i] == 0xE8 else "jmp", lo + i))
elif cmd == "str":
    va = int(sys.argv[2], 16); b = rd(va, 128); print(repr(b.split(b"\0")[0]))
elif cmd == "find":
    pat = bytes.fromhex(sys.argv[2]); i = data.find(pat)
    while i >= 0:
        print("%08X" % (base + i)); i = data.find(pat, i + 1)
elif cmd == "secs":
    print(hex(base))
    for s in pe.sections: print(s.Name, hex(base + s.VirtualAddress), hex(s.Misc_VirtualSize))
else:
    va = int(sys.argv[1], 16); n = int(sys.argv[2], 16)
    for ins in md.disasm(rd(va, n), va):
        print("%08X: %s %s" % (ins.address, ins.mnemonic, ins.op_str))
