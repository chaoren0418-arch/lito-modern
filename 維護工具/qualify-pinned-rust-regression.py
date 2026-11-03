"""Qualify one pinned Release89+main run, preserving exact test executables."""
from pathlib import Path
import argparse,datetime,hashlib,json,re,zipfile
root=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
    return h.hexdigest().upper()
def save(p,d):p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def parse(log,suites):
    rows={};pending=None
    for line in log.splitlines():
        m=re.match(r'\s+Running (unittests src\\main\.rs|tests\\([A-Za-z0-9_]+)\.rs) \(([^)]+\.exe)\)',line)
        if m:
            assert pending is None,'previous suite lacks terminal result'
            name=m[2] or 'main_unit';assert name not in rows
            rows[name]={'product_log_path':m[3]};pending=name
        m=re.match(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',line)
        if m:
            assert pending is not None,'unattributed result block'
            rows[pending].update(passed=int(m[2]),failed=int(m[3]),ignored=int(m[4]),result=m[1]);pending=None
    assert pending is None and set(rows)==set(suites)|{'main_unit'},'missing/extra suite or incomplete log'
    assert len(suites)==89 and len(rows)==90
    assert all(v['result']=='ok' and v['failed']==0 for v in rows.values())
    return rows
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--parse-only',action='store_true');ap.add_argument('--expect-passed',type=int);args=ap.parse_args()
run=args.run.resolve(strict=True);assert run.is_relative_to(root/'驗證紀錄/current/full-rust-release')
receipt=read(run/'receipt.json');logpath=run/'cargo-tests.log';rows=parse(logpath.read_text(encoding='utf-8-sig'),receipt['suites']);passed=sum(v['passed'] for v in rows.values());ignored=sum(v['ignored'] for v in rows.values())
if args.expect_passed is not None:assert passed==args.expect_passed,(passed,args.expect_passed)
if args.parse_only:
    print(json.dumps({'status':'parsed_log_only_no_source_or_product_qualification','suites':len(rows)-1,'passed':passed,'ignored':ignored,'failed':0}));raise SystemExit(0)
assert receipt['exit_code']==0 and receipt['sources_unchanged'] and not receipt.get('library_unit_tests',True)
manifest=read(run/'build-inputs-pre.json');assert manifest['profile']=='Release' and len(manifest['files'])==962
for n,h in manifest['files'].items():assert sha(root/n)==h,('source drift',n)
main=Path(manifest['main']['path']);assert sha(main)==manifest['main']['sha256']==receipt['main_sha256']
products={};deps=(root/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/release/deps').resolve()
for name,row in rows.items():
    p=(root/'LiTo381-Modern'/row['product_log_path']).resolve(strict=True);assert p.is_relative_to(deps)
    products[name]={'path':str(p),'sha256':sha(p)}
known_report=read(root/'驗證紀錄/current/native-room-selection/release-product-retention.json');known_archive=Path(known_report['archive']);assert sha(known_archive)==known_report['archive_sha256'];known={}
with zipfile.ZipFile(known_archive) as z:
    for name,item in known_report['products'].items():
        entry='Release/'+name+'.exe';assert hashlib.sha256(z.read(entry)).hexdigest().upper()==item['sha256'];known[item['sha256']]={'archive':str(known_archive),'entry':entry,'sha256':item['sha256']}
extra=run/'additional-test-products.zip';temp=run/'additional-test-products.tmp.zip';assert not extra.exists() and not temp.exists(),'retention archive already exists; inspect rather than overwrite'
retained={}
with zipfile.ZipFile(temp,'w',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,item in products.items():
        if item['sha256'] in known:retained[name]=known[item['sha256']];continue
        entry=name+'.exe';z.write(item['path'],entry);retained[name]={'archive':str(extra),'entry':entry,'sha256':item['sha256']}
with zipfile.ZipFile(temp) as z:
    for name,item in retained.items():
        if item['archive']==str(extra):assert hashlib.sha256(z.read(item['entry'])).hexdigest().upper()==item['sha256'],name
temp.rename(extra)
retention={'status':'all90_test_products_retained_SHA_readback_verified','products':retained,'additional_archive':str(extra),'additional_archive_sha256':sha(extra),'reused_archive':str(known_archive),'reused_archive_sha256':sha(known_archive),'whole_game_complete':False};save(run/'product-retention.json',retention)
qualified={'status':'verified_current_Release89_integration_suites_and_main_units','qualified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'passed':passed,'ignored':ignored,'failed':0,'integration_suites':89,'main_unit_result':{k:rows['main_unit'][k] for k in ['passed','failed','ignored']},'library_unit_tests':False,'main_sha256':receipt['main_sha256'],'source_count':962,'all_sources_unchanged':True,'per_suite':{n:{k:v[k] for k in ['passed','failed','ignored']} for n,v in rows.items()},'products':products,'log_sha256':sha(logpath),'build_manifest_sha256':sha(run/'build-inputs-pre.json'),'retention_receipt_sha256':sha(run/'product-retention.json'),'qualifier_sha256':sha(Path(__file__)),'full_game_complete':False};save(run/'qualified.json',qualified)
receipt.update(status=qualified['status'],tests_passed=passed,tests_ignored=ignored,tests_failed=0,qualified_receipt_sha256=sha(run/'qualified.json'));save(run/'receipt.json',receipt)
stage=root/'驗證紀錄/current/native-room-selection'
for f in ['qualified.json','progress.json']:
    p=stage/f;d=read(p);d['new_full_regression_status']=qualified['status'];d['new_full_regression_receipt']=str(run/'qualified.json');d['new_full_regression_passed']=passed;d['new_full_regression_ignored']=ignored;d['new_full_regression_session']=None;d['broad_regression_session']=None;save(p,d)
print(json.dumps({'status':qualified['status'],'passed':passed,'ignored':ignored,'failed':0,'retained_products':len(products),'additional_products':sum(x['archive']==str(extra) for x in retained.values())}))