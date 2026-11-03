"""Create one reviewable native Release candidate, without copying game assets."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,zipfile

ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--startup-run',type=Path,required=True);ap.add_argument('--resource-manifest',type=Path,required=True);a=ap.parse_args()
startup=a.startup_run.resolve(strict=True);resources=a.resource_manifest.resolve(strict=True)
assert startup.is_relative_to(ROOT/'驗證紀錄/current/native-startup') and resources.is_relative_to(ROOT/'驗證紀錄/current/native-release-resource-manifest')
s=read(startup/'cohort.json');q=read(startup/'qualified/qualified.json');res=read(resources/'progress.json')
assert q['status']=='independently_qualified_native_startup_settings_and_normal_entry_scope' and q['observed_terminal_exit_code']==0
assert q['source_manifest_sha256']==s['source_manifest_sha256']
assert res['status']=='retained_full_SHA256_for_declared_external_asset_namespaces' and res['metadata_and_file_set_unchanged_before_after'] and res['whole_file_SHA256']
assert sha(resources/'resource-files.json')==res['manifest_sha256']
external=read(resources/'resource-files.json');assert len(external)==res['files'] and sum(v['bytes'] for v in external.values())==res['bytes']
inputs=read(startup/'compiled-inputs.json');assert len(inputs)==1025
for name,digest in inputs.items():assert sha(ROOT/name)==digest,name
assert sha(ROOT/'啟動現代客戶端.ps1')==q['entry_sha256']
release=s['profiles']['Release']['normal'];assert sha(release['path'])==release['sha256']==q['profiles']['Release']['normal_sha256']
jar=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
jar_hash=sha(jar);assert jar_hash=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
base=ROOT/'交付';base.mkdir(exist_ok=True);package=base/'Windows-x64-候選';assert not package.exists();package.mkdir()
archive=base/'LiToModern-Windows-x64-candidate.zip';assert not archive.exists()
evidence=ROOT/'驗證紀錄/current/native-release-package'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');evidence.mkdir(parents=True)
(evidence/'producer.py').write_bytes(Path(__file__).read_bytes())
exe=package/'LiToModern.exe';shutil.copyfile(release['path'],exe);assert sha(exe)==release['sha256']
shutil.copyfile(startup/'compiled-inputs.json',package/'build-inputs.json')
shutil.copyfile(resources/'resource-files.json',package/'resource-files.json')
save(package/'native-settings.example.json',dict(schema=1,login_endpoint='192.168.1.10:32000',resource_root=r'D:\遊戲資源'))
spec=importlib.util.spec_from_file_location('native_release_pe_inspector',ROOT/'維護工具/inspect-release-dependencies.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
dependencies=module.inspect(exe)
for key in ['ordinary_imports','delayed_imports']:dependencies[key]=sorted(dependencies[key],key=lambda name:(name.lower(),name))
dependencies['dependencies']=sorted(dependencies['dependencies'],key=lambda row:(row['name'].lower(),row['name']))
save(evidence/'PE-dependencies.json',dependencies)
assert dependencies['machine']=='AMD64' and dependencies['optional_header']=='PE32+' and dependencies['subsystem']==2
assert all(row['api_set_contract'] or row['current_host_system_file_exists'] for row in dependencies['dependencies'])
cwd=evidence/'empty-cwd';cwd.mkdir();settings=evidence/'測試設定 路徑';settings.mkdir();config=settings/'原生登入.json'
save(config,dict(schema=1,login_endpoint='127.0.0.1:32000',resource_root=r'D:\lineage開發\0-客戶端'))
invalid=settings/'不合格設定.json';save(invalid,read(config)|{'schema':2})
env=os.environ.copy();windows=Path(env.get('SystemRoot',r'C:\Windows'));env['PATH']=str(windows/'System32')+os.pathsep+str(windows)
for key in list(env):
    if key.startswith(('LITO_','CARGO_','RUST')) or key in ['JAVA_HOME','JDK_HOME','JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS']:env.pop(key,None)
checks={}
for label,args,expected in [('version',['--version'],0),('config',['--check-native-config',str(config)],0),('invalid-config',['--check-native-config',str(invalid)],2)]:
    result=subprocess.run([str(exe),*args],cwd=cwd,env=env,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    (evidence/(label+'-stdout.log')).write_bytes(result.stdout);(evidence/(label+'-stderr.log')).write_bytes(result.stderr)
    assert result.returncode==expected and not result.stderr,(label,result.returncode)
    report=json.loads(result.stdout.decode('utf-8-sig'));checks[label]=dict(exit_code=result.returncode,report=report)
assert checks['version']['report']['default_entry']=='native_login_setup' and checks['version']['report']['pointer_bits']==64
assert checks['config']['report']['status']=='valid_native_settings' and not checks['config']['report']['network_started'] and not checks['config']['report']['settings_written']
assert checks['invalid-config']['report']['status']=='failed'
assert not (package/'LiToModern.native.json').exists() and not (cwd/'LiToModern.native.json').exists()
manifest=dict(schema=1,application='LiToModern',distribution='Windows-x64-development-candidate',architecture='AMD64/PE32+',
    executable=dict(file='LiToModern.exe',sha256=sha(exe),bytes=exe.stat().st_size,unique_entry_build=True),
    source=dict(declared_inputs=1024,compiled_inputs=1025,source_manifest_sha256=q['source_manifest_sha256'],compiled_input_list='build-inputs.json',compiled_input_list_sha256=sha(package/'build-inputs.json')),
    external_resources=dict(copied=False,manifest='resource-files.json',manifest_sha256=res['manifest_sha256'],declared_selection=res['selection'],files=res['files'],bytes=res['bytes'],whole_file_SHA256=True,all_formats_or_gameplay_verified=False),
    protocol_reference=dict(server_jar_sha256=jar_hash,resource_profile='tw13081901-v1',server_packaged=False,launcher_deployment_required=False),
    dependencies=dict(ordinary_imports=dependencies['ordinary_imports'],delayed_imports=dependencies['delayed_imports'],bundled_runtime_DLLs=[],current_host_system_dependencies_available=True),
    validation=dict(startup_scoped_tests={profile:info['counts'] for profile,info in q['profiles'].items()},packaged_noninteractive_loader_checks=True,developer_tool_directories_removed_from_PATH=True,external_unicode_config_path_with_spaces=True,GUI_verified=False,physical_input_verified=False,clean_machine_verified=False,portable_package_accepted=False,full_current_regression_verified=False,whole_goal_complete=False),
    created_utc=datetime.now(timezone.utc).isoformat())
save(package/'manifest.json',manifest)
guide='''# LiTo Windows x64 開發候選

解壓縮後開啟 `LiToModern.exe`。首次啟動輸入登入伺服器的 IP:連接埠，以及既有遊戲資源根目錄的本機絕對路徑；再於原生登入視窗輸入帳密。開啟登入視窗不會自動連線。

勾選「記住伺服器與資源路徑」，會把這兩項設定存入本資料夾的 `LiToModern.native.json`。帳密不保存。未勾選則只用於本次啟動。`native-settings.example.json` 是範例，不會自動套用；請使用實際伺服器與資源路徑。

重新設定可在 PowerShell 執行：

```powershell
& .\\LiToModern.exe --configure-native
```

不開視窗的版本及設定檢查：

```powershell
& .\\LiToModern.exe --version | Out-String
& .\\LiToModern.exe --check-native-config "$pwd\\LiToModern.native.json" | Out-String
```

設定檢查只驗證格式、端點與目錄可讀性，不連線或寫設定，也不代表整套資源或遊戲流程已通過。

本包使用外部資源與既有伺服器。`manifest.json` 記錄 EXE、來源及協定參考版本；`build-inputs.json` 是編譯輸入清單，`resource-files.json` 是宣告資源範圍的完整檔案 SHA-256 清單。包內不含舊 LiTo.bin、登入器、JAR 或遊戲資源本體。

目前已通過同來源的啟動相關測試（Debug 29、Release 25）、唯一入口建置、封包外的設定檢查，以及本機空工作目錄／限系統 PATH 的套件載入。這仍是開發候選；新設定視窗的實際操作、乾淨電腦、完整玩法及穩定性尚未完成驗收。相依 DLL 名稱列於 manifest；本包未附 Microsoft C++ 執行階段，本機由系統提供相依 DLL。
'''
(package/'README.md').write_text(guide,encoding='utf-8',newline='\n')
files={f.name:sha(f) for f in package.iterdir() if f.is_file()};assert len(files)==6
assert sha(exe)==release['sha256'] and sha(jar)==jar_hash
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=6) as z:
    for name in sorted(files):z.write(package/name,'LiToModern-Windows-x64/'+name)
with zipfile.ZipFile(archive) as z:
    assert len(z.namelist())==len(files)
    for name,digest in files.items():
        with z.open('LiToModern-Windows-x64/'+name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==digest
save(evidence/'receipt.json',dict(status='created_current_native_Release_development_candidate_and_verified_archive',package=str(package),archive=str(archive),archive_sha256=sha(archive),package_files=files,startup_qualification=str(startup/'qualified/qualified.json'),startup_qualification_sha256=sha(startup/'qualified/qualified.json'),external_resource_manifest=str(resources/'resource-files.json'),external_resource_manifest_sha256=res['manifest_sha256'],checks=checks,clean_machine_verified=False,GUI_verified=False,portable_package_accepted=False,whole_goal_complete=False,finished_utc=datetime.now(timezone.utc).isoformat()))
print(json.dumps(dict(package=str(package),archive=str(archive),evidence=str(evidence),files=len(files),exe_sha256=sha(exe),clean_machine_verified=False),ensure_ascii=False))
