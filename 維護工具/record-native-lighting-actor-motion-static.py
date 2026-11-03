"""Read-only source tracing; does not execute the original client or Java code."""
from pathlib import Path
import hashlib
import json
import re
import struct
import zipfile

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / '驗證紀錄/current/native-lighting-actor-motion-static'
ASM = Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm')
JAR = ROOT / 'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'


def sha(data):
    return hashlib.sha256(data).hexdigest().upper()


def class_methods(data):
    assert data[:4] == b'\xca\xfe\xba\xbe'
    at = 8

    def take(fmt):
        nonlocal at
        result = struct.unpack_from('>' + fmt, data, at)
        at += struct.calcsize('>' + fmt)
        return result

    cp = [None] * take('H')[0]
    i = 1
    while i < len(cp):
        tag = take('B')[0]
        if tag == 1:
            size = take('H')[0]
            cp[i] = data[at:at + size].decode('utf-8')
            at += size
        elif tag in (3, 4):
            cp[i] = (tag, take('I')[0])
        elif tag in (5, 6):
            at += 8
            i += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[i] = (tag, take('H')[0])
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[i] = (tag, *take('HH'))
        elif tag == 15:
            cp[i] = (tag, *take('BH'))
        else:
            raise ValueError(tag)
        i += 1
    take('HHH')
    interfaces = take('H')[0]
    at += 2 * interfaces
    methods = {}
    for group in ('fields', 'methods'):
        for _ in range(take('H')[0]):
            _, name, descriptor, attrs = take('HHHH')
            for _ in range(attrs):
                attr_name, size = take('HI')
                if group == 'methods' and cp[attr_name] == 'Code':
                    stack, locals_, length = struct.unpack_from('>HHI', data, at)
                    methods[cp[name]] = {
                        'descriptor': cp[descriptor], 'max_stack': stack,
                        'max_locals': locals_,
                        'code_hex': data[at + 8:at + 8 + length].hex(' '),
                    }
                at += size
    refs = {}
    for i, item in enumerate(cp):
        if isinstance(item, tuple) and item[0] in (9, 10, 11):
            nt = cp[item[2]]
            refs[str(i)] = [cp[cp[item[1]][1]], cp[nt[1]], cp[nt[2]]]
    return methods, refs


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    assert not (OUT / 'static-findings.json').exists(), 'immutable evidence already exists'
    raw = ASM.read_bytes()
    assert sha(raw) == '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
    lines = raw.decode('utf-8').split('\n')

    def span(lo, hi):
        return '\n'.join(lines[lo - 1:hi])

    # Trace the straight-line cdecl pushes. ECX/EDX additionally carry id/gfx.
    def pushed(lo, hi, initial=None):
        registers = dict(initial or {})
        result = []
        for source in lines[lo - 1:hi]:
            s = source.strip().split(';', 1)[0].rstrip()
            m = re.fullmatch(r'(lea|mov)\s+(e[abcd]x), (.+)', s)
            if m:
                op, reg, value = m.groups()
                registers[reg] = ('&' if op == 'lea' else '') + value
            m = re.fullmatch(r'push\s+(.+)', s)
            if m:
                value = m[1]
                result.append(registers.get(value, value))
        return result

    parser = pushed(342184, 342241)
    assert parser[0] == 'esi'  # saved register, outside the variadic arguments
    assert parser[-2] == 'offset aHhdhccccdhsscd'
    assert '"hhdhccccdhsscdssccccshc"' in span(342238, 342238)
    args = list(reversed(parser[1:-2]))
    fmt = 'hhdhccccdhsscdssccccshc'
    fields = ['x', 'y', 'id', 'graphics', 'walk_action', 'heading', 'light',
              'speed', 'experience', 'lawful', 'name', 'title', 'status',
              'emblem', 'clan', 'master', 'unknown', 'hp_ratio', 'companion',
              'level', 'auxiliary_name', 'tail_word', 'tail_byte']
    destinations = {}
    for kind, field in zip(fmt, fields, strict=True):
        pointer = args.pop(0)
        destinations[pointer.removeprefix('&')] = field
        if kind == 's':
            args.pop(0)  # explicit bounded string capacity
    assert not args
    call = list(reversed(pushed(342646, 342686, {'edi': 'native_U', 'ebx': 'wire_y'})))
    assert len(call) == 19
    mapping = {f'arg_{i * 4:X}': destinations.get(v, v) for i, v in enumerate(call)}
    assert mapping['arg_28'] == 'status'
    assert mapping['arg_38'] == 'unknown'
    assert mapping['arg_3C'] == 'hp_ratio'
    assert mapping['arg_40'] == 'level'
    assert 'mov     cl, [ebp+arg_38]' in span(485099, 485108)
    assert 'mov     [esi+23h], cl' in span(485099, 485108)
    assert 'case 32' in span(372706, 372710)
    assert 'call    sub_4A5A90' in span(372706, 372710)
    assert 'mov     byte ptr [esi+50h], 1' in span(346587, 346602)
    insertion = span(484343, 484440)
    assert 'cmp     [ebx+8], edi\njl      short loc_4E7813' in insertion
    assert 'mov     [edx+ebx*4], esi' in insertion
    assert 'call    sub_4E77E0' in span(484827, 484879)
    assert 'mov     [esi+2Ch], edx' in span(473047, 473062)
    assert 'mov     [esi+30h], eax' in span(473047, 473062)
    jar_raw = JAR.read_bytes()
    assert sha(jar_raw) == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
    with zipfile.ZipFile(JAR) as z:
        movement = z.read('local/lito381/server/NativeMovement.class')
    methods, refs = class_methods(movement)
    code = methods['destination']['code_hex']
    assert methods['destination']['descriptor'] == '(JIII)[B'
    # iload_2; getstatic DX; iload 4; iaload; isub; istore 5, and Y equivalent.
    assert '1c b2 00 0e 15 04 2e 64 36 05 1d b2 00 14 15 04 2e 64 36 06' in code
    assert refs['14'][1:] == ['DX', '[I'] and refs['20'][1:] == ['DY', '[I']
    # opcode18, object id, source x/y (locals5/6), heading, bytes.
    assert '10 12 b6 00 1f 1e b6 00 23 15 05 b6 00 27 15 06 b6 00 27 15 04 b6 00 1f b6 00 2a b0' in code
    (OUT / 'NativeMovement.class').write_bytes(movement)
    spans = [(342184, 342242), (342646, 342687), (484670, 484718),
             (484827, 484955), (485099, 485274), (484343, 484440),
             (372706, 372710), (346413, 346622), (340664, 340750),
             (473047, 473062)]
    (OUT / 'selected-assembly.json').write_text(json.dumps([
        {'first_newline_line': lo, 'last_newline_line': hi, 'assembly': span(lo, hi)}
        for lo, hi in spans], ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    (OUT / 'class-bytecode.json').write_text(json.dumps({
        'class_sha256': sha(movement), 'methods': methods, 'constant_pool_references': refs,
    }, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    findings = {
        'status': 'static_source_traces_retained_no_native_or_JVM_execution',
        'assembly_sha256': sha(raw), 'formal_jar_sha256': sha(jar_raw),
        'packet_parser_destinations': destinations, 'constructor_stack_arguments': mapping,
        'actor_23': 'S0B unknown byte low nibble, not companion byte',
        'actor_1F': 'status bit4 OR clan relation branch; global 834F14/list meanings unproven',
        'actor_50': 'S_ACTION opcode0x20 action8 sets1; do not substitute generic dead without lifecycle proof',
        'registration_order': 'ascending SIGNED actor id via cmp+jl insertion, not arrival order',
        'registration_scope': 'new actor path calls4E77E0; existing id updates same slot',
        'Rust_BTreeMap_u32_order_equivalence': 'only within a common sign partition; high-bit IDs differ',
        'C02_S12_wire_coordinates': 'destination arguments minus DX/DY[heading] = source cell',
        'native_S12_position_base': 'source U=2*wire_x-0x8000,V=wire_y stored +2C/+30; next cell separate',
        'lighting_movement_requirement': 'do not add original whole-step offsets to an already interpolated pose',
        'camera_initialization_verified': False, 'walking_frame_phase_verified': False,
        'actor_flag_full_lifecycle_verified': False, 'native_packet_handler_executed': False,
        'JVM_executed': False, 'actual_game_sessions': 0,
        'product_source_or_current_QA_changed': False, 'full_world_framebuffer_verified': False,
        'whole_goal_complete': False,
    }
    (OUT / 'static-findings.json').write_text(json.dumps(findings, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    files = [OUT / n for n in ('NativeMovement.class', 'selected-assembly.json', 'class-bytecode.json', 'static-findings.json')]
    files.append(Path(__file__))
    pins = {p.name: sha(p.read_bytes()) for p in files}
    archive = OUT / 'static-source-evidence.zip'
    with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED) as z:
        for p in files:
            z.write(p, p.name)
    with zipfile.ZipFile(archive) as z:
        assert {n: sha(z.read(n)) for n in z.namelist()} == pins
    receipt = {'status': 'static_archive_readback_verified', 'files': pins,
               'archive_sha256': sha(archive.read_bytes()), 'actual_game_sessions': 0,
               'native_or_JVM_execution': False, 'whole_goal_complete': False}
    (OUT / 'archive-readback.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(receipt, ensure_ascii=False))


if __name__ == '__main__':
    main()
