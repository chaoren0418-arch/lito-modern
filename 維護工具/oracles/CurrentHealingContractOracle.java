package local.lito381.server;

import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.HexFormat;

/** Catalog and actual codec observations only: no sessions or game authority. */
public final class CurrentHealingContractOracle {
    public static void main(String[] args) throws Exception {
        Path jar = Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        String profile = System.getProperty("lito.resourceProfile", "lito-legacy");
        if (!profile.equals("lito-legacy") && !profile.equals("tw13081901-v1")) throw new AssertionError("Unreviewed resource profile");
        StringBuilder json = new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha)
            .append("\",\"resource_profile\":\"").append(profile).append("\",\"spells\":[");
        boolean comma = false;
        for (int id : new int[] {1, 19, 35, 57}) {
            var spell = BattleSpellCatalog.healingSpell(id);
            if (spell == null || spell.skillId() != id) throw new AssertionError("Healing catalog drift");
            byte[] request = new Protocol.Writer().c(20).c(spell.row()).c(spell.column()).d(2).bytes();
            if (Protocol.objectSpellRequest(request).skillId() != id) throw new AssertionError("Object request selector drift");
            byte[] effect = Protocol.effectObject(2, spell.effectGfx());
            if (effect.length != 7 || effect[0] != 0x37) throw new AssertionError("Object effect envelope drift");
            int wireGfx = Byte.toUnsignedInt(effect[5]) | (Byte.toUnsignedInt(effect[6]) << 8);
            if (comma) json.append(',');
            comma = true;
            json.append("{\"skill\":").append(id).append(",\"catalog_gfx\":").append(spell.effectGfx())
                .append(",\"wire_gfx\":").append(wireGfx).append(",\"action\":").append(spell.actionId())
                .append(",\"mp_cost\":").append(spell.mpCost()).append(",\"reuse_ms\":").append(spell.reuseDelayMillis())
                .append(",\"maximum_range\":").append(spell.maximumRange()).append(",\"request\":\"")
                .append(HexFormat.of().formatHex(request)).append("\",\"effect\":\"")
                .append(HexFormat.of().formatHex(effect)).append("\"}");
        }
        System.out.println(json.append("],\"actual_game_sessions\":0,\"gameplay_verified\":false}"));
    }
}
