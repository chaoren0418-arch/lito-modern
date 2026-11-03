"""Bind the current 71-case selection to its actual ignored-test source audit."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,ast,hashlib,json,re
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
SOURCE='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
LEGACY={
'modern_lito::disintegrate_assets::tests::all_four_shared_sprites_match_independent_csharp_pixels',
'modern_lito::disintegrate_effect::tests::shared_one_shots_match_csharp_composites_and_release_owned_assets',
'modern_lito::tw_action_catalog::tests::shared_tw_pack_has_real_ultimate_definitions_without_base_catalog_fallback',
'modern_lito::world_runtime::npc_fire_storm_runtime_tests::actual_npc70_runtime_preloads_once_and_renders_one_owned_group_without_replay',
'modern_lito::world_scene::tests::real_disintegrate_layers_share_world_depth_and_survive_target_removal'}
parser=argparse.ArgumentParser();parser.add_argument('library',type=Path);a=parser.parse_args()
base=a.library.resolve(strict=True);assert base.is_relative_to(V/'native-installed-resource-profile-library-default')
qpath=base/'Debug/full-default-qualified/qualified.json';q=read(qpath);assert q['source_manifest_sha256']==SOURCE and q['failed']==q['Rust_exit_code']==0
assert(q['passed'],q['ignored'],q['inventory'])==(1975,488,2463)
audit_path=base/'Debug/inventory-audit/ignored-source-reasons.json';audit=read(audit_path);inputs=read(base/'compiled-inputs.json');assert len(inputs)==1049
planner=(T/'prepare-current-shared-resource-library-plan.py').read_text('utf-8-sig');tree=ast.parse(planner)
node=next(n for n in tree.body if isinstance(n,ast.FunctionDef)and n.name=='function_body');exec(compile(ast.Module(body=[node],type_ignores=[]),'sealed-extractor','exec'))
available={x['test']:x for x in audit['tests']if len(x.get('candidates',[]))==1 and 'LITO_SHARED_RESOURCE_ROOT'in x['candidates'][0]['reason']}
assert len(available)==76 and LEGACY<=set(available)
selected=[];bodies={};operations=[];helpers={}
for name,row in available.items():
    if name in LEGACY:continue
    c=row['candidates'][0];p=R/c['source'];assert sha(p)==c['source_sha256']==inputs[c['source']]
    text=p.read_text('utf-8-sig');body,start,end=function_body(text,name.split('::')[-1],c['ignore_line'])
    selected.append(dict(test=name,source=c['source'],source_sha256=c['source_sha256'],ignore_reason=c['reason'],start_line=start,end_line=end,body_sha256=hashlib.sha256(body.encode()).hexdigest().upper()))
    bodies[name]=body
    # Current wrappers share the complete legacy pixel/clock proof, changing
    # only the explicit, already hash-qualified loader passed to that helper.
    if name.endswith('_current_launcher_tw_profile')and 'tw_action_catalog::'not in name:
        helper=name.split('::')[-1].removesuffix('_current_launcher_tw_profile')+'_with_profile'
        pos=text.index('fn '+helper+'(');line=text[:pos].count('\n')+1
        value,_,_=function_body(text,helper,max(1,line-1));helpers[helper]=dict(source=c['source'],body=value,sha256=hashlib.sha256(value.encode()).hexdigest().upper())
        assert 'DisintegrateAssets::load_current_tw'in body and 'profile_loader'in value
    ops=[line.strip()for line in body.splitlines()if re.search(r'File::create|OpenOptions|create_dir|write_image|Command::|TcpListener|TcpStream|UdpSocket|CreateWindow|SendInput|waveOut',line)]
    if ops:operations.append(dict(test=name,operations=ops))
assert len(selected)==71 and len(helpers)==3
assert len([x for x in selected if x['test'].endswith('_current_launcher_tw_profile')])==4
out=V/'native-installed-resource-profile-shared-resource-plan'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,value in [('test-bodies.json',bodies),('shared-helper-bodies.json',helpers)]:
    (out/name).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
record=dict(status='current_installed_resource71_selection_prepared_no_execution',source_manifest_sha256=SOURCE,
    library=str(base),Debug_qualification=str(qpath),Debug_qualification_sha256=sha(qpath),audit=str(audit_path),audit_sha256=sha(audit_path),
    selected_tests=selected,expected_tests=71,excluded_legacy_tests={n:available[n]for n in sorted(LEGACY)},
    exclusion_reason='Five explicit legacy root-TW cases require their original root pack; current launcher profile is separately named and qualified. Preserve legacy contracts, do not substitute its raw hash.',
    selected_bodies=str(out/'test-bodies.json'),selected_bodies_sha256=sha(out/'test-bodies.json'),shared_helpers=str(out/'shared-helper-bodies.json'),shared_helpers_sha256=sha(out/'shared-helper-bodies.json'),
    explicit_operation_review=operations,review_completed=False,compiler_or_runtime_started=False,
    additional_filters_require_help_and_list_validation=True,legacy_output_preservation_required=True,GUI_verified=False,whole_goal_complete=False)
(out/'plan.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'planner.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(plan=str(out/'plan.json'),selected=71,legacy_excluded=5,shared_helpers=3,executed=False),ensure_ascii=False))
