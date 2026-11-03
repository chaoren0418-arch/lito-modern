"""Prepare a real C02 heal-event handoff test; only diagnostic delivery is deferred."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];src=ROOT/'維護工具/candidates/native-teleport-presentation/probe.rs';d=ROOT/'維護工具/candidates/native-teleport-audio';d.mkdir(exist_ok=True)
s=src.read_text(encoding='utf-8');assert not (d/'probe.rs').exists()
s=s.replace('    previous: Value,','    previous: Value,\n    defer_audio: bool,\n    held: Vec<m::native_audio_delivery::PreparedCue>,\n    held_meta: Value,\n    deliveries: Vec<Value>,',1)
s=s.replace('            previous: Value::Null,','            previous: Value::Null,\n            defer_audio: false,\n            held: vec![],\n            held_meta: Value::Null,\n            deliveries: vec![],',1)
start=s.index('        if let Some(frame) = self.view.frame.as_ref() {');end=s.index('        let obs = json!',start)
s=s[:start]+'''        if let Some(frame) = self.view.frame.as_ref() {
            let rows=frame.receipt["audio"].as_array().context("audio receipt rows")?.iter().filter(|row|row["prepared"]==true).collect::<Vec<_>>();
            ensure!(rows.len()==frame.native_audio.len(),"prepared event alignment");
            let own=s.world.status.context("audio owner")?.object_id;
            for (cue,row) in frame.native_audio.iter().zip(rows) {
                let mine=row["event"]["actor"]==own && row["event"]["action"]==0 && row["event"]["sound"]==109 && cue.group().selected(cue.selection())==Some(109);
                if self.defer_audio && mine {
                    if self.held.is_empty() {
                        self.held.push(cue.clone());
                        self.held_meta=json!({"epoch":epoch,"generation":s.world.generation,"row":row,"world_effects":s.world.effects,"own_casts":s.world.own_cast_packets,"delivered109_before":self.sink.played.iter().filter(|&&id|id==109).count()});
                    }
                    continue;
                }
                let outcome=cue.submit(&scope,&mut self.sink)?;
                ensure!(outcome!=m::native_audio_delivery::DeliveryOutcome::Unbound,"published audio cue unbound");
                if mine && matches!(outcome,m::native_audio_delivery::DeliveryOutcome::Played(m::combat_audio::PlayOutcome::Started{sound_id:109,..})) {
                    self.deliveries.push(json!({"epoch":epoch,"generation":s.world.generation,"row":row,"outcome":format!("{outcome:?}")}));
                }
            }
        }
'''+s[end:]
pos=s.index('    fn wait(&mut self,')
methods='''    fn heal_audio(&mut self,defer:bool)->Result<Value> {
        self.ready(8)?;
        ensure!(self.c.snapshot().world.learned_skills.contains(&1),"original heal1 book missing");
        let before=self.c.snapshot().clone();let delivered=self.sink.played.iter().filter(|&&id|id==109).count();let own_delivered=self.deliveries.len();
        self.defer_audio=defer;self.held.clear();self.held_meta=Value::Null;
        let request=m::native_skills::request(&before.world,1,None)?;self.c.cast(1,None,self.view.frame.as_ref().context("heal input frame")?)?;
        let immediate=self.c.snapshot().clone();
        ensure!(before.world.hp==immediate.world.hp && before.world.mp==immediate.world.mp && serde_json::to_value(&before.inventory)?==serde_json::to_value(&immediate.inventory)?,"queue predicted authority");
        self.wait(10,|p|if defer {!p.held.is_empty()}else{p.deliveries.len()>own_delivered} || matches!(p.c.snapshot().skill_feedback,m::native_skills::Feedback::Rejected(_)|m::native_skills::Feedback::NoResponse|m::native_skills::Feedback::Interrupted))?;
        ensure!(self.c.snapshot().skill_feedback==m::native_skills::Feedback::CastAction && self.c.snapshot().world.own_cast_packets==before.world.own_cast_packets+1,"heal1 not acknowledged {:?}",self.c.snapshot().skill_feedback);
        let own=self.c.snapshot().world.status.context("heal own")?.object_id;
        ensure!(self.c.snapshot().world.effects.iter().any(|e|e.graphics==11722 && e.target.is_some_and(|t|t.0==own)),"canonical actual heal11722 absent");
        if defer {ensure!(self.held.len()==1 && self.sink.played.iter().filter(|&&id|id==109).count()==delivered,"source sound consumed before handoff");}
        else {ensure!(self.deliveries.len()==own_delivered+1,"new scene heal sound was not delivered once");}
        Ok(json!({"before":before,"immediately_after_queue":immediate,"after_ack":self.c.snapshot(),"request_hex":request.iter().map(|b|format!("{b:02x}")).collect::<String>(),"deferred":defer,"held":self.held_meta,"delivered109_before":delivered,"delivered109_after":self.sink.played.iter().filter(|&&id|id==109).count(),"own_deliveries_before":own_delivered,"own_deliveries_after":self.deliveries.len(),"own_delivery_events":self.deliveries}))
    }
'''
s=s[:pos]+methods+s[pos:]
needle='        p.ready(8)?;\n        let before = p.c.snapshot().clone();';assert needle in s
s=s.replace(needle,'        let source_audio=p.heal_audio(true)?;\n        p.ready(8)?;\n        let before = p.c.snapshot().clone();',1)
s=s.replace('let old_cues = p.view.frame.as_ref().unwrap().native_audio.clone();','let old_cues = std::mem::take(&mut p.held);\n        let held_source_meta=p.held_meta.clone();\n        ensure!(old_cues.len()==1,"authentic unplayed source cue absent");',1)
needle='''        for cue in &old_cues {
            retired_cues.push(format!("{:?}", cue.submit(&scope, &mut p.sink)?));
        }''';assert needle in s
s=s.replace(needle,'''        let mut repeated_cues=vec![];
        for cue in &old_cues {
            let result=cue.submit(&scope,&mut p.sink)?;ensure!(result==m::native_audio_delivery::DeliveryOutcome::Stale,"unplayed source cue escaped retirement: {result:?}");retired_cues.push(format!("{result:?}"));
            let duplicate=cue.clone().submit(&scope,&mut p.sink)?;ensure!(duplicate==m::native_audio_delivery::DeliveryOutcome::AlreadyConsumed,"retired cue clone replayed");repeated_cues.push(format!("{duplicate:?}"));
        }''')
needle='"cue_outcomes":retired_cues,';assert needle in s;s=s.replace(needle,needle+'"repeated_cue_outcomes":repeated_cues,"held_source_meta":held_source_meta,"authentic_unplayed_source_cue_rejected":true,',1)
s=s.replace('        drop(old_cues);','        drop(old_cues);p.held.clear();p.defer_audio=false;',1)
needle='''        std::fs::write(
            root.join(format!("{role}-handoff.json")),''';assert needle in s
s=s.replace(needle,'''        let destination_audio=p.heal_audio(false)?;
        std::fs::write(
            root.join(format!("{role}-handoff.json")),''',1)
s=s.replace('"source_picture":source_picture,','"source_audio":source_audio,"destination_audio":destination_audio,"source_picture":source_picture,',1)
(d/'probe.rs').write_text(s,encoding='utf-8',newline='\n');(d/'generator.py').write_bytes(Path(__file__).read_bytes())
prep=dict(status='prepared_real_heal109_unplayed_source_cue_handoff_candidate',base_source_manifest_sha256='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182',original_probe_sha256=hashlib.sha256(src.read_bytes()).hexdigest().upper(),real_requests_only=True,source_spell=1,expected_server_effect=11722,expected_original_sound=109,diagnostic_source_delivery_deferred=True,books_or_authority_modified=False,product_modified=False,compiled=False,executed=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(d/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(d)
