package local.lito381.server;

import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.time.Instant;
import java.time.ZonedDateTime;
import java.time.zone.ZoneOffsetTransition;
import java.time.zone.ZoneRules;
import java.time.zone.ZoneRulesProvider;
import java.util.HexFormat;
import java.util.List;

/** Export C02's declared calendar policy; does not resolve the old client's CRT. */
public final class NativeReferenceCalendarOracle {
    private static long samples;
    private static String sha(Path path) throws Exception {
        return HexFormat.of().withUpperCase().formatHex(
                MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(path)));
    }
    private static void emit(PrintWriter output, long raw) {
        if (raw < 0 || raw > Integer.MAX_VALUE) return;
        int reference = ClassicLineageClock.nativeClientSecondOfDay((int) raw);
        ZonedDateTime date = Instant.ofEpochSecond(raw).atZone(ClassicLineageClock.NATIVE_CLIENT_ZONE);
        if (reference != date.toLocalTime().toSecondOfDay()) throw new AssertionError("Reference calendar");
        output.println(raw + "," + date.getOffset().getTotalSeconds() + "," + date.getYear()
                + "," + (date.getDayOfYear() - 1) + "," + date.getMonthValue() + "," + date.getDayOfMonth()
                + "," + date.getHour() + "," + date.getMinute() + "," + date.getSecond() + "," + reference);
        samples++;
    }
    public static void main(String[] args) throws Exception {
        if (args.length != 2) throw new IllegalArgumentException("Expected pinned JAR and output directory");
        Path jar = Path.of(args[0]).toRealPath();
        Path actual = Path.of(ClassicLineageClock.class.getProtectionDomain()
                .getCodeSource().getLocation().toURI()).toRealPath();
        String expected = "C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539";
        if (!actual.equals(jar) || !sha(actual).equals(expected)) throw new AssertionError("Code source");
        if (!ClassicLineageClock.NATIVE_CLIENT_ZONE.getId().equals("Asia/Taipei"))
            throw new AssertionError("Reference declared zone changed");
        Path out = Path.of(args[1]);
        ZoneRules rules = ClassicLineageClock.NATIVE_CLIENT_ZONE.getRules();
        List<ZoneOffsetTransition> transitions = rules.getTransitions().stream()
                .filter(t -> t.toEpochSecond() >= 0 && t.toEpochSecond() <= Integer.MAX_VALUE).toList();
        try (PrintWriter output = new PrintWriter(Files.newBufferedWriter(out.resolve("transitions.csv"),
                StandardCharsets.UTF_8))) {
            output.println("raw,offset_before,offset_after");
            for (ZoneOffsetTransition transition : transitions)
                output.println(transition.toEpochSecond() + "," + transition.getOffsetBefore().getTotalSeconds()
                        + "," + transition.getOffsetAfter().getTotalSeconds());
            if (output.checkError()) throw new AssertionError("Transition output");
        }
        try (PrintWriter output = new PrintWriter(Files.newBufferedWriter(out.resolve("calendar.csv"),
                StandardCharsets.UTF_8))) {
            output.println("raw,offset,year,yday0,month1,day,hour,minute,second,reference_second");
            int[] withinDay = {0, 1, 53999, 54000, 54001, 57599, 57600, 57601, 86399};
            for (long day = 0; day <= Integer.MAX_VALUE / 86400L; day++)
                for (int within : withinDay) emit(output, day * 86400 + within);
            for (ZoneOffsetTransition transition : transitions)
                for (int delta = -2; delta <= 2; delta++) emit(output, transition.toEpochSecond() + delta);
            emit(output, Integer.MAX_VALUE - 1L);
            emit(output, Integer.MAX_VALUE);
            if (output.checkError()) throw new AssertionError("Calendar output");
        }
        String version = ZoneRulesProvider.getVersions("Asia/Taipei").lastKey();
        System.out.println("{\"status\":\"formal_reference_declared_calendar_vectors_exported\","
                + "\"jar_sha256\":\"" + expected + "\",\"zone\":\"Asia/Taipei\","
                + "\"tzdb_version\":\"" + version + "\",\"epoch_zero_offset\":"
                + rules.getOffset(Instant.EPOCH).getTotalSeconds() + ",\"transitions\":" + transitions.size()
                + ",\"calendar_samples\":" + samples
                + ",\"old_client_CRT_UTC_or_local_resolved\":false,\"actual_game_sessions\":0}");
    }
}
