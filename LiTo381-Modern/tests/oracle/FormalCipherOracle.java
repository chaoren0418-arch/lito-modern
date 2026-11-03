import java.lang.reflect.*;
import java.net.*;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.HexFormat;

/** Read-only oracle: instantiate only the deployed JAR's cipher, no server/SQL. */
public final class FormalCipherOracle {
    public static void main(String[] args) throws Exception {
        Path jar = Path.of(args[0]);
        String sha = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        try (URLClassLoader loader = new URLClassLoader(new URL[]{jar.toUri().toURL()}, ClassLoader.getPlatformClassLoader())) {
            if (args.length>1 && args[1].equals("--version-contract")) {
                Class<?> protocol=loader.loadClass("local.lito381.server.Protocol");
                String[] names={"C_VERSION","S_KEY","S_VERSION_CHECK","VERSION_MAGIC","VERSION_PRODUCT","VERSION_BUILD"};
                long[] values=new long[names.length];
                for(int i=0;i<names.length;i++){Field f=protocol.getDeclaredField(names[i]);f.setAccessible(true);values[i]=((Number)f.get(null)).longValue();}
                byte[] request=java.nio.ByteBuffer.allocate(10).order(java.nio.ByteOrder.LITTLE_ENDIAN)
                    .put((byte)values[0]).putShort((short)values[3]).putInt((int)values[4]).put((byte)0).putShort((short)values[5]).array();
                Method parse=protocol.getDeclaredMethod("versionRequest",byte[].class);parse.setAccessible(true);
                Method supported=protocol.getDeclaredMethod("supportedVersion",loader.loadClass("local.lito381.server.Protocol$VersionRequest"));supported.setAccessible(true);
                if(!(boolean)supported.invoke(null,parse.invoke(null,(Object)request)))throw new IllegalStateException("contract request");
                Method accepted=protocol.getDeclaredMethod("versionAccepted"),rejected=protocol.getDeclaredMethod("versionRejected");accepted.setAccessible(true);rejected.setAccessible(true);
                System.out.println("{\"schema\":1,\"jar_sha256\":\""+sha+"\",\"request\":\""+HexFormat.of().formatHex(request)+
                    "\",\"accepted\":\""+HexFormat.of().formatHex((byte[])accepted.invoke(null))+"\",\"rejected\":\""+HexFormat.of().formatHex((byte[])rejected.invoke(null))+"\",\"supported\":true}");
                return;
            }
            Class<?> type = loader.loadClass("local.lito381.server.Protocol$CipherState");
            Constructor<?> ctor = type.getDeclaredConstructor(int.class);
            Method encrypt = type.getDeclaredMethod("encrypt", byte[].class);
            Method value = type.getDeclaredMethod("value");
            ctor.setAccessible(true); encrypt.setAccessible(true); value.setAccessible(true);
            StringBuilder out = new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"sequences\":[");
            int[] seeds = {0, 0x80000000, 0xfffffffe, 0xffffffff};
            for (int s=0; s<seeds.length; s++) {
                if (s>0) out.append(',');
                Object cipher = ctor.newInstance(seeds[s]);
                out.append("{\"seed\":").append(Integer.toUnsignedLong(seeds[s])).append(",\"packets\":[");
                int[] sizes = {1,2,10,27,128,155};
                for (int p=0; p<sizes.length; p++) {
                    if (p>0) out.append(',');
                    byte[] body = new byte[sizes[p]];
                    for (int i=0; i<body.length; i++) body[i]=(byte)(i*37+p*11);
                    long before = (long)value.invoke(cipher);
                    byte[] encoded = (byte[])encrypt.invoke(cipher, (Object)body);
                    out.append("{\"plain\":\"").append(HexFormat.of().formatHex(body))
                       .append("\",\"cipher\":\"").append(HexFormat.of().formatHex(encoded))
                       .append("\",\"before\":\"").append(Long.toUnsignedString(before))
                       .append("\",\"after\":\"").append(Long.toUnsignedString((long)value.invoke(cipher))).append("\"}");
                }
                out.append("]}");
            }
            System.out.println(out.append("]}"));
        }
    }
}
