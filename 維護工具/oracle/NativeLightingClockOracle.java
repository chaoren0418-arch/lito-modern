package local.lito381.server;

import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.time.Instant;
import java.time.ZoneOffset;
import java.util.HexFormat;

/** Calls the unmodified, package-private clock in the pinned reference JAR. */
public final class NativeLightingClockOracle {
    public static void main(String[] args) throws Exception {
        Path expected = Path.of(args[0]).toRealPath();
        Path actual = Path.of(ClassicLineageClock.class.getProtectionDomain()
                .getCodeSource().getLocation().toURI()).toRealPath();
        if (!expected.equals(actual)) throw new AssertionError("Unexpected clock code source");
        String hash = HexFormat.of().withUpperCase().formatHex(
                MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(actual)));
        if (!hash.equals("C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539"))
            throw new AssertionError("Unexpected reference JAR");
        System.out.println("kind,input,result,taipei_second,utc_second,offset_seconds");
        long base = 1735689600000L;
        for (long delta = 0; delta <= 2000; delta++) epoch(base + delta);
        for (long delta : new long[]{-1, 10_000, 60_000, 3_600_000, 86_400_000,
                31L*86_400_000, 365L*86_400_000, 99_999_999_999L,
                357_913_940_999L, 357_913_941_000L, 357_913_941_499L,
                357_913_941_500L, Long.MAX_VALUE-base}) epoch(base + delta);
        for (int raw : new int[]{-1, 0, 1, 86_399, 86_400, 142_041_600,
                299_548_800, 315_532_800, 338_466_069, 1_000_000_000,
                Integer.MAX_VALUE}) raw(raw);
    }

    private static void epoch(long input) {
        try {
            int result = ClassicLineageClock.atEpochMillis(input);
            System.out.println("epoch," + input + "," + result + ",,,");
        } catch (IllegalArgumentException | ArithmeticException error) {
            System.out.println("epoch," + input + ",ERROR:" + error.getClass().getSimpleName() + ",,,");
        }
    }

    private static void raw(int input) {
        try {
            int taipei = ClassicLineageClock.nativeClientSecondOfDay(input);
            Instant instant = Instant.ofEpochSecond(input);
            int utc = instant.atZone(ZoneOffset.UTC).toLocalTime().toSecondOfDay();
            int offset = ClassicLineageClock.NATIVE_CLIENT_ZONE.getRules()
                    .getOffset(instant).getTotalSeconds();
            System.out.println("raw," + input + "," + taipei + "," + taipei + "," + utc + "," + offset);
        } catch (IllegalArgumentException error) {
            System.out.println("raw," + input + ",ERROR:" + error.getClass().getSimpleName() + ",,,");
        }
    }
}
