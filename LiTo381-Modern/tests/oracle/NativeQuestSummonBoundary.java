package local.lito381.server;
public final class NativeQuestSummonBoundary {
 public static void main(String[] args) throws Exception {
  for(var c:NativePeerHarness.fixture("turn-undead-quest").characters("PeerFixture"))System.out.println("INITIAL requestedHP60000 object="+c.objectId()+" recordMaxHP="+c.maxHp()+" runtimeMaxHP="+NativeCharacterStats.runtimeVital(c.maxHp()));
  var s=BattleTimeAttackMagicSkillCatalog.summonForNpc(45601);
  System.out.println("SUMMON "+s);
  for(int d=0;d<=10;d++)System.out.println("DIST "+d+" "+s.triggerDistance(d));
  for(int r=1;r<=7;r++)System.out.println("ROLL "+r+" "+s.triggerRoll(r));
  System.out.println("ACTION19 "+BattleAnimationCatalog.get().supportsTimedAction(240,19));
  System.out.println("ACTION18 "+BattleAnimationCatalog.get().supportsTimedAction(240,18));
 }
}
