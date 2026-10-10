using System.Runtime.InteropServices;
using Interop = ModWorld.wit.Imports.tinyecs.modding.v0_1_0.EcsImportsInterop;

namespace CuoModSdk;

/// <summary>
/// The per-run half of <c>tinyecs:modding/ecs</c> (commands / query / res / events),
/// lowered by hand over the generated raw imports (Wit/*EcsImportsInterop.cs). It carries
/// no values (those go through cuo:modding/components, Typed.cs). The generated wrapper
/// classes allocate per call (UTF-16 strings, GCHandles, cleanup lists); these pass UTF-8
/// out of cached buffers and hand results back over the host-allocated memory, freed when
/// the run ends. Setup-time calls (system / app) use the generated classes: they run once.
/// </summary>
internal static unsafe class EcsAbi
{
    const string Ecs = "tinyecs:modding/ecs@0.1.0";

    // ── resource drops (every param handle is owned by the guest for one call) ──
    [DllImport(Ecs, EntryPoint = "[resource-drop]commands"), WasmImportLinkage]
    static extern void DropCommands(int handle);

    [DllImport(Ecs, EntryPoint = "[resource-drop]query"), WasmImportLinkage]
    static extern void DropQuery(int handle);

    [DllImport(Ecs, EntryPoint = "[resource-drop]res"), WasmImportLinkage]
    static extern void DropRes(int handle);

    [DllImport(Ecs, EntryPoint = "[resource-drop]events"), WasmImportLinkage]
    static extern void DropEvents(int handle);

    internal const byte ParamCommands = 0, ParamQuery = 1, ParamRes = 2, ParamEvents = 3;

    internal static void Drop(byte kind, int handle)
    {
        switch (kind)
        {
            case ParamCommands: DropCommands(handle); break;
            case ParamQuery: DropQuery(handle); break;
            case ParamRes: DropRes(handle); break;
            case ParamEvents: DropEvents(handle); break;
        }
    }

    // ── type paths: UTF-8 once, in memory that never moves ──
    static readonly Dictionary<string, (nint Ptr, int Len)> _paths = new(StringComparer.Ordinal);

    internal static (nint Ptr, int Len) Path(string path)
    {
        if (_paths.TryGetValue(path, out var p))
            return p;
        var n = System.Text.Encoding.UTF8.GetByteCount(path);
        var ptr = (byte*)NativeMemory.Alloc((nuint)Math.Max(1, n));
        System.Text.Encoding.UTF8.GetBytes(path, new Span<byte>(ptr, n));
        return _paths[path] = ((nint)ptr, n);
    }

    [StructLayout(LayoutKind.Sequential)]
    struct Str
    {
        public int Ptr, Len;
    }

    internal static void Remove(int commands, ulong entity, string path)
    {
        var (pp, pl) = Path(path);
        var item = new Str { Ptr = (int)pp, Len = pl };
        Interop.Commands.RemoveWasmInterop.wasmImportRemove(commands, (long)entity, (nint)(&item), 1);
    }

    internal static void Despawn(int commands, ulong entity) =>
        Interop.Commands.DespawnWasmInterop.wasmImportDespawn(commands, (long)entity);

    internal static bool ResUnchanged(int res) => Interop.Res.UnchangedWasmInterop.wasmImportUnchanged(res) != 0;

    // ── results: host-allocated (cabi_realloc = malloc) and owned by the guest ──

    /// <summary>The entity of element <paramref name="i"/> of a <c>query.entities</c> list.</summary>
    internal static ulong EntityAt(nint list, int i) => *(ulong*)(list + i * 8);

    /// <summary>query.entities(): the u64 array and its count. Free with <see cref="FreeEntities"/>.</summary>
    internal static (nint List, int Count) Entities(int query)
    {
        var ret = stackalloc int[2];
        Interop.Query.EntitiesWasmInterop.wasmImportEntities(query, (nint)ret);
        return (ret[0], ret[1]);
    }

    internal static void FreeEntities(nint list, int count)
    {
        if (count > 0)
            NativeMemory.Free((void*)list);
    }

    internal static void Free(nint ptr, int len)
    {
        if (len > 0)
            NativeMemory.Free((void*)ptr);
    }
}
