from pathlib import Path
import sys,hashlib,zipfile
formal,changed,output=map(Path,sys.argv[1:])
root=Path(__file__).resolve().parents[3]
assert output.resolve().is_relative_to((root/"LiTo381-Modern/target/current").resolve())
assert hashlib.sha256(formal.read_bytes()).hexdigest().upper()=="BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43"
assert not output.exists()
name="local/lito381/server/ClientSession.class"
with zipfile.ZipFile(formal) as src,zipfile.ZipFile(output,"w") as dst:
 assert not any(n.upper().endswith((".SF",".RSA",".DSA")) for n in src.namelist())
 for info in src.infolist():dst.writestr(info,changed.read_bytes() if info.filename==name else src.read(info))
with zipfile.ZipFile(formal) as a,zipfile.ZipFile(output) as b:
 assert a.namelist()==b.namelist()
 assert [n for n in a.namelist() if a.read(n)!=b.read(n)]==[name]
assert hashlib.sha256(output.read_bytes()).hexdigest().upper()=="6887A337E347D561366E0EB7EB8E93A2B7EA03ADD7A3888FC4BE2B70C3479DA7"
print(output)
