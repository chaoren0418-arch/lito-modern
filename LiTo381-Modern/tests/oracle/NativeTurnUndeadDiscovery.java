package local.lito381.server;
public final class NativeTurnUndeadDiscovery {
 public static void main(String[] args) {
  for(int map:new int[]{0,4,513,527}) {
   for(var row:ClassicWorldMonsterCatalog.definitions(map)) {
    var npc=row.template();var stats=SourceNpcPhysicalStats.memoryFixture(npc.npcId());
    if(BattleTurnUndead.eligible(stats))System.out.println("ELIGIBLE map="+map+" spawn="+row.spawnId()+" npc="+npc.npcId()+" count="+row.count()+" point="+row.locX()+","+row.locY()+" rect="+row.locX1()+","+row.locY1()+","+row.locX2()+","+row.locY2()+" gfx="+npc.gfxId()+" HP="+npc.hp()+" MR="+stats.magicResistance()+" IsTU="+stats.turnUndeadEligible()+" undead="+stats.undead());
   }
  }
 }
}