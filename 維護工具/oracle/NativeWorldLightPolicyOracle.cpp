// Execute only the retained 488AE0..488BA7 map-to-mask-argument fragment.
// The original world renderer and its callees are not executed.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <vector>

static_assert(sizeof(void*) == 4, "Private original fragment requires x86");
static unsigned char map_flag;
static std::vector<unsigned char> read(const std::filesystem::path& path) {
    std::ifstream file(path, std::ios::binary);
    if (!file) throw std::runtime_error("Input unavailable");
    return {(std::istreambuf_iterator<char>(file)), {}};
}
__declspec(naked) static int invoke(const void*, int) {
    __asm {
        push ebp
        mov ebp, esp
        sub esp, 8
        mov edx, [ebp+12]
        mov eax, [ebp+8]
        call eax
        mov esp, ebp
        pop ebp
        ret
    }
}
static int expected(int map, bool flag) {
    if (!flag) return map >= 547 && map <= 550 ? 8 : 15;
    if (map >= 72 && map <= 74) return 4;
    if (map >= 75 && map <= 77) return 11;
    for (int special : {307, 308, 309, 310, 320, 330, 511, 527})
        if (map == special) return 2;
    return 1;
}
int wmain(int argc, wchar_t** argv) {
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX);
    try {
        if (argc != 2) throw std::runtime_error("Expected owned evidence directory");
        const std::filesystem::path out(argv[1]);
        const auto original = read(out / "map-argument-original.bin");
        if (original.size() != 199 || original[0] != 0x8a || original[1] != 0x0d)
            throw std::runtime_error("Unexpected original instruction boundaries");
        uint32_t operand;
        memcpy(&operand, original.data() + 2, 4);
        if (operand != 0x75aa60) throw std::runtime_error("Unexpected flag operand");
        auto code = static_cast<unsigned char*>(VirtualAlloc(nullptr, 4096,
            MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
        if (!code) throw std::runtime_error("Private allocation failed");
        memcpy(code, original.data(), original.size());
        const auto flag_address = reinterpret_cast<uintptr_t>(&map_flag);
        memcpy(code + 2, &flag_address, 4);
        std::ofstream relocated(out / "map-argument-relocated.bin", std::ios::binary);
        relocated.write(reinterpret_cast<const char*>(code), original.size());
        relocated.close();
        if (!relocated) throw std::runtime_error("Relocation retention failed");
        // Original next instruction calls485A40 with ECX. Observe that argument
        // at the original call boundary; do not substitute any internal branch.
        const unsigned char tail[] = {0x8b, 0xc1, 0xc3}; // mov eax,ecx; ret
        memcpy(code + original.size(), tail, sizeof(tail));
        DWORD old;
        if (!VirtualProtect(code, 4096, PAGE_EXECUTE_READ, &old)
                || !FlushInstructionCache(GetCurrentProcess(), code, 4096))
            throw std::runtime_error("Private executable protection failed");
        std::ofstream values(out / "native-map-levels.bin", std::ios::binary);
        if (!values) throw std::runtime_error("Output unavailable");
        for (unsigned char flag : {0, 1, 128, 255}) {
            map_flag = flag;
            for (int map = 0; map < 65536; ++map) {
                const int value = invoke(code, map);
                if (value != expected(map, flag != 0))
                    throw std::runtime_error("Map argument mismatch");
                values.put(static_cast<char>(value));
            }
            for (int map : {std::numeric_limits<int>::min(), -1, 65536,
                            std::numeric_limits<int>::max()}) {
                if (invoke(code, map) != expected(map, flag != 0))
                    throw std::runtime_error("Signed boundary mismatch");
            }
        }
        values.close();
        if (!values) throw std::runtime_error("Output retention failed");
        VirtualFree(code, 0, MEM_RELEASE);
        std::cout << "{\"status\":\"original_world_map_mask_argument_fragment_verified\","
            "\"map16_cases\":262144,\"signed_boundaries\":16,\"original_bytes\":199,"
            "\"actual_game_sessions\":0,\"world_renderer_executed\":false}\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
