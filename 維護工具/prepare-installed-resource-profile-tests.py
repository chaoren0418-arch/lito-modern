"""Stage current-profile tests while retaining every legacy oracle and entry point."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re,difflib
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256']
assert source=='B0628AC904FE5AB7C2227FF176811A4A6208EAC7B5BA8892C73D0AFD99F2F2C4'
oracle=V/'native-installed-Text-reference/20261019-200910/readback.json';proof=read(oracle);current=proof['results'][0]
assert current['count']==37497 and current['unique_names']==28361 and current['ambiguous_names']==110
assert current['list_spr_sha256']=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127'
assert current['file_hashes']['Text.idx']=='CF4D420DDAB51BADB9CFF20F60C9ABB2E04156404A443897FBB8A44FED909D7E'
planner=(T/'prepare-current-shared-resource-library-plan.py').read_text('utf-8-sig');tree=ast.parse(planner)
node=next(n for n in tree.body if isinstance(n,ast.FunctionDef)and n.name=='function_body')
exec(compile(ast.Module(body=[node],type_ignores=[]),'sealed-function-extractor','exec'))
out=T/'candidates'/('installed-resource-profiles-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
changed={};before={};diffs=[];newtests=[]
def body(text,name):
    pos=text.index('fn '+name+'(');line=text[:pos].count('\n')+1
    return function_body(text,name,max(1,line-1))[0]
specs=[('disintegrate_assets.rs','all_four_shared_sprites_match_independent_csharp_pixels'),
    ('disintegrate_effect.rs','shared_one_shots_match_csharp_composites_and_release_owned_assets'),
    ('world_scene.rs','real_disintegrate_layers_share_world_depth_and_survive_target_removal')]
for file,name in specs:
    path=R/'LiTo381-Modern/src/modern_lito'/file;old=path.read_text('utf-8-sig');original=body(old,name)
    helper=name+'_with_profile';newname=name+'_current_launcher_tw_profile'
    assert original.count('DisintegrateAssets::load(')==1
    openbrace=original.index('{');inner=original[openbrace:].replace('DisintegrateAssets::load(','profile_loader(')
    replacement='fn '+name+'() {\n        '+helper+'(crate::modern_lito::disintegrate_assets::DisintegrateAssets::load);\n    }\n\n'
    replacement+='    #[test]\n    #[ignore = "LITO_SHARED_RESOURCE_ROOT; explicit current launcher TW profile; original CSharp SPR/pixel/clock assertions retained, no legacy root fallback"]\n'
    replacement+='    fn '+newname+'() {\n        '+helper+'(crate::modern_lito::disintegrate_assets::DisintegrateAssets::load_current_tw);\n    }\n\n'
    replacement+='    fn '+helper+'(profile_loader: fn(&std::path::Path, &std::sync::atomic::AtomicBool) -> anyhow::Result<crate::modern_lito::disintegrate_assets::DisintegrateAssets>) '+inner
    new=old.replace(original,replacement);assert new!=old
    rel=path.relative_to(R).as_posix();before[rel]=sha(path);(out/file).write_text(new,encoding='utf-8',newline='\n');changed[rel]=sha(out/file);newtests.append(newname)
    diffs.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=rel,tofile=rel))
file='tw_action_catalog.rs';path=R/'LiTo381-Modern/src/modern_lito'/file;old=path.read_text('utf-8-sig');name='shared_tw_pack_has_real_ultimate_definitions_without_base_catalog_fallback';original=body(old,name)
newname=name+'_current_launcher_tw_profile';extra=original.replace('fn '+name+'(', 'fn '+newname+'(')
extra=extra.replace('SharedTwActionCatalog::load_selected(','SharedTwActionCatalog::load_selected_from_launcher(')
extra=extra.replace('resources::read_shared_tw_action_pack(', 'resources::read_shared_launcher_tw_action_pack(')
assert extra.count('fc7b4cac77379a98e2a2edf3f645bb82ca8be9ee3f2b7fae76fcc61423ee8798')==1
extra=extra.replace('fc7b4cac77379a98e2a2edf3f645bb82ca8be9ee3f2b7fae76fcc61423ee8798','b3788df4c7f077ef3ff15b070a5d020ccb224e48dec3757c4cd34528fca4b218')
new=old.replace(original,original+'\n\n    #[test]\n    #[ignore = "LITO_SHARED_RESOURCE_ROOT; exact current launcher TW pack and unchanged plaintext/definitions; no root-pack fallback"]\n    '+extra)
rel=path.relative_to(R).as_posix();before[rel]=sha(path);(out/file).write_text(new,encoding='utf-8',newline='\n');changed[rel]=sha(out/file);newtests.append(newname)
diffs.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=rel,tofile=rel))
file='resources.rs';path=R/'LiTo381-Modern/src/modern_lito'/file;old=path.read_text('utf-8-sig')
anchor='''        assert_eq!(archive.len(), 37_496);
        assert_eq!(archive.unique_name_count(), 28_360);''';assert old.count(anchor)==1
replacement='''        // Separate installed reference, independently read with CSharp
        // Lin.Helper.Core 1.5.4 on 2026-10-20. Pin both complete source files;
        // retain the historical reference and all payload/ambiguity assertions.
        let installed_index = std::fs::read(Path::new(&root).join("Text.idx")).unwrap();
        let installed_pak = std::fs::read(Path::new(&root).join("Text.pak")).unwrap();
        let current_profile = format!("{:X}", Sha256::digest(&installed_index))
            == "CF4D420DDAB51BADB9CFF20F60C9ABB2E04156404A443897FBB8A44FED909D7E"
            && format!("{:X}", Sha256::digest(&installed_pak))
                == "19BFE9D44C55F697D55CA23292509AEA56DBBC1F534427AFC740522081B7448C";
        let (records, names) = if current_profile { (37_497, 28_361) } else { (37_496, 28_360) };
        assert_eq!(archive.len(), records);
        assert_eq!(archive.unique_name_count(), names);'''
new=old.replace(anchor,replacement);rel=path.relative_to(R).as_posix();before[rel]=sha(path);(out/file).write_text(new,encoding='utf-8',newline='\n');changed[rel]=sha(out/file)
diffs.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=rel,tofile=rel))
(out/'change.diff').write_text(''.join(diffs),encoding='utf-8')
record=dict(base_source_manifest_sha256=source,before_files=before,candidate_files=changed,changed_files=list(changed),created_files=[],
    independent_Text_oracle=str(oracle),independent_Text_oracle_sha256=sha(oracle),new_ignored_tests=newtests,
    product_runtime_code_changed=False,legacy_TW_root_entry_points_and_all_original_assertions_retained=True,
    current_loader_already_requires_pack_and_plaintext_hashes=True,source_or_runtime_started=False,compiled=False,applied=False,whole_goal_complete=False)
(out/'candidate.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(candidate=str(out),changed_files=len(changed),new_ignored_tests=len(newtests),applied=False,compiled=False),ensure_ascii=False))
