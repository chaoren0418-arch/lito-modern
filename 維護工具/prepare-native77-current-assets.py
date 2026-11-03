"""Separate explicit current TW data loading from the preserved root contract."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
source=ROOT/'LiTo381-Modern/src/modern_lito/disintegrate_assets.rs'
out=ROOT/'維護工具/candidates/native77-current-assets';out.mkdir()
before=source.read_text(encoding='utf-8-sig')
old='''        let catalog = SharedTwActionCatalog::load_selected(root, &IDS.into(), cancel)?;
        qualify(catalog.catalog())?;'''
assert before.count(old)==1
new='''        let catalog = SharedTwActionCatalog::load_selected(root, &IDS.into(), cancel)?;
        Self::from_catalog(root, catalog, cancel)
    }
    /// Explicit current TW asset-data namespace. This never changes the legacy
    /// root-pack entry point, merges base definitions, or grants spell permission.
    pub fn load_current_tw(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let catalog = SharedTwActionCatalog::load_selected(&root.join("launcher"), &IDS.into(), cancel)?;
        require_current_profile(catalog.pack_sha256(), catalog.text_sha256(), catalog.decoded_bytes(), catalog.definitions().len())?;
        Self::from_catalog(root, catalog, cancel)
    }
    fn from_catalog(root: &Path, catalog: SharedTwActionCatalog, cancel: &AtomicBool) -> Result<Self> {
        qualify(catalog.catalog())?;'''
after=before.replace(old,new)
pack=bytes.fromhex('B3788DF4C7F077EF3FF15B070A5D020CCB224E48DEC3757C4CD34528FCA4B218')
text=bytes.fromhex('7EBE2B464C8B1EA9C58AC985DD8FF4C823784D8231894262EDDCD7691CA94008')
constants='\nconst CURRENT_PACK: [u8;32] = '+str(list(pack))+';\nconst CURRENT_TEXT: [u8;32] = '+str(list(text))+''';
fn require_current_profile(pack:[u8;32],text:[u8;32],decoded:usize,definitions:usize)->Result<()> {
    ensure!(pack==CURRENT_PACK && text==CURRENT_TEXT && decoded==5_974_928 && definitions==4,"unqualified current TW disintegrate namespace");
    Ok(())
}
'''
position=after.index('\nfn qualify(');after=after[:position]+constants+after[position:]
test='''
    #[test]
    fn current_tw_namespace_requires_exact_profile_and_cancels_before_io() {
        require_current_profile(CURRENT_PACK,CURRENT_TEXT,5_974_928,4).unwrap();
        for index in [0,15,31] {
            let mut bad=CURRENT_PACK;bad[index]^=1;assert!(require_current_profile(bad,CURRENT_TEXT,5_974_928,4).is_err());
            let mut bad=CURRENT_TEXT;bad[index]^=1;assert!(require_current_profile(CURRENT_PACK,bad,5_974_928,4).is_err());
        }
        for decoded in [0,5_974_927,5_974_929] {assert!(require_current_profile(CURRENT_PACK,CURRENT_TEXT,decoded,4).is_err());}
        for selected in [0,3,5] {assert!(require_current_profile(CURRENT_PACK,CURRENT_TEXT,5_974_928,selected).is_err());}
        assert!(DisintegrateAssets::load_current_tw(Path::new("does-not-exist"),&AtomicBool::new(true)).err().unwrap().to_string().contains("cancelled"));
    }
'''
position=after.rfind('\n}');after=after[:position]+test+after[position:]
(out/'disintegrate_assets.rs').write_text(after,encoding='utf-8',newline='\n')
(out/'profile-gate.rs').write_text(constants,encoding='utf-8',newline='\n');(out/'added-test.rs').write_text(test,encoding='utf-8',newline='\n')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
record=dict(status='prepared_explicit_current_TW_disintegrate_asset_entry',source=str(source),source_sha256=sha(source),candidate=str(out/'disintegrate_assets.rs'),candidate_sha256=sha(out/'disintegrate_assets.rs'),
    preserved_root_entry=True,shared_existing_loader_body=True,explicit_asset_subdirectory='launcher',pack_sha256=pack.hex().upper(),text_sha256=text.hex().upper(),decoded_bytes=5974928,selected_definitions=4,
    original_qualification_and_cancellation_checks_preserved=True,production_source_modified=False,new_casts=0,GUI_verified=False,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(record,ensure_ascii=False))
