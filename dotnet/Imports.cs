using System.Runtime.InteropServices;
using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using Utf8 = System.Text.Encoding;

namespace CuoModSdk;

/// <summary>
/// Host imports (wasm import module <c>"cuo"</c>) + safe wrappers — twin of
/// cuo-mod-sdk/src/imports.rs.
///
/// These are the mid-run RPCs a guest can call synchronously during an export
/// (<c>mod_run</c> / <c>mod_observer</c> / <c>mod_filter</c>). Everything else rides the
/// ABI buffers. Out-parameter calls (<see cref="ResolveCliloc"/>, <see cref="EntityChildren"/>,
/// <see cref="ComponentGetJson"/>, <see cref="ResourceGetJson"/>) use a buffer + retry
/// loop: the host returns the needed length (or count); if it exceeds capacity we recall
/// with a bigger one. Buffers are managed arrays pinned for the call — not the ABI
/// arena, which is reserved for the call in flight.
/// </summary>
internal static unsafe class Imports
{
    static class Ffi
    {
        [DllImport("cuo", EntryPoint = "resolve_serial"), WasmImportLinkage]
        public static extern ulong ResolveSerial(uint serial);

        [DllImport("cuo", EntryPoint = "gump_size"), WasmImportLinkage]
        public static extern uint GumpSize(uint id);

        [DllImport("cuo", EntryPoint = "measure_text"), WasmImportLinkage]
        public static extern uint MeasureText(uint font, int ptr, int len);

        [DllImport("cuo", EntryPoint = "resolve_cliloc"), WasmImportLinkage]
        public static extern uint ResolveCliloc(uint id, int outPtr, int cap);

        [DllImport("cuo", EntryPoint = "net_send"), WasmImportLinkage]
        public static extern void NetSend(int ptr, int len);

        [DllImport("cuo", EntryPoint = "log"), WasmImportLinkage]
        public static extern void Log(int ptr, int len);

        [DllImport("cuo", EntryPoint = "entity_parent"), WasmImportLinkage]
        public static extern ulong EntityParent(ulong entity);

        [DllImport("cuo", EntryPoint = "entity_children"), WasmImportLinkage]
        public static extern uint EntityChildren(ulong entity, int outPtr, int cap);

        [DllImport("cuo", EntryPoint = "component_get"), WasmImportLinkage]
        public static extern uint ComponentGet(ulong entity, uint typeId, int outPtr, int cap);

        [DllImport("cuo", EntryPoint = "resource_get"), WasmImportLinkage]
        public static extern uint ResourceGet(uint typeId, int outPtr, int cap);
    }

    /// <summary>Append a diagnostic line to the host log.</summary>
    public static void Log(string msg)
    {
        var b = Utf8.UTF8.GetBytes(msg);
        fixed (byte* p = b)
            Ffi.Log((int)(nint)p, b.Length);
    }

    /// <summary>Resolve a UO serial to its entity id (0 = not mapped).</summary>
    public static ulong ResolveSerial(uint serial) => Ffi.ResolveSerial(serial);

    /// <summary>Gump dimensions ((0,0) when unknown).</summary>
    public static (int Width, int Height) GumpSize(uint id)
    {
        var packed = Ffi.GumpSize(id);
        return ((int)(packed >> 16), (int)(packed & 0xFFFF));
    }

    /// <summary>Rendered pixel width of <paramref name="text"/> in the given UO font.</summary>
    public static int MeasureText(uint font, string text)
    {
        var b = Utf8.UTF8.GetBytes(text);
        fixed (byte* p = b)
            return (int)Ffi.MeasureText(font, (int)(nint)p, b.Length);
    }

    /// <summary>Send raw framed packet bytes to the server (usable from the packet filter).</summary>
    public static void NetSend(ReadOnlySpan<byte> bytes)
    {
        fixed (byte* p = bytes)
            Ffi.NetSend((int)(nint)p, bytes.Length);
    }

    /// <summary>Parent entity id (0 = none / root).</summary>
    public static ulong EntityParent(ulong entity) => Ffi.EntityParent(entity);

    /// <summary>Child entity ids of <paramref name="entity"/>, in order.</summary>
    public static ulong[] EntityChildren(ulong entity)
    {
        var cap = 16;
        while (true)
        {
            var buf = new ulong[cap];
            int count;
            fixed (ulong* p = buf)
                count = (int)Ffi.EntityChildren(entity, (int)(nint)p, cap);
            if (count <= cap)
            {
                Array.Resize(ref buf, count);
                return buf;
            }
            cap = count;
        }
    }

    /// <summary>Resolve a cliloc id to its string (empty if unknown).</summary>
    public static string ResolveCliloc(uint id) =>
        ReadString((ptr, cap) => Ffi.ResolveCliloc(id, ptr, cap)) ?? "";

    /// <summary>JSON of a component on <paramref name="entity"/>, or <c>null</c> when absent.</summary>
    public static string? ComponentGetJson(ulong entity, ushort typeId) =>
        ReadString((ptr, cap) => Ffi.ComponentGet(entity, typeId, ptr, cap));

    /// <summary>JSON of a resource, or <c>null</c> when absent.</summary>
    public static string? ResourceGetJson(ushort typeId) =>
        ReadString((ptr, cap) => Ffi.ResourceGet(typeId, ptr, cap));

    /// <summary>
    /// A component on <paramref name="entity"/> deserialized into <typeparamref name="T"/>
    /// (JSON wire), or <c>default</c> when absent. Typed sugar over
    /// <see cref="ComponentGetJson"/>; JsonTypeInfo overload only (AOT-safe under ILC).
    /// </summary>
    public static T? ComponentGet<T>(ulong entity, ushort typeId, JsonTypeInfo<T> typeInfo)
    {
        var json = ComponentGetJson(entity, typeId);
        return json == null ? default : JsonSerializer.Deserialize(json, typeInfo);
    }

    /// <summary>
    /// A singleton resource deserialized into <typeparamref name="T"/> (JSON wire), or
    /// <c>default</c> when absent. Typed sugar over <see cref="ResourceGetJson"/>;
    /// JsonTypeInfo overload only (AOT-safe under ILC).
    /// </summary>
    public static T? ResourceGet<T>(ushort typeId, JsonTypeInfo<T> typeInfo)
    {
        var json = ResourceGetJson(typeId);
        return json == null ? default : JsonSerializer.Deserialize(json, typeInfo);
    }

    // Shared out-buffer retry loop for the string-returning imports. call(ptr, cap)
    // returns the needed byte length; 0 means absent (null). Recalls with a bigger
    // buffer when the needed length exceeds cap.
    static string? ReadString(Func<int, int, uint> call)
    {
        var cap = 256;
        while (true)
        {
            var buf = new byte[cap];
            int needed;
            fixed (byte* p = buf)
                needed = (int)call((int)(nint)p, cap);
            if (needed == 0)
                return null;
            if (needed <= cap)
                return Utf8.UTF8.GetString(buf, 0, needed);
            cap = needed;
        }
    }
}
