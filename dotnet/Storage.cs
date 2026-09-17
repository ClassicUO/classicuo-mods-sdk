using System.Runtime.InteropServices;
using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using Utf8 = System.Text.Encoding;

namespace CuoModSdk;

/// <summary>
/// Per-mod persistent storage over the host <c>"cuo"</c> imports <c>storage_get</c> /
/// <c>storage_set</c> — twin of cuo-mod-sdk/src/storage.rs.
///
/// One opaque UTF-8 blob per mod, persisted by the host at
/// <c>&lt;client&gt;/Data/Mods/&lt;modname&gt;/storage.json</c>. Writes are
/// write-through (no debounce) — call <see cref="Set(string)"/> on a user action,
/// not every frame.
///
/// The blob is only reachable once the host has registered the mod, which happens
/// AFTER <c>mod_setup</c> returns: read settings from a mod-startup (or Update)
/// system, not from the setup callback — <see cref="Get()"/> returns <c>""</c> if
/// called too early.
///
/// The typed overloads take a <see cref="JsonTypeInfo{T}"/> so the mod's own
/// source-generated context does the work: reflection-based STJ can't survive the
/// wasi-wasm ILC publish.
/// </summary>
internal static unsafe class Storage
{
    static class Ffi
    {
        [DllImport("cuo", EntryPoint = "storage_get"), WasmImportLinkage]
        public static extern uint StorageGet(uint arg, int outPtr, int cap);

        [DllImport("cuo", EntryPoint = "storage_set"), WasmImportLinkage]
        public static extern void StorageSet(int ptr, int len);
    }

    /// <summary>The stored blob, or <c>""</c> when the mod has never written one
    /// (or storage isn't available yet — see the class docs).</summary>
    public static string Get()
    {
        // Same needed-length retry protocol as Imports.ResolveCliloc: the host
        // returns the UTF-8 byte length and only fills the buffer when it fits.
        var cap = 512;
        while (true)
        {
            var buf = new byte[cap];
            int needed;
            fixed (byte* p = buf)
                needed = (int)Ffi.StorageGet(0, (int)(nint)p, cap);
            if (needed == 0)
                return "";
            if (needed <= cap)
                return Utf8.UTF8.GetString(buf, 0, needed);
            cap = needed;
        }
    }

    /// <summary>Replace the stored blob. Persisted immediately.</summary>
    public static void Set(string json)
    {
        var b = Utf8.UTF8.GetBytes(json);
        fixed (byte* p = b)
            Ffi.StorageSet((int)(nint)p, b.Length);
    }

    /// <summary><see cref="Get()"/> parsed as <typeparamref name="T"/>.
    /// <c>default</c> when nothing is stored or the blob doesn't deserialize (a
    /// schema change in the mod, a hand-edited file).</summary>
    public static T? Get<T>(JsonTypeInfo<T> typeInfo)
    {
        var raw = Get();
        if (raw.Length == 0)
            return default;
        try
        {
            return JsonSerializer.Deserialize(raw, typeInfo);
        }
        catch (JsonException)
        {
            return default;
        }
    }

    /// <summary>Serialize <paramref name="value"/> as JSON and <see cref="Set(string)"/> it.</summary>
    public static void Set<T>(T value, JsonTypeInfo<T> typeInfo)
        => Set(JsonSerializer.Serialize(value, typeInfo));
}
