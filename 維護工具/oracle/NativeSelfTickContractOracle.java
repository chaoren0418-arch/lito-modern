package local.lito381.server;

import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;

/** Pinned codec execution only; does not create sessions or cast a spell. */
public final class NativeSelfTickContractOracle {
    private static byte[] request(long tick) {
        return new byte[]{0x14,7,3,(byte)tick,(byte)(tick>>>8),(byte)(tick>>>16),(byte)(tick>>>24)};
    }
    private static boolean rejected(byte[] bytes) {
        try { Protocol.selfSpellRequest(bytes); return false; }
        catch (IllegalArgumentException expected) { return true; }
    }
    public static void main(String[] args) throws Exception {
        Path loaded=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
        if (!loaded.equals(Path.of(args[0]).toRealPath())) throw new IllegalStateException("Pinned Protocol origin differs");
        String profile=System.getProperty("lito.resourceProfile");
        if (!"lito-legacy".equals(profile) && !"tw13081901-v1".equals(profile)) throw new IllegalArgumentException("Profile");
        StringBuilder json=new StringBuilder("{\"profile\":\"").append(profile).append("\",\"positive\":[");
        long[] ticks={0,2,0x12345678L,0x7fffffffL,0x80000000L,0xffffffffL};
        for (int i=0;i<ticks.length;i++) {
            byte[] bytes=request(ticks[i]);var parsed=Protocol.selfSpellRequest(bytes);
            if (parsed.skillId()!=60 || parsed.clientTick()!=ticks[i]) throw new IllegalStateException("Self tick roundtrip differs");
            if(i!=0)json.append(',');
            json.append("{\"hex\":\"").append(HexFormat.of().formatHex(bytes)).append("\",\"skill_id\":60,\"tick\":").append(ticks[i]).append('}');
        }
        int negative=0;byte[] valid=request(0x12345678L);
        for(int length=0;length<7;length++) {
            if(!rejected(Arrays.copyOf(valid,length)))throw new IllegalStateException("Truncated self tick accepted");
            negative++;
        }
        for(int length:new int[]{8,9,11}) {
            if(!rejected(Arrays.copyOf(valid,length)))throw new IllegalStateException("Trailing self tick bytes accepted");
            negative++;
        }
        for(int field=0;field<3;field++) {
            byte[] bad=valid.clone();bad[field]=(byte)(field==0?0x15:field==1?28:8);
            if(!rejected(bad))throw new IllegalStateException("Invalid self tick selector accepted");
            negative++;
        }
        System.out.println(json.append("],\"negative_rejections\":").append(negative)
            .append(",\"actual_game_sessions\":0,\"actual_casts\":0,\"native_clock_source_verified\":false}"));
    }
}
