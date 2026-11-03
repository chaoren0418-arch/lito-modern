"""Instrument one isolated resource module; never edit normal product bodies."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';T=ROOT/'維護工具'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
original=P/'src/modern_lito/resources.rs';out=T/'candidates/native8CC5-resource-open-profile';out.mkdir()
body=original.read_text('utf-8-sig')
profile_code='''
static DIAGNOSTIC_SHARED_ARCHIVE_PROFILE:std::sync::OnceLock<std::sync::Mutex<Vec<serde_json::Value>>>=std::sync::OnceLock::new();
pub fn diagnostic_shared_archive_profile()->Vec<serde_json::Value>{
    DIAGNOSTIC_SHARED_ARCHIVE_PROFILE.get_or_init(Default::default).lock().unwrap_or_else(|e|e.into_inner()).clone()
}
fn diagnostic_archive_event(value:serde_json::Value){
    let mut events=DIAGNOSTIC_SHARED_ARCHIVE_PROFILE.get_or_init(Default::default).lock().unwrap_or_else(|e|e.into_inner());
    if events.len()<4096 {events.push(value);}
}
'''
body+=profile_code
needle='''    fn open_index(
        supplied_root: &Path,
        kind: SharedArchiveKind,
        limits: ResourceLimits,
        selection: Option<(BTreeSet<String>, &AtomicBool)>,
    ) -> Result<Self, ResourceError> {'''
assert body.count(needle)==1
wrapper='''    fn open_index(
        supplied_root: &Path,
        kind: SharedArchiveKind,
        limits: ResourceLimits,
        selection: Option<(BTreeSet<String>, &AtomicBool)>,
    ) -> Result<Self, ResourceError> {
        let started=std::time::Instant::now();
        let selected=selection.as_ref().map(|(names,_)|names.len());
        let result=Self::diagnostic_original_open_index(supplied_root,kind,limits,selection);
        let elapsed=started.elapsed().as_micros();
        diagnostic_archive_event(serde_json::json!({"stage":"archive_open","kind":format!("{kind:?}"),
            "thread":format!("{:?}",std::thread::current().id()),"thread_name":std::thread::current().name(),
            "elapsed_us":elapsed,"selected_names":selected,"success":result.is_ok(),
            "records":result.as_ref().ok().map(|archive|archive.len()),
            "error":result.as_ref().err().map(|e|format!("{e:?}"))}));
        result
    }

    fn diagnostic_original_open_index(
        supplied_root: &Path,
        kind: SharedArchiveKind,
        limits: ResourceLimits,
        selection: Option<(BTreeSet<String>, &AtomicBool)>,
    ) -> Result<Self, ResourceError> {'''
body=body.replace(needle,wrapper,1)
needle='        let entries = if let Some((names, cancel)) = &selection {';assert body.count(needle)==1
body=body.replace(needle,'        let diagnostic_parse_start=std::time::Instant::now();\n'+needle,1)
needle='        let record_count = entries.values().map(Vec::len).sum();';assert body.count(needle)==1
body=body.replace(needle,needle+'''
        diagnostic_archive_event(serde_json::json!({"stage":"index_parse","kind":format!("{kind:?}"),
            "thread":format!("{:?}",std::thread::current().id()),"thread_name":std::thread::current().name(),
            "elapsed_us":diagnostic_parse_start.elapsed().as_micros(),"records":record_count,
            "index_bytes":index_size,"pak_bytes":pak_size}));''',1)
(out/'resources.rs').write_text(body,encoding='utf-8',newline='\n')
probe=T/'candidates/native8CC5-scene-cost/profile.rs';text=probe.read_text('utf-8-sig')
text=text.replace('normal_source_bodies_unchanged":true','normal_source_bodies_unchanged":false')
text=text.replace('"results":results,"network_sessions":0','"results":results,"source_overlays_used":true,"resource_open_profile":lito_modern::modern_lito::resources::diagnostic_shared_archive_profile(),"network_sessions":0',1)
text=text.replace('threads.push(std::thread::spawn(move||measure(input,out,role,w,e,barrier)));','threads.push(std::thread::Builder::new().name(format!("scene-cost-{role}")).spawn(move||measure(input,out,role,w,e,barrier))?);',1)
(out/'profile.rs').write_text(text,encoding='utf-8',newline='\n')
record=dict(status='prepared_uncompiled_resource_open_instrumentation',original_resource=dict(path=str(original),sha256=sha(original)),
    candidate_resource=dict(path=str(out/'resources.rs'),sha256=sha(out/'resources.rs')),
    original_probe=dict(path=str(probe),sha256=sha(probe)),candidate_probe=dict(path=str(out/'profile.rs'),sha256=sha(out/'profile.rs')),
    source_overlays_used=True,normal_source_bodies_unchanged=False,normal_Model_body_unchanged=True,normal_product_source_modified=False,
    instrumentation='Wrap exact original open_index body; record per-thread whole-open and parse time/count/bytes. Return the original result unchanged.',
    gameplay_qualification_granted=False,G11_qualified=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),normal_product_source_modified=False,compiled=False),ensure_ascii=False))
