import sys, re, struct, pefile, capstone
PE = r"D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe"
pe = pefile.PE(PE, fast_load=True); base = pe.OPTIONAL_HEADER.ImageBase
data = pe.get_memory_mapped_image()
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
lo, hi = 0x401000, 0x4F7000
blob = data[lo-base:hi-base]
mode = sys.argv[1]
if mode == "disp32":
    pat = struct.pack("<I", int(sys.argv[2],16))
    i = blob.find(pat)
    while i >= 0:
        for back in (2,3,1):
            ins = next(md.disasm(blob[i-back:i-back+16], lo+i-back), None)
            if ins and sys.argv[2].lower().lstrip("0") in ins.op_str and ins.size >= back+4:
                print("%08X: %s %s" % (ins.address, ins.mnemonic, ins.op_str)); break
        i = blob.find(pat, i+1)
else:
    # byte-sized access to [reg+disp8]
    d = int(sys.argv[2],16)
    for m in re.finditer(bytes([0x80]) + b"[\x78-\x7b\x7e\x7f]" + bytes([d]) + b"|" + b"\x8a[\x40-\x43\x46-\x4b\x4e-\x53\x56-\x5b\x5e\x5f]" + bytes([d]) + b"|\x38[\x40-\x43\x46-\x4b\x4e-\x53\x56-\x5b\x5e\x5f]" + bytes([d]), blob):
        ins = next(md.disasm(blob[m.start():m.start()+16], lo+m.start()), None)
        if ins: print("%08X: %s %s" % (ins.address, ins.mnemonic, ins.op_str))
