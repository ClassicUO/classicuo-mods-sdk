using System.Runtime.InteropServices;

namespace CuoModSdk;

/// <summary>
/// Bump arena over an unmanaged (fixed-address) region serving the host&lt;-&gt;guest byte
/// boundary. The host writes call inputs here via <c>mod_alloc</c> and reads packed
/// returns from here; the SDK copies finished flatbuffers in via <c>ModRuntime</c>.
/// NativeMemory (not a managed byte[]) keeps the base address stable — a returned
/// pointer must survive until the host reads it, and the NativeAOT GC must never
/// relocate the region under it. Twin of cuo-mod-sdk/src/arena.rs.
/// </summary>
internal static unsafe class Arena
{
    const int Align = 8; // flatbuffer roots want 8-byte alignment (u64/i64 wire fields)

    static byte* _buf;
    static int _cap;
    static int _top;

    /// <summary>Reserve <paramref name="size"/> bytes (>=1, 8-aligned start); returns the absolute linear-memory address.</summary>
    public static int Alloc(int size)
    {
        var start = (_top + (Align - 1)) & ~(Align - 1);
        var end = start + (size < 1 ? 1 : size);
        if (end > _cap)
        {
            var newCap = _cap == 0 ? 64 * 1024 : _cap;
            while (newCap < end)
                newCap <<= 1;
            _buf = (byte*)NativeMemory.Realloc(_buf, (nuint)newCap);
            _cap = newCap;
        }
        _top = end;
        return (int)(nint)(_buf + start);
    }

    public static void Reset() => _top = 0;
}
