"""Readonly full-method review. Never loads game classes or invokes tests."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
def main():
 primary=V/'native-E149-reduction88-bounce89-inspection/20261024-180259-031734/inspection.json';require(sha(primary)=='0CC173E96CBAEF16917ABB07C256D3C706E6E9D984B5AE13D0DE7DCC16920587','primary changed');j=read(primary);check_pins(j['source_pins']);require(all(sha(p)==h for p,h in j['external_readonly_pins'].items()),'external changed')
 def body(name):
  rows=[Path(p).read_text('utf-8-sig')for h,p in j['method_files'].items()if ' '+name+'('in h];require(len(rows)==1,'method differs '+name);return rows[0]
 def at(s,offset,text):require(re.search(r'^\s*'+str(offset)+r':[^\n]*'+re.escape(text),s,re.M),'original offset differs '+str(offset)+' '+text)
 for name,kind,activation,hp,mp,act,effect,hpwire,mpwire in [('useReductionArmorSpell','ReductionArmor',512,525,542,670,683,907,955),('useBounceAttackSpell','BounceAttack',524,537,554,682,695,919,967)]:
  s=body(name)
  for offset,text in {1:'Protocol.selectorOnlySpellRequest:',242:kind+'Spell_effectiveMpCost_III:',456:kind+'Spell.hpCost:',459:'if_icmpgt     470',462:'279',476:'if_icmpge     484',479:'278',activation:'BattleAbilityStatus.activate'+kind+':',hp:'Field playerHp:I',mp:'Field playerMp:I',act:'Protocol.objectAction:',effect:'Protocol.effectObject:',hpwire:'13',mpwire:'Protocol.manaPoint:'}.items():at(s,offset,text)
  require('Phase.BATTLE:'in s and 'Phase.WORLD:'in s and 'monitorenter'in s and 'commitSpellReady:'in s,'admission/commit differs')
 at(body('useBounceAttackSpell'),986,'ownCharacterAttrDef:')
 s=body('selectorOnlySpellRequest');at(s,30,'Reader.remaining:');at(s,33,'ifeq          47');require('invalid selector-only C_USE_SPELL layout'in s,'selector trailing bytes differ')
 s=body('applyBounceAttackWeaponDamage')
 for offset,text in {7:'soulOfFlameActive:',36:'BattleAbilityStatus.bounceAttack:',43:'remainingSeconds:',54:'present:',61:'ranged:',74:'ThreadLocalRandom.nextInt:',84:'BattleBounceAttack.decide:',129:'damagesWeapon:',142:'damageAttackingWeapon:'}.items():at(s,offset,text)
 expected=[dict(skillId=88,row=10,column=7,actionId=19,effectGfx=4824,mpCost=5,hpCost=50,reuseDelayMillis=3000,durationSeconds=192),dict(skillId=89,row=11,column=0,actionId=19,effectGfx=4648,mpCost=5,hpCost=120,reuseDelayMillis=5000,durationSeconds=64)]
 for row,want in zip(j['definitions'],expected):require(all(row[k]==v for k,v in want.items()),'formal record differs')
 profiles=[]
 for gfx,sound,source in [(4824,3089,3),(4648,1924,4)]:
  raw=Path(j['raw_definitions'][str(gfx)]['path']).read_text('utf-8-sig');require('104.attr(8)'in raw and '109.effect(0 -1)'in raw,'class differs');a=re.search(r'0.animation\(0 (\d+),([^)]*)\)',raw);frames=re.findall(r'(\d+)\.(\d+):(\d+)(?:<(\d+))?',a[2]);require(len(frames)==int(a[1]) and [(i,int(f[3]))for i,f in enumerate(frames)if f[3]]==[(source,sound)],'frames/audio differ');ticks=[int(f[2])for f in frames];require(all(f[0]=='0'and int(f[1])==(len(frames)-1 if i==0 else i-1)for i,f in enumerate(frames)),'source order differs');profiles.append(dict(graphics=gfx,frames=len(frames),ticks=ticks,overlap_sound=sound,overlap_source=source,audio_model_ms=sum(t*1000//24 for t in ticks[1:source]),finite_model_ms=sum(t*1000//24 for t in ticks)))
 out=V/'native-E149-reduction88-bounce89-review'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'reviewer.py').write_bytes(Path(__file__).read_bytes());pins=dict(j['source_pins']);pins.update({str(p):sha(p)for p in [primary,out/'reviewer.py']});p=out/'review.json';p.write_bytes(encoded(dict(schema='E149-reduction88-bounce89-reviewed-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,external_readonly_pins=j['external_readonly_pins'],primary=str(primary),primary_sha256=sha(primary),definitions=j['definitions'],raw_definitions=j['raw_definitions'],profiles=profiles,selector_only_no_target=True,strict_server_hp_greater_than_cost_279=True,server_cost_status_and_reuse_commit=True,bounce_own_authoritative_AC_packet=True,bounce_weapon_damage_server_only=True,finite_screen_overlap_audio_not_status_ack=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(review=str(p),sha256=sha(p),profiles=profiles,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
