# Read-only: look up member names in old28 idx files (plain). Usage: python -I -B find_img.py 773
import sys, struct, os, hashlib
root = r"D:\lineage開發\0-客戶端"
want = [f"{sys.argv[1]}{s}.img" for s in ("", "c", "h", "j", "e")]
def scan(idx, pak):
    b = open(idx, "rb").read()
    n = struct.unpack_from("<I", b)[0]
    if len(b) != 4 + n * 28:
        print(idx, "not plain old28", len(b), n); return
    hit = False
    for i in range(n):
        off, name, size = struct.unpack_from("<I20sI", b, 4 + i * 28)
        name = name.split(b"\0")[0].decode("latin1").lower()
        if name in want:
            hit = True
            with open(pak, "rb") as f:
                f.seek(off); d = f.read(size)
            print(os.path.basename(idx), name, "off", off, "size", size, "sha256", hashlib.sha256(d).hexdigest().upper(), "head", d[:8].hex())
            if os.path.basename(idx).lower() == "ltsurf.idx" and len(sys.argv) > 2:
                w, h = struct.unpack_from("<hh", d)
                px = struct.unpack_from("<%dH" % (w * h), d, 8)
                for y in range(h):
                    print("".join("." if px[y * w + x] == struct.unpack_from("<H", d, 6)[0] else "#" for x in range(w)))
    if not hit: print(os.path.basename(idx), "no", want[0], "entries", n)
scan(os.path.join(root, "LiTo", "LTsurf.idx"), os.path.join(root, "LiTo", "LTsurf.pak"))
scan(os.path.join(root, "LiTo", "LTicon.idx"), os.path.join(root, "LiTo", "LTicon.pak"))
scan(os.path.join(root, "LiTo", "LTsprite.idx"), os.path.join(root, "LiTo", "LTsprite.pak"))
for s in ["Sprite"] + ["Sprite%02d" % i for i in range(16)]:
    scan(os.path.join(root, s + ".idx"), os.path.join(root, s + ".pak"))
