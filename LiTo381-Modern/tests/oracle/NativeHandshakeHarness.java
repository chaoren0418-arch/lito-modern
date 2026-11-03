import java.io.*;
import java.lang.reflect.*;
import java.net.*;
import java.nio.*;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;

/** Isolated loopback TCP harness using deployed Protocol methods. No Main,
 * ClientSession, repository, config, SQL driver or player account is constructed. */
public final class NativeHandshakeHarness {
    static Method method(Class<?> type, String name, Class<?>... args) throws Exception {
        Method m=type.getDeclaredMethod(name,args);m.setAccessible(true);return m;
    }
    static int constant(Class<?> type,String name) throws Exception {
        Field f=type.getDeclaredField(name);f.setAccessible(true);return f.getInt(null);
    }
    static final class Fragmented extends OutputStream {
        private final OutputStream target;
        Fragmented(OutputStream target){this.target=target;}
        @Override public void write(int value) throws IOException {target.write(value);target.flush();}
        @Override public void write(byte[] raw,int offset,int length) throws IOException {
            for(int i=0;i<length;i++)write(raw[offset+i]);
        }
    }
    public static void main(String[] args) throws Exception {
        Path jar=Path.of(args[0]),ready=Path.of(args[1]);String mode=args[2];
        if(!Set.of("accept","reject","malformed","header-eof","body-eof","stall").contains(mode))throw new IllegalArgumentException("mode");
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        try(URLClassLoader loader=new URLClassLoader(new URL[]{jar.toUri().toURL()},ClassLoader.getPlatformClassLoader());
            ServerSocket listener=new ServerSocket(0,1,InetAddress.getByName("127.0.0.1"))) {
            listener.setSoTimeout(12000);
            Class<?> protocol=loader.loadClass("local.lito381.server.Protocol");
            Class<?> cipherType=loader.loadClass("local.lito381.server.Protocol$CipherState");
            Constructor<?> ctor=cipherType.getDeclaredConstructor(int.class);ctor.setAccessible(true);
            Method encrypt=method(cipherType,"encrypt",byte[].class),decrypt=method(cipherType,"decrypt",byte[].class),value=method(cipherType,"value");
            Method read=method(protocol,"readFrame",InputStream.class,int.class),write=method(protocol,"writeFrame",OutputStream.class,byte[].class);
            Method parse=method(protocol,"versionRequest",byte[].class);
            Method supported=method(protocol,"supportedVersion",loader.loadClass("local.lito381.server.Protocol$VersionRequest"));
            Method accepted=method(protocol,"versionAccepted"),rejected=method(protocol,"versionRejected");
            Files.writeString(ready,"{\"port\":"+listener.getLocalPort()+",\"jar_sha256\":\""+sha+"\",\"mode\":\""+mode+"\"}",StandardOpenOption.CREATE_NEW);
            try(Socket socket=listener.accept()) {
                socket.setTcpNoDelay(true);socket.setSoTimeout(10000);
                int seed=0xfffffffe;
                byte[] key=ByteBuffer.allocate(5).order(ByteOrder.LITTLE_ENDIAN).put((byte)constant(protocol,"S_KEY")).putInt(seed).array();
                OutputStream out=new Fragmented(socket.getOutputStream());
                write.invoke(null,out,key);
                Object incoming=ctor.newInstance(seed),outgoing=ctor.newInstance(seed);
                byte[] encrypted=(byte[])read.invoke(null,socket.getInputStream(),65535);
                if(encrypted==null)throw new EOFException("client did not send version");
                byte[] plain=(byte[])decrypt.invoke(incoming,(Object)encrypted);
                Object request=parse.invoke(null,(Object)plain);
                boolean valid=(boolean)supported.invoke(null,request);
                if(!valid)throw new IllegalStateException("new client version rejected by actual Protocol.supportedVersion");
                byte[] response=(byte[])(mode.equals("reject")?rejected:accepted).invoke(null);
                if(mode.equals("malformed")){response=Arrays.copyOf(response,response.length+1);response[response.length-1]=42;}
                byte[] cipher=(byte[])encrypt.invoke(outgoing,(Object)response);
                if(mode.equals("stall")){Thread.sleep(7000);}
                else if(mode.equals("header-eof")){out.write((cipher.length+2)&255);}
                else if(mode.equals("body-eof")){out.write((cipher.length+2)&255);out.write((cipher.length+2)>>>8);out.write(cipher,0,3);}
                else {write.invoke(null,out,cipher);while(socket.getInputStream().read()!=-1){} }
                System.out.println("{\"status\":\"completed\",\"mode\":\""+mode+"\",\"version_valid\":true,\"request_bytes\":"+plain.length+
                    ",\"rx_counter\":\""+Long.toUnsignedString((long)value.invoke(incoming))+"\",\"tx_counter\":\""+Long.toUnsignedString((long)value.invoke(outgoing))+
                    "\",\"sql_access\":false,\"jar_sha256\":\""+sha+"\"}");
            }
        }
    }
}
