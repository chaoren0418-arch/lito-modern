#[cfg(test)]mod finite_scene_history_development_tests {
 use super::*;
 use super::super::native_skill_effects::finite_history_development_tests::{history_for,has_seen};
 fn history(offset:u64)->FiniteHistory {
  FiniteHistory{animation:super::super::native_world_animation::Animator::default().history(),skills:std::array::from_fn(|i|history_for(7,offset+i as u64)),many:super::super::native_attack_many::Effects::default().history(),spell_projectiles:super::super::native_spell_projectiles::Projectiles::default().history(),class1:super::super::native_class1::Effects::default().history(),disintegrate:super::super::native_disintegrate::Effects::default().history(),call_lightning:super::super::native_call_lightning::Effects::default().history(),cone_cold:super::super::native_cone_cold::Effects::default().history(),restore_other_effects:true}
 }
 #[test]fn all_four_profile_sequences_are_retained_only_for_the_original_source(){let h=history(11);assert_eq!(h.active_spell_sequences(),std::collections::BTreeSet::from([11,12,13,14]));let retired=h.for_source(false);assert!(retired.active_spell_sequences().is_empty());assert!(retired.for_source(true).active_spell_sequences().is_empty());}
 #[test]fn retired_worker_admission_is_merged_into_each_profile_without_importing_its_graph(){let mut current=history(11);current.merge_disintegrate_admission(&history(21));for(i,h)in current.skills.iter().enumerate(){assert!(has_seen(h,21+i as u64));assert_eq!(h.active_sequences(),std::collections::BTreeSet::from([11+i as u64]));}assert_eq!(current.active_spell_sequences(),std::collections::BTreeSet::from([11,12,13,14]));}
}
