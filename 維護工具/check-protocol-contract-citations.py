#!/usr/bin/env python3
"""Check that PROTOCOL-CONTRACT.md's key citations point at the stated source line.
usage: python3 -I 維護工具/check-protocol-contract-citations.py <lito-launcher-server root>"""
import sys, pathlib, subprocess
root = pathlib.Path(sys.argv[1])
srv = root / "lito伺服器/LiTo381-Server/src/main/java/local/lito381/server"
lau = root / "lito登入器/LiTo381-Launcher/src"
P, CS, GSS, NB = "Protocol.java", "ClientSession.java", "GameStateStore.java", "NumericBreakthrough.java"
checks = [
 (srv/P,33,"KEY_HIGH = 0x87546CA1L"),(srv/P,1577,"enum LoginCheckResult"),(srv/P,1733,"VERSION_ACCEPTED = 0"),
 (srv/P,1727,"VERSION_MAGIC = 0x8033"),(srv/P,1731,"VERSION_BUILD = 0x14F8"),(srv/P,1859,"CipherState(int seed)"),
 (srv/P,1867,"byte[] encrypt(byte[] plain)"),(srv/P,1883,"value += plain.length"),(srv/P,1891,"byte[] decrypt(byte[] cipher)"),
 (srv/P,1913,"value += cipher.length"),(srv/P,1951,"static final class Reader"),(srv/P,1983,"String S()"),
 (srv/P,2049,"static final class Writer"),(srv/P,2057,"Writer c(int value)"),(srv/P,2061,"NumericBreakthrough.enabled()"),
 (srv/P,2089,"Writer s(String value)"),(srv/P,2101,"Writer S(String value)"),(srv/P,2129,"static byte[] readFrame"),
 (srv/P,2141,"invalid LiTo frame length"),(srv/P,2153,"static void writeFrame"),(srv/P,2157,"total > 0xFFFF"),
 (srv/P,2209,"battleBootstrap(int mapId"),(srv/P,2223,"litoTimeLimit"),(srv/P,2255,"battleResult(LitoBattleResult"),
 (srv/P,2419,"litoRoomParticipant("),(srv/P,2529,"awards.legacyCertificateCodes()"),(srv/P,3317,"enum LitoVsServerAvailability"),
 (srv/P,3355,"enum LitoCharacterSelectResult"),(srv/P,3411,"SILENT_CLEANUP(1)"),(srv/P,4059,"record SelfSpellRequest"),
 (srv/P,4071,"record TeleportSpellRequest(int row, int column, int mapId"),(srv/P,4945,"static LoginRequest loginRequest"),
 (srv/P,4979,"connectorBytes.length > 31"),(srv/P,4997,"static VersionRequest versionRequest"),(srv/P,5021,"static boolean supportedVersion"),
 (srv/P,5051,"static byte[] versionAccepted"),(srv/P,5085,"static byte[] versionRejected"),(srv/P,5109,"static byte[] loginCheck"),
 (srv/P,5175,"static byte[] classicServerList"),(srv/P,5229,"requireClassicServerSelect"),(srv/P,5245,"static byte[] litoVsServerIp"),
 (srv/P,5527,"static byte[] notice(int messageId)"),(srv/P,5551,"static byte[] commonNews"),(srv/P,5589,"static void readNotice"),
 (srv/P,5599,"static void loginResult"),(srv/P,8765,"static boolean containsControl"),(srv/P,8773,"containsUnsafeBoardControl"),
 (srv/P,8891,"static MoveRequest moveRequest"),(srv/P,8933,"static SpellSelector spellSelector"),(srv/P,8965,"selectorOnlySpellRequest"),
 (srv/P,9347,"static AttackSpellRequest attackSpellRequest"),(srv/P,10711,"static byte[] characterStatus"),
 (srv/P,10857,".c(strength).c(intelligence).c(wisdom)"),(srv/P,10895,"packet.d(0).c(nativeFoodGauge"),(srv/P,11705,"litoCharacterDirectoryCount"),
 (srv/P,12023,"static byte[] worldTime"),(srv/P,12053,"static byte[] inventoryBatch"),(srv/P,12741,"appendInventoryRecord"),
 (srv/P,13169,"requireBoundedLegacyString"),(srv/P,13191,"requireNativeString"),(srv/P,13923,"litoNewsTrigger"),
 (srv/P,13933,"litoLoginOk"),(srv/P,13981,"litoCharacterSourceRequest"),(srv/P,14039,"litoEnterChannelRequest"),
 (srv/P,16285,"vsServerStatus("),(srv/P,16333,"litoServerDirectory("),(srv/P,16441,"allowPostTransition"),
 (srv/CS,157,"enum Endpoint { LOGIN, GAME }"),(srv/CS,158,"private enum Phase"),(srv/CS,1087,"public void run()"),
 (srv/CS,1092,"setTcpNoDelay(true)"),(srv/CS,1095,"Protocol.S_KEY"),(srv/CS,1106,"handle(plain);"),(srv/CS,1112,"finally {"),
 (srv/CS,1755,'"malformed-login"'),(srv/CS,7328,"phase != Phase.KEY_SENT"),(srv/CS,7351,"phase != Phase.VERSION_OK"),
 (srv/CS,7357,'"malformed-login"'),(srv/CS,7384,"suppliedAccount.length() > 64"),(srv/CS,7394,"SYSTEM_ERROR"),
 (srv/CS,7398,"ACCOUNT_BANNED"),(srv/CS,7404,"sidecar-account-banned"),(srv/CS,7470,"character-backend-error"),
 (srv/CS,7479,"account-in-use"),(srv/CS,7510,"isNativeGameLoginPreamble("),(srv/CS,7519,"beginGameBootstrap()"),
 (srv/CS,7536,"sendNextLoginNoticeOrBootstrap()"),(srv/CS,7550,"acknowledgeReadNotice"),(srv/CS,7562,"acknowledgeLoginResult"),
 (srv/CS,7686,"sendGameBootstrap()"),(srv/CS,7697,"completeGameBootstrap()"),(srv/CS,7735,"C_LITO_SELECT_CHAR"),
 (srv/CS,7769,"sendLoginBootstrap()"),(srv/CS,7805,"sendLoginFailure("),(srv/CS,7811,"phase = Phase.VERSION_OK"),
 (srv/CS,7814,"sendClassicServerList()"),(srv/CS,7848,"sendLitoServerList()"),(srv/CS,7880,"selectClassicServer("),
 (srv/CS,7919,"enterLobby(String trigger)"),(srv/CS,7928,"sendCharacterDirectory()"),(srv/CS,7979,"litoCharacterDirectoryRow("),
 (srv/CS,8004,"requestLitoCharacterInfo("),(srv/CS,8074,"S_RESTART -> C_D9"),(srv/CS,9750,"Protocol.inventoryBatch("),
 (srv/CS,9839,"allowPostTransition()"),(srv/CS,10218,"joinRoom(byte[] payload"),(srv/CS,15252,"allowPostTransition()"),
 (srv/CS,15282,"Protocol.worldTime("),(srv/CS,88455,"ipv4LittleEndian(String address)"),
 (srv/GSS,893,"TOURNAMENT_LEGACY_CERTIFICATE_SLOTS = 12"),(srv/GSS,894,"TOURNAMENT_VICTORY_BADGE_MODE_COUNT = 5"),
 (srv/GSS,5742,"validTournamentLegacyCertificate"),(srv/NB,6,"LEVEL_ABILITY_SCHEMA_VERSION = 2"),(srv/NB,20,'"LITO_NUMERIC_BREAKTHROUGH"'),
 (srv/NB,25,"static int schemaVersion()"),(srv/"ServerConfig.java",272,'"max.frame", 8192'),
 (srv/"UnifiedConfiguration.java",77,'"login.bootstrap.sequence","login_check,classic_server_list,lito_server_list"'),
 (srv/"UnifiedConfiguration.java",79,'"advertised.host",target.getProperty("game.bind")'),(srv/"Main.java",67,"loginServer.bind"),
 (lau/"LauncherService.cs",18,"NumericBreakthroughSchemaVersion = 4"),(lau/"LauncherSettings.cs",54,"NumericBreakthroughEnabled { get; set; } = true"),
 (lau/"LocalServerService.cs",230,"LITO_NUMERIC_BREAKTHROUGH"),
 (srv/P,4107,"record TrueTargetRequest"),(srv/P,9389,"static TrueTargetRequest trueTargetRequest"),(srv/P,9391,"spellSelector(payload)"),
 (srv/P,14785,"static byte[] observerBox"),(srv/CS,8886,"sendInventoryBootstrap();"),(srv/CS,9733,"sendInventoryBootstrap()"),
 (srv/CS,9734,"if (inventoryBootstrapSent) return;"),(srv/CS,9723,"inventoryBootstrapSent = false"),(srv/CS,20272,"inventoryBootstrapSent = false"),
 (srv/CS,20413,"inventoryBootstrapSent = false"),(srv/CS,20483,"inventoryBootstrapSent = false"),(srv/CS,10484,"Protocol.battleBootstrap(scene.mapId(), 1)"),
 (srv/CS,12008,"Protocol.battleBootstrap("),(srv/CS,12010,"sendEncrypted(bootstrap.world()"),(srv/CS,12017,"BattleRules.Mode.TIME_ATTACK"),
 (srv/CS,12021,"phase = Phase.BATTLE;"),(srv/CS,12034,"sendInventoryBootstrap();"),(srv/CS,44998,"Protocol.trueTargetRequest(payload)"),
]
EXPECTED_HEAD = "2066774824b314b2c65834f9afe72b68f7391f06"
ok = 0
for f, n, needle in checks:
    lines = f.read_text(encoding="utf-8").split("\n")
    hit = needle in lines[n-1]
    ok += hit
    if not hit: print(f"FAIL {f.name}:{n} expected {needle!r} got {lines[n-1].strip()[:80]!r}")
head = subprocess.run(["git","-C",str(root),"rev-parse","HEAD"],capture_output=True,text=True).stdout.strip()
selector_parsers = sum("spellSelector(payload)" in l for l in (srv/P).read_text(encoding="utf-8").split("\n"))
print(f"source HEAD {head}; {ok}/{len(checks)} citations OK; spellSelector(payload) parsers={selector_parsers}")
if head != EXPECTED_HEAD:
    print(f"FAIL source HEAD {head} != expected {EXPECTED_HEAD}"); sys.exit(2)
if ok != len(checks) or selector_parsers != 11:
    sys.exit(1)
