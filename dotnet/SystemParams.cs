using System.Text.Json;

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
/// the host passes to <c>run</c> / <c>observe</c>.
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
    internal ulong TriggerEntity;
    internal nint TriggerValue;
    internal int TriggerValueLen;
    internal Packet Packet;
    internal Verdict Verdict;

    byte[] _kinds = [];
    int[] _handles = [];
    int _count;
    // The handle Mut write-backs go through as commands.insert (0 = none: query.set).
    internal int CommandsHandle;

    // query.rows() results, fetched on first use per param.
    nint[] _rows = new nint[4];
    int[] _rowCounts = new int[4];
    bool[] _fetched = new bool[4];

    readonly List<Action> _writeBacks = new();
    // Mut<T> row storage, keyed (query param slot << 8 | term index).
    internal readonly Dictionary<int, object> MutColumns = new();

    internal void Begin(byte[] kinds, int[] handles, int count, List<object>? locals)
    {
        _kinds = kinds;
        _handles = handles;
        _count = count;
        Locals = locals;
        Verdict = default;
        CommandsHandle = 0;
        for (var i = 0; i < count; i++)
            if (kinds[i] == EcsAbi.ParamCommands)
            {
                CommandsHandle = handles[i];
                break;
            }
        Commands.Begin(CommandsHandle);
        if (_fetched.Length < count)
        {
            _rows = new nint[count];
            _rowCounts = new int[count];
            _fetched = new bool[count];
        }
    }

    internal int Handle(int slot) => (uint)slot < (uint)_count ? _handles[slot] : 0;

    internal (nint Rows, int Count) Rows(int slot)
    {
        if ((uint)slot >= (uint)_count || _kinds[slot] != EcsAbi.ParamQuery)
            return (0, 0);
        if (!_fetched[slot])
        {
            (_rows[slot], _rowCounts[slot]) = EcsAbi.Rows(_handles[slot]);
            _fetched[slot] = true;
        }
        return (_rows[slot], _rowCounts[slot]);
    }

    internal void AfterRun(Action writeBack) => _writeBacks.Add(writeBack);

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
        MutColumns.Clear();
        for (var i = 0; i < _count; i++)
            if (_fetched[i])
            {
                EcsAbi.FreeRows(_rows[i], _rowCounts[i]);
                _fetched[i] = false;
                _rows[i] = 0;
                _rowCounts[i] = 0;
            }
        _count = 0;
        Locals = null;
        TriggerValue = 0;
        TriggerValueLen = 0;
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
/// (take <c>Opt&lt;Res&lt;T&gt;&gt;</c> to run anyway).
/// </summary>
public readonly struct Res<T> : ISystemParam<Res<T>>
{
    public readonly T Value;

    Res(T value) => Value = value;

    public static void Describe(ParamDescriber d) =>
        d.Add(new ParamDecl { Kind = ParamKind.Res, Path = ModHost.PathOf<T>() });

    public static bool TryCreate(ParamContext c, out Res<T> value)
    {
        value = default;
        if (!ResJson.TryGet(c, out T? parsed))
            return false;
        value = new Res<T>(parsed!);
        return true;
    }
}

internal static unsafe class ResJson
{
    internal static bool TryGet<T>(ParamContext c, out T? parsed)
    {
        parsed = default;
        var handle = c.Scope.Handle(c.Slot);
        if (handle == 0 || !EcsAbi.ResGet(handle, out var ptr, out var len))
            return false;
        try
        {
            parsed = Payload.Parse(new ReadOnlySpan<byte>((void*)ptr, len), c.Host.Json<T>());
        }
        finally
        {
            EcsAbi.Free(ptr, len);
        }
        return parsed is not null;
    }
}

/// <summary>
/// Writable access to a resource: change <see cref="Value"/> and it is written back when
/// the system returns (only when it actually changed). Skipped like <see cref="Res{T}"/>.
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
        if (!ResJson.TryGet(c, out T? parsed))
            return false;
        var info = c.Host.Json<T>();
        var res = new ResMut<T>(parsed!);
        // Compare against our own serialization, not the host's text: the two format
        // differently, and a spurious write marks the resource changed. The snapshot is
        // pooled (a T that is a class is mutated in place, so it must be taken now).
        var original = JsonScratch.Rent(parsed!, info);
        var handle = c.Scope.Handle(c.Slot);
        c.Scope.AfterRun(() =>
        {
            try
            {
                if (JsonScratch.Differs(res.Value, info, original.Span, out var now))
                    EcsAbi.ResSet(handle, now);
            }
            finally
            {
                original.Return();
            }
        });
        value = res;
        return true;
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

/// <summary>JSON payloads as the host sends them: a marker's empty payload reads as <c>{}</c>, <c>null</c> as default.</summary>
internal static class Payload
{
    internal static T? Parse<T>(ReadOnlySpan<byte> json, System.Text.Json.Serialization.Metadata.JsonTypeInfo<T> info)
    {
        if (json.Length == 0)
            json = "{}"u8;
        else if (json.SequenceEqual("null"u8))
            return default;
        return JsonSerializer.Deserialize(json, info);
    }
}

/// <summary>
/// The write-back compares (ResMut, Mut): serialize into reused writers, compare in
/// place; a write goes out of the scratch, without a payload allocation.
/// </summary>
internal static class JsonScratch
{
    static readonly System.Buffers.ArrayBufferWriter<byte> _buf = new(256);
    static readonly Utf8JsonWriter _writer = new(_buf);

    /// <summary>A pooled copy of <paramref name="value"/>'s JSON (Return it when done).</summary>
    internal static Rented Rent<T>(T value, System.Text.Json.Serialization.Metadata.JsonTypeInfo<T> info)
    {
        var json = Write(value, info);
        var arr = System.Buffers.ArrayPool<byte>.Shared.Rent(Math.Max(1, json.Length));
        json.CopyTo(arr);
        return new Rented(arr, json.Length);
    }

    /// <summary>
    /// True when <paramref name="value"/> serializes differently from <paramref name="original"/>;
    /// <paramref name="now"/> is the new JSON, valid until the next scratch use.
    /// </summary>
    internal static bool Differs<T>(T value, System.Text.Json.Serialization.Metadata.JsonTypeInfo<T> info, ReadOnlySpan<byte> original, out ReadOnlySpan<byte> now)
    {
        now = Write(value, info);
        return !now.SequenceEqual(original);
    }

    static ReadOnlySpan<byte> Write<T>(T value, System.Text.Json.Serialization.Metadata.JsonTypeInfo<T> info)
    {
        _buf.ResetWrittenCount();
        _writer.Reset(_buf);
        JsonSerializer.Serialize(_writer, value, info);
        return _buf.WrittenSpan;
    }

    internal readonly struct Rented(byte[] array, int length)
    {
        public ReadOnlySpan<byte> Span => array.AsSpan(0, length);
        public void Return() => System.Buffers.ArrayPool<byte>.Shared.Return(array);
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
        var (list, count) = EcsAbi.EventsRead(handle);
        try
        {
            if (count == 0)
            {
                value = new EventReader<T>([]);
                return true;
            }
            var info = c.Host.Json<T>();
            var events = new T[count];
            var n = 0;
            for (var i = 0; i < count; i++)
                if (Payload.Parse(EcsAbi.StringAt(list, i), info) is { } e)
                    events[n++] = e;
            if (n != events.Length)
                Array.Resize(ref events, n);
            value = new EventReader<T>(events);
            return true;
        }
        finally
        {
            EcsAbi.FreeStrings(list, count);
        }
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
