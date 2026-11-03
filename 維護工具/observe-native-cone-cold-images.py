"""Match every saved candidate PNG to its actual frame receipt and archive pin."""
from pathlib import Path
import argparse,hashlib,json
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def resolved(p):
    original=Path(p).resolve(strict=True);value=str(original)
    if value.startswith('\\\\?\\') and len(value)>6 and value[4].isalpha() and value[5]==':':
        normal=Path(value[4:]).resolve(strict=True);assert original.samefile(normal);return normal
    return original
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--qualification',type=Path,required=True);ap.add_argument('--viewed',nargs='*',default=[]);a=ap.parse_args()
run=a.run.resolve(strict=True);assert any(run.is_relative_to(ROOT/'驗證紀錄/current'/n)for n in ['native-cone-cold-live','native-cone-cold-class2-regression'])
qpath=a.qualification.resolve(strict=True);assert qpath.is_relative_to(run);q=read(qpath);assert q['status']=='independently_qualified_isolated_C02_scene_admission_and_refusal'
r=read(run/'receipt.json');runtime=resolved(r['runtime']);assert r['all_started_processes_terminal'] and not r['owned_processes']
references={}
def walk(value,source):
    if isinstance(value,dict):
        if isinstance(value.get('png'),str)and isinstance(value.get('receipt'),dict)and isinstance(value['receipt'].get('rgba_sha256'),str):
            path=resolved(value['png']);assert path.is_relative_to(runtime);references.setdefault(path,[]).append(dict(receipt_file=str(source),rgba_sha256=value['receipt']['rgba_sha256'].upper()))
        for v in value.values():walk(v,source)
    elif isinstance(value,list):
        for v in value:walk(v,source)
for path in runtime.rglob('*.json'):
    assert path.stat().st_size<=64*1024*1024;walk(read(path),path)
rows=[];viewed=set(a.viewed);seen=set()
for path in sorted(runtime.rglob('*.png')):
    assert path.resolve().is_relative_to(runtime)and not path.is_symlink();name=path.relative_to(runtime).as_posix();assert sha(path)==q['raw_files'][name]
    with Image.open(path)as im:
        assert im.size==(800,600)and im.mode=='RGBA';rgba=hashlib.sha256(im.tobytes()).hexdigest().upper()
    refs=references.get(path.resolve(),[]);assert refs and all(v['rgba_sha256']==rgba for v in refs),(name,'saved image receipt mismatch')
    actual_view=name in viewed;seen.add(name)
    rows.append(dict(path=str(path),archive_member=name,png_sha256=sha(path),rgba_sha256=rgba,receipt_references=refs,actually_viewed=actual_view))
assert rows and viewed<=seen
out=run/'image-observation.json';assert not out.exists()
record=dict(status='all_owned_diagnostic_PNGs_match_pinned_actual_frame_receipts',qualification=str(qpath),qualification_sha256=sha(qpath),base_source_manifest_sha256=q['base_source_manifest_sha256'],kind=q['kind'],compile_profile=q['compile_profile'],fixture_profile=q['profile'],images=rows,image_count=len(rows),actually_viewed_count=len(viewed),actual_C02_sessions=q['actual_sessions'],normal_product_registered=False,GUI_verified=False,physical_audio_verified=False,pixel_brightness_qualified=False,whole_goal_complete=False)
out.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(observation=str(out),matched_images=len(rows),actually_viewed=len(viewed),GUI_verified=False),ensure_ascii=False))
