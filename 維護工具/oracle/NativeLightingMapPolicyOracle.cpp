// Isolated original map-change policy fragment. No game or framebuffer runs.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <algorithm>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <vector>

static_assert(sizeof(void*) == 4, "Original fragment is x86");
static int map_fixture;
static unsigned char flag_fixture;

static std::vector<unsigned char> read(const std::filesystem::path& path) {
    std::ifstream file(path, std::ios::binary);
    if (!file) throw std::runtime_error("Input unavailable");
    return {(std::istreambuf_iterator<char>(file)), {}};
}
static void relocate(unsigned char* code, size_t size, uint32_t old, uintptr_t replacement, int expected) {
    int found = 0;
    for (size_t i = 0; i + 4 <= size; ++i) {
        uint32_t value;
        memcpy(&value, code + i, 4);
        if (value == old) {
            memcpy(code + i, &replacement, 4);
            ++found;
            i += 3;
        }
    }
    if (found != expected) throw std::runtime_error("Unexpected absolute operand count");
}
__declspec(naked) static int invoke(const void*, int) {
    __asm {
        push esi
        mov eax, [esp+8]
        mov esi, [esp+12]
        call eax
        pop esi
        ret
    }
}
int wmain(int argc, wchar_t** argv) {
    try {
        if (argc != 5) throw std::runtime_error("Expected fragment table relocated-fragment output-flags");
        const auto original = read(argv[1]);
        const auto table_bytes = read(argv[2]);
        if (original.size() != 49 || table_bytes.size() != 237*4)
            throw std::runtime_error("Unexpected original extent");
        std::vector<uint32_t> table(237);
        memcpy(table.data(), table_bytes.data(), table_bytes.size());
        auto code = static_cast<unsigned char*>(VirtualAlloc(nullptr, 65536, MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE));
        if (!code) throw std::runtime_error("Private allocation failed");
        memcpy(code, original.data(), original.size());
        relocate(code, 49, 0x62f324, reinterpret_cast<uintptr_t>(&map_fixture), 1);
        relocate(code, 49, 0x75aa60, reinterpret_cast<uintptr_t>(&flag_fixture), 2);
        relocate(code, 49, 0x62ef68, reinterpret_cast<uintptr_t>(table.data()), 1);
        relocate(code, 49, 0x62f31c, reinterpret_cast<uintptr_t>(table.data()+table.size()), 1);
        std::ofstream relocated(std::filesystem::path(argv[3]), std::ios::binary);
        relocated.write(reinterpret_cast<const char*>(code), 49);
        relocated.close();
        if (!relocated) throw std::runtime_error("Relocation retention failed");
        // Original branches target byte49; append a harness-only flag return there.
        const unsigned char tail[] = {0x0f,0xb6,0x05,0,0,0,0,0xc3};
        memcpy(code+49, tail, sizeof(tail));
        const auto flag_address = reinterpret_cast<uintptr_t>(&flag_fixture);
        memcpy(code+52, &flag_address, 4);
        DWORD old;
        if (!VirtualProtect(code, 65536, PAGE_EXECUTE_READ, &old)
                || !FlushInstructionCache(GetCurrentProcess(), code, 65536))
            throw std::runtime_error("Private code protection failed");
        std::ofstream flags(std::filesystem::path(argv[4]), std::ios::binary);
        if (!flags) throw std::runtime_error("Flag output unavailable");
        for (int map = 0; map < 65536; ++map) {
            flag_fixture = 0x7e;
            const int actual = invoke(code, map);
            const bool expected = map >= 0x4000 || std::find(table.begin(), table.end(), map) != table.end();
            if (actual != expected || map_fixture != map) throw std::runtime_error("Map policy mismatch");
            flags.put(static_cast<char>(actual));
        }
        for (int map : {std::numeric_limits<int>::min(), -1, 65536, std::numeric_limits<int>::max()}) {
            flag_fixture = 0x7e;
            const int actual = invoke(code, map);
            if (actual != (map >= 0x4000) || map_fixture != map)
                throw std::runtime_error("Signed map boundary mismatch");
        }
        flags.close();
        if (!flags) throw std::runtime_error("Flag output failed");
        VirtualFree(code, 0, MEM_RELEASE);
        std::cout << "{\"status\":\"original_map_change_policy_matches_table\",\"map16_cases\":65536,\"signed_boundaries\":4,\"table_entries\":237,\"game_sessions\":0,\"framebuffer_verified\":false}\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
