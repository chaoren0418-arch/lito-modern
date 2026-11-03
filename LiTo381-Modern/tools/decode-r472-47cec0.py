"""Static IDA dd-byte decoding only; no original code or callbacks execute."""
import hashlib, json, pathlib, re, sys
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

project = pathlib.Path(__file__).resolve().parent.parent
source = pathlib.Path(sys.argv[1])
output = pathlib.Path(sys.argv[2])
text = source.read_text(encoding="utf-8")
match = re.search(r"(?ms)^dword_47CEC0 dd .*?(?=^sub_47CF00 proc near)", text)
assert match is not None
raw = bytearray()
for line in match.group().splitlines():
    if line.startswith("dword_47CEC0 dd "):
        line = line.split(" dd ", 1)[1]
    elif line.startswith("dd "):
        line = line[3:]
    else:
        continue
    for token in line.split(","):
        token = token.strip()
        repeat = re.fullmatch(r"(\d+) dup\(([0-9A-Fa-f]+)h\)", token)
        if repeat:
            count, value = int(repeat[1]), int(repeat[2], 16)
        else:
            assert re.fullmatch(r"[0-9A-Fa-f]+h", token), token
            count, value = 1, int(token[:-1], 16)
        assert 1 <= count <= 3 and 0 <= value <= 0xFFFFFFFF
        raw.extend(value.to_bytes(4, "little") * count)
assert len(raw) == 64
instructions = list(Cs(CS_ARCH_X86, CS_MODE_32).disasm(bytes(raw), 0x47CEC0))
assert sum(item.size for item in instructions) == 64
rows = [{"address": f"{v.address:08X}", "bytes": v.bytes.hex().upper(),
         "mnemonic": v.mnemonic, "operands": v.op_str} for v in instructions]
assert next(v.address for v in instructions if v.mnemonic == "ret") == 0x47CEF1
calls = [v.op_str for v in instructions if v.mnemonic == "call"]
assert calls == ["0x773b0440", "0x4d65f0"], calls
result = {"state": "STATIC_DD64_X86_DECODE_NOT_EXECUTION", "source": str(source),
          "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest().upper(),
          "raw_bytes": len(raw), "raw_hex": raw.hex().upper(),
          "raw_sha256": hashlib.sha256(raw).hexdigest().upper(),
          "decoded": rows, "call_targets": calls, "first_ret": "0047CEF1",
          "source_semantics": "Reads647E18 pointer, writes647EA0 byte1, calls773B0440 with[647E18+4], multipliesreturned EAX by1000, schedules47CDF0 owner0 via4D65F0 thenRET; paddingNOPs.",
          "runtime_pointer_contents_observed": False, "original_BIN_executed": False,
          "callback_executed": False, "provider_executed": False, "GUI": 0}
output.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps({"bytes": len(raw), "instructions": len(rows), "calls": calls}))
