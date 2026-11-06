//! Static evidence gates. Never load/execute the original executable.
use sha2::{Digest, Sha256};

#[test]
#[ignore = "LITO_NATIVE_CLOCK_PE and LITO_NATIVE_CALLBACK_ASM; pinned read-only source"]
fn original_callback_cfg_suffixes_nested_drain_and_defaults_match() {
    let pe =
        std::fs::read(std::env::var_os("LITO_NATIVE_CLOCK_PE").expect("original PE path")).unwrap();
    assert_eq!(
        format!("{:X}", Sha256::digest(&pe)),
        "4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9"
    );
    // The pinned rebuilt image has RVA == file offset. Full 4E2150 includes
    // its epilogue/ret, not just the two suffixes or IDA's first basic block.
    for (start, end, expected) in [
        (
            0x4d6570,
            0x4d65f0,
            "7FEE4E3AC0CAD12FC03E8CD6CA411D9850A9E578E469275FA5524D4445E3FA4F",
        ),
        (
            0x4857b0,
            0x485800,
            "DF80F49E95BE1642D04283028ADBC0ECCC31D1AB7D8EC1EFE8B114A771A8D042",
        ),
        (
            0x4e2150,
            0x4e34b8,
            "5B772D97DF44BC8B6A7FEC29C84349E3609AD10E45BE62669160737BB16E0AC3",
        ),
        (
            0x43de10,
            0x43ed00,
            "E7D31FB8AAF133144FEAB6D776E3BC4DC0BDA00324A6485E3DDB603D401DBAAB",
        ),
        (
            0x4dfdd0,
            0x4dfec0,
            "80B17A94B8EA1139DB2264A5DC8C390FC51FA62E07D4553431FD968931184297",
        ),
        (
            0x4e62a1,
            0x4e63c0,
            "E02F06166239D8C9CF09C58F6E1DF19B445C9B5B9C265A51B489CA0CF9D5B66E",
        ),
    ] {
        assert_eq!(
            format!(
                "{:X}",
                Sha256::digest(&pe[start - 0x400000..end - 0x400000])
            ),
            expected
        );
    }
    // Critical upstream branches: frame completion, queued action, held input.
    for (site, expected) in [
        (0x4e2e9e, &[0x0f, 0x85, 0xf2, 0x01, 0, 0][..]), // nonzero -> 4E3096, skips EB3=1
        (0x4e2fda, &[0x0f, 0x8e, 0x88, 0, 0, 0][..]),    // signed <=0 -> 4E3068
        (0x4e306f, &[0x75, 0x0d][..]),                   // byte760098 !=0 -> 4E307E
        (0x4e3078, &[0x0f, 0x84, 0x94, 0, 0, 0][..]),    // byte64BBC3 ==0 -> 4E3112
        (0x4e3088, &[0xc6, 0x05, 0xb3, 0x4e, 0x83, 0, 1][..]), // sole EB3=1 store
    ] {
        assert_eq!(
            &pe[site - 0x400000..site - 0x400000 + expected.len()],
            expected
        );
    }
    for (site, target) in [
        (0x4e2e97, 0x4e5b50),
        (0x4e308f, 0x4d65f0),
        (0x4e3118, 0x4e0200),
        (0x4e33ad, 0x488860),
        (0x4e3432, 0x488860),
    ] {
        let at = site - 0x400000;
        assert_eq!(pe[at], 0xe8);
        let relative = i32::from_le_bytes(pe[at + 1..at + 5].try_into().unwrap());
        assert_eq!((site as i64 + 5 + i64::from(relative)) as usize, target);
    }
    let asm =
        std::fs::read(std::env::var_os("LITO_NATIVE_CALLBACK_ASM").expect("original ASM path"))
            .unwrap();
    assert_eq!(
        format!("{:X}", Sha256::digest(&asm)),
        "2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A"
    );
    let asm = String::from_utf8(asm).unwrap();
    let function = |name: &str| {
        let start = asm.find(&format!("sub_{name} proc near")).unwrap();
        let end = start + asm[start..].find(&format!("sub_{name} endp")).unwrap();
        &asm[start..end]
    };
    let actor = function("4E2150");
    assert_eq!(actor.matches("call    sub_488860").count(), 2);
    assert_eq!(actor.matches("mov     byte_834EB3, 1").count(), 1);
    assert!(actor.contains("mov     byte_834EB3, 0"));
    let pump = function("4857B0");
    let drain = pump.find("call    sub_4D6570").unwrap();
    let draw = pump.find("call    sub_488860").unwrap();
    assert!(drain < draw);
    assert!(pump[draw..].contains("call    sub_4D69F0"));
    assert!(pump.contains("cmp     edi, 1F4h"));
    assert!(pump.contains("jl      short loc_4857D7"));
    let packet = function("4ADDA0");
    assert!(packet.contains("mov     edi, 0A9h"));
    assert!(packet.contains("cmp     word ptr [ebp+var_10], di"));
    assert!(packet.contains("cmp     esi, dword_834EE4"));
    assert_eq!(packet.matches("call    sub_4857B0").count(), 1);

    // Guard provenance matters as much as the two suffix call sites. These
    // are ordered instructions in the SHA-pinned ASM, not live observations
    // or permission to substitute modern animation/input state for globals.
    let ordered = |body: &str, instructions: &[&str]| {
        let mut rest = body;
        for instruction in instructions {
            let at = rest
                .find(instruction)
                .unwrap_or_else(|| panic!("missing {instruction}"));
            rest = &rest[at + instruction.len()..];
        }
    };
    ordered(
        packet,
        &[
            "call    dword ptr [edx+4]",
            "mov     al, byte_63DF4C",
            "test    al, al",
            "jz      short loc_4ADF45",
            "cmp     word ptr [ebp+var_10], di",
            "jnz     short loc_4ADF45",
            "cmp     esi, dword_834EE4",
            "jnz     short loc_4ADF45",
            "call    sub_4857B0",
        ],
    );
    ordered(
        pump,
        &[
            "xor     edx, edx",
            "mov     ecx, offset sub_482F50",
            "call    sub_4D6870",
            "mov     dword_62F32C, 0FFFFFFFFh",
            "mov     dword_75A988, edi",
            "call    sub_4D69F0",
            "call    sub_4D6570",
            "call    sub_488860",
            "call    sub_4D69F0",
            "sub     ecx, esi",
            "add     edi, ecx",
            "cmp     edi, 1F4h",
            "jl      short loc_4857D7",
            "mov     byte_75AB1C, 1",
        ],
    );
    let suffix = &actor[actor.find("\nloc_4E3207:").unwrap()..];
    ordered(
        suffix,
        &[
            "mov     al, [ecx+13h]",
            "jnz     loc_4E3429",
            "mov     al, byte_834F1D",
            "jz      short loc_4E325C",
            "call    sub_48BC80",
            "mov     ecx, offset sub_4E2150",
            "call    sub_4D6870",
            "mov     byte_834EB3, 0",
            "loc_4E325C:",
            "mov     al, byte_834EB2",
            "jz      loc_4E339B",
            "loc_4E3354:",
            "mov     byte_63DF4C, 1",
            "loc_4E339B:",
            "mov     al, byte_834EB3",
            "jz      short loc_4E3420",
            "mov     al, byte_641498",
            "jnz     short loc_4E33B2",
            "call    sub_488860",
            "loc_4E3429:",
            "mov     al, byte_641498",
            "jnz     short loc_4E3437",
            "call    sub_488860",
        ],
    );
    // 641498 is produced by the normal render tail: base action count 12
    // AND an unsigned raw render-clock delta greater than 40 ms. It is not
    // a minimized-window flag, a pending worker, or the current walk pose.
    let render = function("488860");
    let tail = &render[render.find("\nloc_48AED4:").unwrap()..];
    ordered(
        tail,
        &[
            "call    sub_4D69F0",
            "mov     esi, dword_75A834",
            "sub     edx, esi",
            "mov     dword_75AC18, edx",
            "mov     dword_75A834, eax",
            "call    sub_4E8E70",
            "jz      short loc_48AF1B",
            "cmp     eax, 28h",
            "mov     byte_641498, 1",
            "ja      short loc_48AF22",
            "loc_48AF1B:",
            "mov     byte_641498, 0",
            "loc_48AF22:",
            "cmp     eax, 1F4h",
            "jbe     short loc_48AF33",
            "mov     dword_75AC18, 1F4h",
        ],
    );
    ordered(
        function("4E8E70"),
        &[
            "movsx   edx, word ptr [ecx+14h]",
            "movsx   ecx, byte ptr [ecx+18h]",
            "mov     bl, [ecx+edx*4+1]",
            "cmp     bl, 0Ch",
            "setz    al",
        ],
    );
    // Player duration has an additional source branch before ordinary speed
    // modifiers. native_speed::factor alone cannot qualify 4E2150 deadlines.
    ordered(
        function("4E8750"),
        &[
            "cmp     ebx, dword_75AA48",
            "mov     al, byte_64BC13",
            "jz      short loc_4E88C9",
            "mov     al, byte_64AFBF",
            "jnz     short loc_4E88C9",
            "lea     eax, ds:0[edx*8]",
            "sub     eax, edx",
            "sar     eax, 4",
            "jmp     short loc_4E8935",
        ],
    );
}
