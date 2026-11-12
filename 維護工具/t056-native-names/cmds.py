import sys, re, pefile, capstone
PE = r"D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe"
pe = pefile.PE(PE, fast_load=True); base = pe.OPTIONAL_HEADER.ImageBase
data = pe.get_memory_mapped_image()
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
va=int(sys.argv[1],16); n=int(sys.argv[2],16)
def s(a):
    b=data[a-base:a-base+64].split(b"\0")[0]
    return b.decode("cp950","replace")
for ins in md.disasm(data[va-base:va-base+n], va):
    t="%s %s"%(ins.mnemonic,ins.op_str)
    if ins.mnemonic=="push":
        m=re.fullmatch(r"0x(50[0-9a-f]{4})",ins.op_str)
        if m: print("%08X: push str %s = %r"%(ins.address,ins.op_str,s(int(m.group(1),16))))
    elif ins.mnemonic in("je","jne","jmp","call") and ins.op_str.startswith("0x4"):
        tgt=int(ins.op_str,16)
        if ins.mnemonic=="call" or tgt>ins.address+0x40 or tgt<ins.address-0x40: print("%08X: %s"%(ins.address,t))
    elif "0x834bc4" in t or re.search(r"\[e.. \+ 0x[0-9a-f]{3}\]",t) and ins.mnemonic=="mov" and "ebp" not in t: print("%08X: %s"%(ins.address,t))
    elif "0x13a" in t or "ret" == ins.mnemonic: print("%08X: %s"%(ins.address,t))
