using System.Runtime.InteropServices;

namespace CuoModSdk;

/// <summary>
/// A type usable as a system / observer PARAMETER. The parameter list of the lambda a
/// mod hands to <c>ModBuilder.AddSystem</c> IS the declaration of what the system needs
/// — the same shape as a host <c>TinyEcs.Bevy</c> system.
///
/// <para><see cref="Describe"/> runs once, at <c>AddSystem</c> time, and appends this
/// parameter's declaration. <see cref="TryCreate"/> runs per call and builds the value
/// from the param handle the host passed; <c>false</c> skips the run (a <see cref="Res{T}"/>
/// the host has no value for — take <see cref="Opt{P}"/> to run anyway).</para>
///
/// <para>Static abstract members keep the whole resolution reflection-free under the
/// NativeAOT-LLVM publish.</para>
/// </summary>
public interface ISystemParam<TSelf> where TSelf : ISystemParam<TSelf>
{
    static abstract void Describe(ParamDescriber d);

    static abstract bool TryCreate(ParamContext c, out TSelf value);
}

internal enum ParamKind : byte { Commands, Query, Res, ResMut, Events }

/// <summary>One declared host parameter (what <c>system.add-*</c> is called with).</summary>
internal sealed class ParamDecl
{
    internal ParamKind Kind;
    internal string Path = "";
    internal List<QueryTerm>? Terms;
}

/// <summary>
/// Collects the parameter declarations of one system / observer as its parameter types
/// describe themselves. The index of each declaration is the index of the param handle
/// the host passes to the system's export.
/// </summary>
public sealed class ParamDescriber
{
    internal readonly ModHost Host;
    internal readonly List<ParamDecl> Params = new();
    int _locals;
    int _last = -1;

    internal ParamDescriber(ModHost host) => Host = host;

    /// <summary>Append a host parameter; the describing parameter gets its index.</summary>
    internal void Add(ParamDecl decl)
    {
        _last = Params.Count;
        Params.Add(decl);
    }

    /// <summary>A guest-only slot (a <see cref="Local{T}"/>), not declared to the host.</summary>
    internal void AddLocal() => _last = _locals++;

    // The slot the parameter just described (-1 = none). Captured per parameter position
    // so TryCreate() can index the param handles without re-running Describe.
    internal int Take()
    {
        var last = _last;
        _last = -1;
        return last;
    }
}

/// <summary>
/// Per-run state shared by a run's parameters. ONE instance, reused by every run (runs
/// never nest); <see cref="Release"/> hands back everything the run borrowed.
/// </summary>
internal sealed class RunScope
{
    internal ModHost Host => ModRuntime.Host;
    internal readonly Commands Commands = new();
    internal List<object>? Locals;
    internal Entry? Entry;
    internal ulong TriggerEntity;
    // The trigger's record (0 for a tag), see TypedTrigger.
    internal nint TriggerValue;
    internal Packet Packet;
    internal Verdict Verdict;

    byte[] _kinds = [];
    int[] _handles = [];
    int _count;

    // query.entities() results, fetched on first use per param.
    nint[] _rows = new nint[4];
    int[] _rowCounts = new int[4];
    bool[] _fetched = new bool[4];
    // entity -> row per query param, built by the first big lookup of a run; the
    // dictionaries are kept (cleared) across runs.
    Dictionary<ulong, int>?[] _index = new Dictionary<ulong, int>?[4];
    bool[] _indexed = new bool[4];
    // Columns fetched this run, freed by Release.
    readonly List<TypedColumnBase> _columns = new();

    readonly List<Action> _writeBacks = new();
    // Mut<T> row storage per [query param slot][term index], cleared by Release.
    object?[]?[] _mutColumns = [];

    internal void Begin(byte[] kinds, int[] handles, int count, List<object>? locals)
    {
        _kinds = kinds;
        _handles = handles;
        _count = count;
        Locals = locals;
        Verdict = default;
        var commands = 0;
        for (var i = 0; i < count; i++)
            if (kinds[i] == EcsAbi.ParamCommands)
            {
                commands = handles[i];
                break;
            }
        Commands.Begin(commands);
        if (_fetched.Length < count)
        {
            _rows = new nint[count];
            _rowCounts = new int[count];
            _fetched = new bool[count];
            Array.Resize(ref _index, count);
            _indexed = new bool[count];
        }
    }

    internal int Handle(int slot) => (uint)slot < (uint)_count ? _handles[slot] : 0;

    internal (nint Entities, int Count) Entities(int slot)
    {
        if ((uint)slot >= (uint)_count || _kinds[slot] != EcsAbi.ParamQuery)
            return (0, 0);
        if (!_fetched[slot])
        {
            (_rows[slot], _rowCounts[slot]) = EcsAbi.Entities(_handles[slot]);
            _fetched[slot] = true;
        }
        return (_rows[slot], _rowCounts[slot]);
    }

    // Below this many rows a lookup scans: cheaper than building the index.
    const int IndexMin = 16;

    /// <summary>The row of <paramref name="id"/> among query param <paramref name="slot"/>'s rows, or -1.</summary>
    internal int RowOf(int slot, nint rows, int count, ulong id)
    {
        if (count < IndexMin)
        {
            for (var i = 0; i < count; i++)
                if (EcsAbi.EntityAt(rows, i) == id)
                    return i;
            return -1;
        }
        var index = _index[slot] ??= new Dictionary<ulong, int>(count);
        if (!_indexed[slot])
        {
            for (var i = 0; i < count; i++)
                index.TryAdd(EcsAbi.EntityAt(rows, i), i);
            _indexed[slot] = true;
        }
        return index.TryGetValue(id, out var row) ? row : -1;
    }

    /// <summary>The column of read term <paramref name="term"/> of query param <paramref name="slot"/> (kept on the system across runs).</summary>
    internal TypedColumn<T> Column<T>(int slot, int term, TypedCodec<T> codec)
    {
        var entry = Entry!;
        var columns = entry.Columns ??= new TypedColumnBase?[entry.Params.Count][];
        var terms = columns[slot] ??= new TypedColumnBase?[entry.Params[slot].Terms!.Count];
        return (TypedColumn<T>)(terms[term] ??= new TypedColumn<T>(codec, (byte)term));
    }

    /// <summary>The Mut storage slot of term <paramref name="term"/> of query param <paramref name="slot"/> for this run.</summary>
    internal ref object? MutColumn(int slot, int term)
    {
        if (_mutColumns.Length <= slot)
            Array.Resize(ref _mutColumns, slot + 1);
        var terms = _mutColumns[slot];
        if (terms == null || terms.Length <= term)
        {
            Array.Resize(ref terms, term + 1);
            _mutColumns[slot] = terms;
        }
        return ref terms[term];
    }

    internal void HoldColumn(TypedColumnBase column) => _columns.Add(column);

    internal void AfterRun(Action writeBack) => _writeBacks.Add(writeBack);

    /// <summary>The cell this system keeps for host param <paramref name="slot"/> across its runs.</summary>
    internal TCell Cell<TCell>(int slot) where TCell : class, new()
    {
        var entry = Entry!;
        if (entry.ResCells.Length <= slot)
            Array.Resize(ref entry.ResCells, slot + 1);
        return entry.ResCells[slot] as TCell ?? (TCell)(entry.ResCells[slot] = new TCell());
    }

    /// <summary>After the body: the pending spawn, then the write-backs (they land after the run's commands).</summary>
    internal void End()
    {
        Commands.Flush();
        for (var i = 0; i < _writeBacks.Count; i++)
            _writeBacks[i]();
        Commands.Flush();
    }

    internal void Release()
    {
        Commands.End();
        _writeBacks.Clear();
        foreach (var terms in _mutColumns)
            if (terms != null)
                Array.Clear(terms);
        for (var i = 0; i < _columns.Count; i++)
            _columns[i].Release();
        _columns.Clear();
        TypedArena.Reset();
        for (var i = 0; i < _count; i++)
        {
            if (_fetched[i])
            {
                EcsAbi.FreeEntities(_rows[i], _rowCounts[i]);
                _fetched[i] = false;
                _rows[i] = 0;
                _rowCounts[i] = 0;
            }
            if (_indexed[i])
            {
                _index[i]!.Clear();
                _indexed[i] = false;
            }
        }
        _count = 0;
        Entry = null;
        Locals = null;
        TriggerValue = 0;
        Packet = default;
    }
}

/// <summary>
/// The one call's worth of state a parameter is built from. Opaque to a mod: it exists
/// so <see cref="ISystemParam{TSelf}.TryCreate"/> has a single argument.
/// </summary>
public readonly struct ParamContext
{
    internal readonly RunScope Scope;
    internal readonly int Slot;

    internal ParamContext(RunScope scope, int slot)
    {
        Scope = scope;
        Slot = slot;
    }

    internal ModHost Host => Scope.Host;
}

/// <summary>
/// Read-only access to a resource. The system is skipped while the client has none
/// (take <c>Opt&lt;Res&lt;T&gt;&gt;</c> to run anyway). A value read on an earlier run is
/// reused while the host reports it unchanged: treat it as read-only.
/// </summary>
public readonly struct Res<T> : ISystemParam<Res<T>>
{
    readonly ResCell<T> _cell;

    Res(ResCell<T> cell) => _cell = cell;

    public T Value => _cell != null ? _cell.Value! : default!;

    public static void Describe(ParamDescriber d) =>
        d.Add(new ParamDecl { Kind = ParamKind.Res, Path = ModHost.PathOf<T>() });

    public static bool TryCreate(ParamContext c, out Res<T> value)
    {
        value = default;
        var handle = c.Scope.Handle(c.Slot);
        if (handle == 0)
            return false;
        var cell = c.Scope.Cell<ResCell<T>>(c.Slot);
        if (!cell.Begin(handle))
            return false;
        value = new Res<T>(cell);
        return true;
    }
}

/// <summary>
/// One <see cref="Res{T}"/> parameter's state across its system's runs: whether the host
/// had a value, and that value. <c>res.unchanged</c> lets a run skip <c>get-&lt;x&gt;</c>.
/// </summary>
internal sealed class ResCell<T>
{
    bool _known, _present;
    internal T? Value;

    /// <summary>Starts a run; false when the host has no value.</summary>
    internal bool Begin(int handle)
    {
        if (_known && EcsAbi.ResUnchanged(handle))
            return _present;
        _known = true;
        _present = Typed<T>.Record.Get(handle, out var value);
        Value = value;
        return _present;
    }
}

/// <summary>
/// Writable access to a resource: change <see cref="Value"/> and it is written back
/// (<c>put-&lt;x&gt;</c>) when the system returns, only when it actually changed. Skipped
/// like <see cref="Res{T}"/>. While the host reports it unchanged, the previous run's
/// value is reused.
/// </summary>
public sealed class ResMut<T> : ISystemParam<ResMut<T>>
{
    public T Value;

    ResMut(T value) => Value = value;

    public static void Describe(ParamDescriber d) =>
        d.Add(new ParamDecl { Kind = ParamKind.ResMut, Path = ModHost.PathOf<T>() });

    public static bool TryCreate(ParamContext c, out ResMut<T> value)
    {
        value = null!;
        var handle = c.Scope.Handle(c.Slot);
        if (handle == 0)
            return false;
        var cell = c.Scope.Cell<ResMutCell<T>>(c.Slot);
        if (!cell.Begin(handle))
            return false;
        var res = new ResMut<T>(cell.Value!);
        c.Scope.AfterRun(() => cell.WriteBack(handle, res.Value));
        value = res;
        return true;
    }
}

/// <summary>
/// A <see cref="ResMut{T}"/> parameter's value and the host record it was lifted from
/// (what the write-back compares against), kept across runs.
/// </summary>
internal sealed unsafe class ResMutCell<T>
{
    bool _valid;
    internal T? Value;
    byte* _rec;

    /// <summary>Starts a run; false when the host has no value.</summary>
    internal bool Begin(int handle)
    {
        if (_valid && EcsAbi.ResUnchanged(handle))
            return true;
        var codec = Typed<T>.Record;
        Drop(codec);
        var rec = (byte*)NativeMemory.AllocZeroed((nuint)Math.Max(1, codec.Size));
        if (!codec.GetRecord(handle, rec))
        {
            NativeMemory.Free(rec);
            return false;
        }
        _rec = rec;
        Value = codec.Lift(rec);
        _valid = true;
        return true;
    }

    internal void WriteBack(int handle, T now)
    {
        var codec = Typed<T>.Record;
        if (!codec.Differs(now, _rec))
            return;
        var rec = stackalloc byte[codec.Size];
        codec.LowerInto(now, rec);
        codec.Put(handle, rec);
        // The cached object may have been mutated in place: read afresh next run.
        Drop(codec);
    }

    void Drop(TypedCodec codec)
    {
        _valid = false;
        if (_rec == null)
            return;
        codec.FreeElement(_rec);
        NativeMemory.Free(_rec);
        _rec = null;
    }
}

/// <summary>
/// A parameter that may be missing (<c>Opt&lt;Res&lt;Time&gt;&gt;</c>): the system runs
/// anyway, with <see cref="HasValue"/> false.
/// </summary>
public readonly struct Opt<P> : ISystemParam<Opt<P>> where P : ISystemParam<P>
{
    public readonly bool HasValue;
    public readonly P Value;

    Opt(bool has, P value)
    {
        HasValue = has;
        Value = value;
    }

    public static void Describe(ParamDescriber d) => P.Describe(d);

    public static bool TryCreate(ParamContext c, out Opt<P> value)
    {
        var has = P.TryCreate(c, out var inner);
        value = new Opt<P>(has, inner);
        return true;
    }
}

/// <summary>The events of type <typeparamref name="T"/> sent since this system last ran.</summary>
public readonly struct EventReader<T> : ISystemParam<EventReader<T>>
{
    readonly T[] _events;

    EventReader(T[] events) => _events = events;

    public static void Describe(ParamDescriber d) =>
        d.Add(new ParamDecl { Kind = ParamKind.Events, Path = ModHost.PathOf<T>() });

    public static bool TryCreate(ParamContext c, out EventReader<T> value)
    {
        var handle = c.Scope.Handle(c.Slot);
        if (handle == 0)
        {
            value = new EventReader<T>([]);
            return true;
        }
        var events = Typed<T>.Event ?? throw new InvalidOperationException($"{ModHost.PathOf<T>()} is not an event");
        value = new EventReader<T>(events.Read(handle));
        return true;
    }

    public ReadOnlySpan<T> Read() => _events ?? [];

    public int Count => _events?.Length ?? 0;

    public bool IsEmpty => Count == 0;

    public ReadOnlySpan<T>.Enumerator GetEnumerator() => Read().GetEnumerator();
}

/// <summary>
/// State that belongs to one system and survives between its runs (a counter, a timer,
/// the window it spawned). Guest-only: never declared to the host.
/// </summary>
public sealed class Local<T> : ISystemParam<Local<T>> where T : new()
{
    public T Value = new();

    public static void Describe(ParamDescriber d) => d.AddLocal();

    public static bool TryCreate(ParamContext c, out Local<T> value)
    {
        var locals = c.Scope.Locals!;
        while (locals.Count <= c.Slot)
            locals.Add(null!);
        if (locals[c.Slot] is not Local<T> local)
            locals[c.Slot] = local = new Local<T>();
        value = local;
        return true;
    }
}
