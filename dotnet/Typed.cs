using System.Runtime.InteropServices;
using CuoModSdk.Types;

namespace CuoModSdk;

// The typed half of the contract (cuo:modding/components, generated into cuo-mod.wit by
// tools/mod-typegen): the curated host types cross as canonical-ABI records instead of
// JSON. A mod still names the generated DTOs (Hits, Node, …); each codec (generated into
// Typed.g.cs by tools/mod-typegen) maps one DTO onto its record's canonical-ABI memory
// layout and calls the raw imports, so a column / write / insert / event costs no JSON
// and no per-call managed garbage beyond what the DTO itself holds (strings, arrays).
// This file is the generic machinery; everything not in TypedCodecs keeps the JSON path.

/// <summary>A curated type's record codec, untyped: what a pending spawn replays.</summary>
internal abstract unsafe class TypedCodec
{
    /// <summary>Record size (= list element stride).</summary>
    internal abstract int Size { get; }

    /// <summary>The type has an <c>entity-builder</c> method (read-only types do not).</summary>
    internal virtual bool CanBuild => true;

    /// <summary><c>[method]entity-builder.&lt;x&gt;</c> with the record at <paramref name="rec"/>: the new builder handle.</summary>
    internal abstract int Build(int builder, byte* rec);

    /// <summary>A pending spawn that turned out mixed goes out as JSON: re-add this record to the JSON bundle.</summary>
    internal abstract void AddJson(Commands commands, byte* rec);

    /// <summary>Frees what a host-written record owns (strings, lists); the record itself is the caller's.</summary>
    internal virtual void FreeElement(byte* rec) { }
}

/// <summary>A curated DTO's record codec.</summary>
internal abstract unsafe class TypedCodec<T> : TypedCodec
{
    internal abstract T Lift(byte* rec);

    /// <summary>Writes the record; <paramref name="rec"/> is <see cref="Size"/> bytes, already zeroed. Strings go to <see cref="TypedArena"/>.</summary>
    protected abstract void Lower(in T value, byte* rec);

    internal void LowerInto(in T value, byte* rec)
    {
        new Span<byte>(rec, Size).Clear();
        Lower(value, rec);
    }

    internal virtual bool HasColumn => true;

    /// <summary><c>column-&lt;x&gt;(q, term)</c> into the 8-byte return area <paramref name="ret"/>.</summary>
    internal virtual void Column(int query, byte term, nint ret) => throw new NotSupportedException();

    /// <summary>The type has a <c>set-&lt;x&gt;</c> (a writable component).</summary>
    internal virtual bool CanSet => true;

    internal virtual void Set(int query, ulong entity, byte term, byte* rec) => throw new NotSupportedException();

    /// <summary>The type is a resource with a <c>get-&lt;x&gt;</c>.</summary>
    internal virtual bool HasGet => false;

    internal virtual bool Get(int res, out T value) => throw new NotSupportedException();

    /// <summary>The type is a writable resource with a <c>set-resource-&lt;x&gt;</c>.</summary>
    internal virtual bool CanSetResource => false;

    internal virtual void SetResource(int commands, byte* rec) => throw new NotSupportedException();

    /// <summary>Whether <paramref name="now"/> differs from the record at <paramref name="original"/> (the host's value).</summary>
    internal virtual bool Differs(in T now, byte* original)
    {
        var n = Size;
        var mine = stackalloc byte[n];
        LowerInto(now, mine);
        return !new ReadOnlySpan<byte>(mine, n).SequenceEqual(new ReadOnlySpan<byte>(original, n));
    }

    internal override void AddJson(Commands commands, byte* rec) => commands.AddToBundle(Lift(rec));
}

/// <summary>A curated zero-size tag (<c>cuo:ui/movable</c>, …): a builder method without a value.</summary>
internal sealed unsafe class TagCodec(string path, delegate*<int, int> build) : TypedCodec
{
    internal override int Size => 0;

    internal override int Build(int builder, byte* rec) => build(builder);

    internal override void AddJson(Commands commands, byte* rec) => EcsAbi.BundleAddMarker(path);
}

/// <summary>A curated event, typed: <c>send-&lt;x&gt;</c> / <c>read-&lt;x&gt;</c> (a zero-size one reads a count).</summary>
internal abstract class TypedEvent<T>
{
    internal abstract void Send(int commands, in T value);

    internal abstract T[] Read(int events);
}

/// <summary>One flat core param of a typed trigger record: its C# core type, its offset in the record, its memory type.</summary>
internal readonly record struct TypedFlat(string Core, int Offset, string Store);

/// <summary>
/// A curated component / event an observer takes typed: the export's leading params are
/// <c>entity: entity, value: &lt;Record&gt;</c> (a tag: <c>entity</c> alone) instead of
/// <c>trigger: trigger-data</c>. <see cref="Flat"/> is the record's flattened core params
/// (ModDescribe's glue stores them back into the record); <see cref="Codec"/> lifts / frees it.
/// </summary>
internal sealed class TypedTrigger(string? record, int size, int align, TypedFlat[] flat, TypedCodec? codec)
{
    internal readonly string? Record = record;
    internal readonly int Size = size;
    internal readonly int Align = align;
    internal readonly TypedFlat[] Flat = flat;
    internal readonly TypedCodec? Codec = codec;
}

internal static class Typed<T>
{
    internal static readonly TypedCodec<T>? Codec = TypedCodecs.Of<T>();

    /// <summary>A curated tag type (inserted through its builder method).</summary>
    internal static readonly TagCodec? Tag = TypedCodecs.TagOf<T>();

    internal static readonly TypedEvent<T>? Event = TypedCodecs.EventOf<T>();
}

internal static unsafe partial class TypedCodecs
{
    // string { ptr, len } — a lifted element owns its bytes (cabi_realloc = malloc).
    internal static string ReadString(byte* p)
    {
        var len = *(int*)(p + 4);
        return len == 0 ? "" : System.Text.Encoding.UTF8.GetString((byte*)(nint)(*(int*)p), len);
    }

    internal static void FreeString(byte* p)
    {
        if (*(int*)(p + 4) > 0)
            NativeMemory.Free((void*)(nint)(*(int*)p));
    }

    internal static void WriteString(string? s, byte* p)
    {
        var (ptr, len) = TypedArena.Utf8(s);
        *(int*)p = (int)ptr;
        *(int*)(p + 4) = len;
    }
}

/// <summary>
/// UTF-8 for lowered strings: native memory that never moves, valid until the run ends
/// (a pending spawn keeps lowered records until its chain closes).
/// </summary>
internal static unsafe class TypedArena
{
    const int ChunkSize = 4096;
    static byte* _chunk;
    static int _used, _cap;
    static readonly List<nint> _retired = new();

    internal static (nint Ptr, int Len) Utf8(string? s)
    {
        if (string.IsNullOrEmpty(s))
            return (0, 0);
        var n = System.Text.Encoding.UTF8.GetByteCount(s);
        if (_used + n > _cap)
        {
            if (_chunk != null)
                _retired.Add((nint)_chunk);
            _cap = Math.Max(ChunkSize, n);
            _chunk = (byte*)NativeMemory.AlignedAlloc((nuint)_cap, 8);
            _used = 0;
        }
        var p = _chunk + _used;
        System.Text.Encoding.UTF8.GetBytes(s, new Span<byte>(p, n));
        _used += n;
        return ((nint)p, n);
    }

    /// <summary><paramref name="size"/> bytes aligned to <paramref name="align"/> (a lowered list's items).</summary>
    internal static byte* Alloc(int size, int align)
    {
        if (size == 0)
            return (byte*)align;
        var at = (_used + align - 1) & ~(align - 1);
        if (_chunk == null || at + size > _cap)
        {
            if (_chunk != null)
                _retired.Add((nint)_chunk);
            _cap = Math.Max(ChunkSize, size);
            _chunk = (byte*)NativeMemory.AlignedAlloc((nuint)_cap, 8);
            _used = 0;
            at = 0;
        }
        _used = at + size;
        return _chunk + at;
    }

    internal static void Reset()
    {
        foreach (var p in _retired)
            NativeMemory.AlignedFree((void*)p);
        _retired.Clear();
        _used = 0;
    }
}

/// <summary>The calls around the per-type ones: builders, entities, list frees.</summary>
internal static unsafe class TypedAbi
{
    [DllImport("cuo:modding/components@0.1.0", EntryPoint = "[resource-drop]entity-builder"), WasmImportLinkage]
    static extern void DropBuilder(int handle);

    [DllImport("cuo:modding/components@0.1.0", EntryPoint = "spawn"), WasmImportLinkage]
    static extern int SpawnImport(int commands);

    [DllImport("cuo:modding/components@0.1.0", EntryPoint = "entity-of"), WasmImportLinkage]
    static extern int EntityOfImport(int commands, long entity);

    [DllImport("cuo:modding/components@0.1.0", EntryPoint = "[method]entity-builder.id"), WasmImportLinkage]
    static extern long IdImport(int builder);

    internal static void Drop(int builder) => DropBuilder(builder);

    internal static int Spawn(int commands) => SpawnImport(commands);

    internal static int EntityOf(int commands, ulong entity) => EntityOfImport(commands, (long)entity);

    internal static ulong Id(int builder) => (ulong)IdImport(builder);

    /// <summary><c>commands</c>: insert one record (or a tag: <paramref name="rec"/> unused) on <paramref name="entity"/>.</summary>
    internal static void Insert(int commands, ulong entity, TypedCodec codec, byte* rec)
    {
        var b = EntityOf(commands, entity);
        var next = codec.Build(b, rec);
        Drop(b);
        Drop(next);
    }

    /// <summary>A column: the list (host-allocated, owned by the guest) and its count.</summary>
    internal static (nint List, int Count) Column<T>(TypedCodec<T> codec, int query, byte term)
    {
        var ret = stackalloc int[2];
        codec.Column(query, term, (nint)ret);
        return (ret[0], ret[1]);
    }

    internal static void FreeColumn<T>(TypedCodec<T> codec, nint list, int count)
    {
        if (count <= 0)
            return;
        var size = codec.Size;
        for (var i = 0; i < count; i++)
            codec.FreeElement((byte*)list + i * size);
        NativeMemory.Free((void*)list);
    }
}

/// <summary>
/// One typed column of one query param, kept on its system across runs (no allocation
/// per run): fetched on first read in a run, freed when the run ends.
/// </summary>
internal abstract class TypedColumnBase
{
    internal abstract void Release();
}

internal sealed unsafe class TypedColumn<T>(TypedCodec<T> codec, byte term) : TypedColumnBase
{
    internal readonly TypedCodec<T> Codec = codec;
    nint _list;
    int _count;
    bool _fetched;

    internal byte* Element(RunScope scope, int slot, int row)
    {
        if (!_fetched)
        {
            (_list, _count) = TypedAbi.Column(Codec, scope.Handle(slot), term);
            _fetched = true;
            scope.HoldColumn(this);
        }
        if ((uint)row >= (uint)_count)
            throw new InvalidOperationException($"typed column has {_count} row(s), asked for #{row}");
        return (byte*)_list + row * Codec.Size;
    }

    internal T Get(RunScope scope, int slot, int row) => Codec.Lift(Element(scope, slot, row));

    internal override void Release()
    {
        TypedAbi.FreeColumn(Codec, _list, _count);
        _list = 0;
        _count = 0;
        _fetched = false;
    }
}

internal static unsafe class TypedStrings
{
    /// <summary>Two UTF-8 string records { ptr, len } hold the same bytes.</summary>
    internal static bool SameRecord(byte* a, byte* b)
    {
        var len = *(int*)(a + 4);
        return len == *(int*)(b + 4)
            && (len == 0 || new ReadOnlySpan<byte>((byte*)(nint)(*(int*)a), len).SequenceEqual(new ReadOnlySpan<byte>((byte*)(nint)(*(int*)b), len)));
    }

    /// <summary><paramref name="s"/> equals the UTF-8 string record at <paramref name="rec"/> (null reads as empty).</summary>
    internal static bool Same(string? s, byte* rec)
    {
        var len = *(int*)(rec + 4);
        s ??= "";
        if (System.Text.Encoding.UTF8.GetByteCount(s) != len)
            return false;
        if (len == 0)
            return true;
        Span<byte> buf = len <= 256 ? stackalloc byte[len] : new byte[len];
        System.Text.Encoding.UTF8.GetBytes(s, buf);
        return buf.SequenceEqual(new ReadOnlySpan<byte>((byte*)(nint)(*(int*)rec), len));
    }
}
