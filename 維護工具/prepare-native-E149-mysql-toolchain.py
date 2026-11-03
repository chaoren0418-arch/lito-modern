"""Copy pinned MySQL binaries/vendor scripts and JDBC dependency, never run them."""
from pathlib import Path
from datetime import datetime,timezone
from ctypes import wintypes
import ctypes,hashlib,json,os,shutil,stat,struct
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
ref=w['current_formal_reference_identity'];assert sha(ref['formal'])==ref['formal_sha256']=='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
normal=read(w['current_development_typecheck_qualification']['qualification'])
inputs=read(Path(normal['run'])/'base-compiled-inputs.json');assert all(sha(R/n)==h for n,h in inputs.items())
vendor=Path(r'C:\Program Files\MySQL\MySQL Server 5.5')
def plain(path):
    st=path.lstat();assert not(st.st_file_attributes&stat.FILE_ATTRIBUTE_REPARSE_POINT),path
    return path
for p in [vendor,vendor/'bin',vendor/'share',vendor/'share/english',vendor/'share/charsets']:plain(p)
version_api=ctypes.WinDLL('version',use_last_error=True)
version_api.GetFileVersionInfoSizeW.argtypes=[wintypes.LPCWSTR,ctypes.POINTER(wintypes.DWORD)]
version_api.GetFileVersionInfoW.argtypes=[wintypes.LPCWSTR,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p]
version_api.VerQueryValueW.argtypes=[ctypes.c_void_p,wintypes.LPCWSTR,ctypes.POINTER(ctypes.c_void_p),ctypes.POINTER(wintypes.UINT)]
def version(path):
    ignored=wintypes.DWORD();size=version_api.GetFileVersionInfoSizeW(str(path),ctypes.byref(ignored));assert size
    buf=ctypes.create_string_buffer(size);assert version_api.GetFileVersionInfoW(str(path),0,size,buf)
    ptr=ctypes.c_void_p();length=wintypes.UINT();assert version_api.VerQueryValueW(buf,'\\',ctypes.byref(ptr),ctypes.byref(length))
    values=ctypes.cast(ptr,ctypes.POINTER(wintypes.DWORD));assert values[0]==0xFEEF04BD
    return '.'.join(str(v) for v in [values[2]>>16,values[2]&65535,values[3]>>16,values[3]&65535])
required=['bin/mysqld.exe','bin/mysql.exe','bin/mysqladmin.exe',
          'share/mysql_system_tables.sql','share/mysql_system_tables_data.sql','share/fill_help_tables.sql',
          'share/english/errmsg.sys']
for folder in ['bin','share/charsets']:
    root=vendor/folder
    for parent,dirs,files in os.walk(root,followlinks=False):
        for name in dirs:plain(Path(parent)/name)
        for name in files:
            path=plain(Path(parent)/name)
            if folder=='share/charsets' or path.suffix.lower()=='.dll':required.append(path.relative_to(vendor).as_posix())
required=sorted(set(required))
assert all(not set(Path(n).parts)&{'data','mysql'} and Path(n).suffix.lower() not in ['.frm','.myd','.myi','.ibd'] for n in required)
assert all(plain(vendor/n).is_file() for n in required)
versions={n:version(vendor/n) for n in required if n.endswith('.exe')}
assert set(versions.values())=={'5.5.22.0'},versions
pins={n:sha(vendor/n) for n in required};identity=pins['bin/mysqld.exe'][:12]
out=P/'target/current/owned-database-toolchains'/('mysql-5.5.22-'+identity)
assert out.resolve().is_relative_to(R.resolve());out.mkdir(parents=True,exist_ok=True)
for n,h in pins.items():
    src=vendor/n;dst=out/n;dst.parent.mkdir(parents=True,exist_ok=True)
    if dst.exists():assert plain(dst).is_file() and sha(dst)==h
    else:shutil.copyfile(src,dst)
    assert sha(src)==sha(dst)==h
connector=Path(ref['formal']).parent/'lib/mysql-connector-j-8.0.33.jar'
assert plain(connector).is_file()
cached=P/'target/current/server-reference/lib'/connector.name;cached.parent.mkdir(parents=True,exist_ok=True)
connector_sha=sha(connector)
if cached.exists():assert plain(cached).is_file() and sha(cached)==connector_sha
else:shutil.copyfile(connector,cached)
assert sha(cached)==connector_sha
pe=(out/'bin/mysqld.exe').read_bytes();offset=struct.unpack_from('<I',pe,0x3c)[0];assert pe[offset:offset+4]==b'PE\0\0'
machine=struct.unpack_from('<H',pe,offset+4)[0];assert machine in [0x14c,0x8664]
record=dict(status='private_toolchain_and_JDBC_prepared_no_executable_invocation',vendor_root=str(vendor),bundle_root=str(out),
    formal_target_sha256=ref['formal_sha256'],versions=versions,machine=hex(machine),files=pins,
    total_bytes=sum((out/n).stat().st_size for n in pins),
    JDBC=dict(source=str(connector),source_sha256=connector_sha,cached=str(cached),cached_sha256=sha(cached)),
    data_directories_copied=False,original_configuration_copied=False,executables_invoked=0,database_started=False,SQL_executed=False,tests_executed=0)
manifest=out/'bundle.json'
if manifest.exists():assert read(manifest)==record
else:manifest.write_bytes(data(record))
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
assert all(sha(R/n)==h for n,h in inputs.items()) and all(sha(p)==h for p,h in normal['normal_product_pins'].items())
assert sha(ref['formal'])==ref['formal_sha256']
print(json.dumps(dict(bundle=str(manifest),files=len(pins),bytes=record['total_bytes'],JDBC=str(cached),JDBC_sha256=connector_sha,executables_invoked=0,database_started=False),ensure_ascii=False))
