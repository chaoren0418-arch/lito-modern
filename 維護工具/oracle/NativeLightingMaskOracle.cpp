// Execute the original, self-contained x86 mask compositor and clear routine.
// All input/output buffers and executable allocations belong to this helper.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <vector>

static_assert(sizeof(void*) == 4, "Original fragments require the isolated x86 helper");
using Blend = void(__cdecl*)(unsigned char*, const unsigned char*, int, int, int, int);
using Clear = void(__cdecl*)(unsigned char*);
static constexpr size_t N = 320 * 204, GUARD = 64;

static std::vector<unsigned char> bytes(const std::filesystem::path& path) {
    std::ifstream in(path, std::ios::binary);
    if(!in) throw std::runtime_error("Cannot open bounded input");
    return std::vector<unsigned char>((std::istreambuf_iterator<char>(in)), {});
}
static void* load(const std::filesystem::path& path, size_t count) {
    auto data = bytes(path);
    if(data.size() != count) throw std::runtime_error("Original fragment extent changed");
    auto code = VirtualAlloc(nullptr, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if(!code) throw std::runtime_error("Code allocation failed");
    memcpy(code, data.data(), count);
    DWORD previous;
    if(!VirtualProtect(code, 4096, PAGE_EXECUTE_READ, &previous) ||
       !FlushInstructionCache(GetCurrentProcess(), code, count))
        throw std::runtime_error("Code protection failed");
    return code;
}
template<class T> static T get(std::istream& in) {
    T value{}; in.read(reinterpret_cast<char*>(&value), sizeof(value));
    if(!in) throw std::runtime_error("Truncated command stream");
    return value;
}
int wmain(int argc, wchar_t** argv) {
    try {
        if(argc != 5) throw std::runtime_error("Expected blend.bin clear.bin input.bin output.bin");
        auto blend = reinterpret_cast<Blend>(load(argv[1], 184));
        auto clear = reinterpret_cast<Clear>(load(argv[2], 19));
        std::ifstream input(std::filesystem::path(argv[3]), std::ios::binary);
        std::ofstream output(std::filesystem::path(argv[4]), std::ios::binary);
        if(!input || !output) throw std::runtime_error("Cannot open command/output stream");
        const auto count = get<uint32_t>(input);
        if(count > 4096) throw std::runtime_error("Case budget");
        uint64_t calls = 0, cleared = 0, guards = 0, source_bytes = 0;
        for(uint32_t c = 0; c < count; ++c) {
            const auto seed = get<uint32_t>(input), reset = get<uint32_t>(input), ops = get<uint32_t>(input);
            if(ops > 64 || reset > 1) throw std::runtime_error("Operation budget");
            std::vector<unsigned char> dst(N + 2 * GUARD, 0xa7);
            for(size_t i = 0; i < N; ++i) dst[GUARD + i] = static_cast<unsigned char>(i * 17 + seed);
            if(reset) { clear(dst.data() + GUARD); ++cleared; }
            for(uint32_t op = 0; op < ops; ++op) {
                const auto x = get<int32_t>(input), y = get<int32_t>(input);
                const auto w = get<int32_t>(input), h = get<int32_t>(input);
                const auto size = get<uint32_t>(input);
                if(size > 320 * 204 || size != static_cast<uint32_t>(std::max(0, w) * std::max(0, h)))
                    throw std::runtime_error("Source budget");
                std::vector<unsigned char> src(size + 2 * GUARD, 0xb9);
                input.read(reinterpret_cast<char*>(src.data() + GUARD), size);
                if(!input) throw std::runtime_error("Truncated source");
                const auto saved = src;
                blend(dst.data() + GUARD, src.data() + GUARD, x, y, w, h); ++calls;
                if(src != saved) throw std::runtime_error("Original compositor changed source/guards");
                source_bytes += src.size();
            }
            for(size_t i = 0; i < GUARD; ++i) {
                if(dst[i] != 0xa7 || dst[GUARD + N + i] != 0xa7)
                    throw std::runtime_error("Destination guard changed");
                guards += 2;
            }
            output.write(reinterpret_cast<const char*>(dst.data() + GUARD), N);
            clear(dst.data() + GUARD); ++cleared;
            for(size_t i = 0; i < N; ++i)
                if(dst[GUARD + i] != 0) throw std::runtime_error("Clear extent/value differs");
            for(size_t i = 0; i < GUARD; ++i)
                if(dst[i] != 0xa7 || dst[GUARD + N + i] != 0xa7)
                    throw std::runtime_error("Clear changed a guard");
        }
        if(input.peek() != EOF || !output) throw std::runtime_error("Trailing input/output failure");
        output.close();
        std::cout << "{\"cases\":" << count << ",\"compositor_calls\":" << calls
            << ",\"clear_calls\":" << cleared << ",\"output_bytes\":" << count * N
            << ",\"destination_guard_bytes_checked\":" << guards
            << ",\"source_and_guard_bytes_unchanged\":" << source_bytes << "}\n";
        VirtualFree(reinterpret_cast<void*>(blend), 0, MEM_RELEASE);
        VirtualFree(reinterpret_cast<void*>(clear), 0, MEM_RELEASE);
        return 0;
    } catch(const std::exception& e) { std::cerr << e.what() << '\n'; return 1; }
}
