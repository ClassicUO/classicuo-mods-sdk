using System.Buffers;
using System.Runtime.InteropServices;
using Interop = ModWorld.wit.Imports.tinyecs.modding.v0_1_0.EcsImportsInterop;

namespace CuoModSdk;

/// <summary>
/// The per-run half of <c>tinyecs:modding/ecs</c> (commands / query / res / events),
/// lowered by hand over the generated raw imports (Wit/*EcsImportsInterop.cs). The
/// generated wrapper classes allocate per call (UTF-16 strings, GCHandles, cleanup
/// lists); these pass UTF-8 straight out of reused buffers and hand results back as
/// spans over the host-allocated memory, freed when the run ends. Setup-time calls
/// (system / app) use the generated classes: they run once.
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

    // ── a bundle being built: (path, json) pairs, the JSON in one reused buffer ──
    internal static readonly ArrayBufferWriter<byte> Json = new(1024);
    static readonly List<(string Path, int Offset, int Length)> _bundle = new();

    internal static int BundleCount => _bundle.Count;

    internal static void BundleClear()
    {
        _bundle.Clear();
        Json.ResetWrittenCount();
    }

    /// <summary>Adds a component whose JSON the caller just appended to <see cref="Json"/> from <paramref name="start"/>.</summary>
    internal static void BundleAdd(string path, int start) => _bundle.Add((path, start, Json.WrittenCount - start));

    internal static void BundleAddMarker(string path)
    {
        var start = Json.WrittenCount;
        "{}"u8.CopyTo(Json.GetSpan(2));
        Json.Advance(2);
        BundleAdd(path, start);
    }

    // list<tuple<string, string>> element / list<string> element: wasm32 pointers.
    [StructLayout(LayoutKind.Sequential)]
    struct PathJson
    {
        public int PathPtr, PathLen, JsonPtr, JsonLen;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct Str
    {
        public int Ptr, Len;
    }

    static PathJson[] _tuples = new PathJson[16];

    // Lowers the bundle into _tuples; `json` is the pinned Json buffer.
    static int Lower(byte* json)
    {
        var n = _bundle.Count;
        if (_tuples.Length < n)
            _tuples = new PathJson[Math.Max(n, _tuples.Length * 2)];
        for (var i = 0; i < n; i++)
        {
            var (path, off, len) = _bundle[i];
            var (pp, pl) = Path(path);
            _tuples[i] = new PathJson { PathPtr = (int)pp, PathLen = pl, JsonPtr = (int)(nint)(json + off), JsonLen = len };
        }
        return n;
    }

    /// <summary>commands.spawn(bundle) with the current bundle; clears it.</summary>
    internal static ulong Spawn(int commands)
    {
        long id;
        fixed (byte* json = Json.WrittenSpan)
        {
            var n = Lower(json);
            // Lower may have grown _tuples: pin the current array.
            fixed (PathJson* cur = _tuples)
                id = Interop.Commands.SpawnWasmInterop.wasmImportSpawn(commands, (nint)cur, n);
        }
        BundleClear();
        return (ulong)id;
    }

    /// <summary>commands.insert(entity, bundle) with the current bundle; clears it.</summary>
    internal static void Insert(int commands, ulong entity)
    {
        fixed (byte* json = Json.WrittenSpan)
        {
            var n = Lower(json);
            fixed (PathJson* cur = _tuples)
                Interop.Commands.InsertWasmInterop.wasmImportInsert(commands, (long)entity, (nint)cur, n);
        }
        BundleClear();
    }

    internal static void Remove(int commands, ulong entity, string path)
    {
        var (pp, pl) = Path(path);
        var item = new Str { Ptr = (int)pp, Len = pl };
        Interop.Commands.RemoveWasmInterop.wasmImportRemove(commands, (long)entity, (nint)(&item), 1);
    }

    internal static void Despawn(int commands, ulong entity) =>
        Interop.Commands.DespawnWasmInterop.wasmImportDespawn(commands, (long)entity);

    /// <summary>commands.send(path, json) with the JSON appended to <see cref="Json"/> from <paramref name="start"/>.</summary>
    internal static void Send(int commands, string path, int start)
    {
        var (pp, pl) = Path(path);
        fixed (byte* json = Json.WrittenSpan)
            Interop.Commands.SendWasmInterop.wasmImportSend(commands, pp, pl, (nint)(json + start), Json.WrittenCount - start);
    }

    /// <summary>commands.set-resource(path, json) with the JSON appended to <see cref="Json"/> from <paramref name="start"/>.</summary>
    internal static void SetResource(int commands, string path, int start)
    {
        var (pp, pl) = Path(path);
        fixed (byte* json = Json.WrittenSpan)
            Interop.Commands.SetResourceWasmInterop.wasmImportSetResource(commands, pp, pl, (nint)(json + start), Json.WrittenCount - start);
    }

    internal static void QuerySet(int query, ulong entity, byte index, ReadOnlySpan<byte> json)
    {
        fixed (byte* p = json)
            Interop.Query.SetWasmInterop.wasmImportSet(query, (long)entity, index, (nint)p, json.Length);
    }

    internal static void ResSet(int res, ReadOnlySpan<byte> json)
    {
        fixed (byte* p = json)
            Interop.Res.SetWasmInterop.wasmImportSet(res, (nint)p, json.Length);
    }

    internal static bool ResUnchanged(int res) => Interop.Res.UnchangedWasmInterop.wasmImportUnchanged(res) != 0;

    // ── results: host-allocated (cabi_realloc = malloc) and owned by the guest ──

    /// <summary>query.rows(): the row array (16 bytes per row) and its count. Free with <see cref="FreeRows"/>.</summary>
    internal static (nint Rows, int Count) Rows(int query)
    {
        var ret = stackalloc int[2];
        Interop.Query.RowsWasmInterop.wasmImportRows(query, (nint)ret);
        return (ret[0], ret[1]);
    }

    internal static ulong RowEntity(nint rows, int i) => *(ulong*)(rows + i * 16);

    internal static int RowValueCount(nint rows, int i) => *(int*)(rows + i * 16 + 12);

    internal static ReadOnlySpan<byte> RowValue(nint rows, int i, int index)
    {
        var values = *(nint*)(rows + i * 16 + 8);
        var v = (int*)(values + index * 8);
        return new ReadOnlySpan<byte>((void*)(nint)v[0], v[1]);
    }

    internal static void FreeRows(nint rows, int count)
    {
        for (var i = 0; i < count; i++)
        {
            var row = rows + i * 16;
            var values = *(nint*)(row + 8);
            var n = *(int*)(row + 12);
            FreeStrings(values, n);
        }
        if (count > 0)
            NativeMemory.Free((void*)rows);
    }

    /// <summary>res.get(): false when the host has none. Free <paramref name="json"/> with <see cref="Free"/>.</summary>
    internal static bool ResGet(int res, out nint ptr, out int len)
    {
        var ret = stackalloc int[3];
        Interop.Res.GetWasmInterop.wasmImportGet(res, (nint)ret);
        ptr = ret[1];
        len = ret[2];
        return (byte)ret[0] != 0;
    }

    /// <summary>events.read(): the list&lt;string&gt; (8 bytes per element). Free with <see cref="FreeStrings"/>.</summary>
    internal static (nint List, int Count) EventsRead(int events)
    {
        var ret = stackalloc int[2];
        Interop.Events.ReadWasmInterop.wasmImportRead(events, (nint)ret);
        return (ret[0], ret[1]);
    }

    internal static ReadOnlySpan<byte> StringAt(nint list, int i)
    {
        var v = (int*)(list + i * 8);
        return new ReadOnlySpan<byte>((void*)(nint)v[0], v[1]);
    }

    internal static void FreeStrings(nint list, int count)
    {
        for (var i = 0; i < count; i++)
        {
            var v = (int*)(list + i * 8);
            if (v[1] > 0)
                NativeMemory.Free((void*)(nint)v[0]);
        }
        if (count > 0)
            NativeMemory.Free((void*)list);
    }

    internal static void Free(nint ptr, int len)
    {
        if (len > 0)
            NativeMemory.Free((void*)ptr);
    }
}
