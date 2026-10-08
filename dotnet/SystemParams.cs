using System.Text.Json;
using ModAbi;

namespace CuoModSdk;

/// <summary>
/// A type usable as a system / observer PARAMETER. The parameter list of the lambda a
/// mod hands to <c>ModBuilder.AddSystem</c> IS the declaration of what the system needs
/// — the same shape as a host <c>TinyEcs.Bevy</c> system.
///
/// <para><see cref="Describe"/> runs once, at <c>AddSystem</c> time, and appends this
/// parameter's wire declaration. <see cref="TryCreate"/> runs per call and builds the
/// value from the data the host pushed; <c>false</c> skips the run (a <see cref="Res{T}"/>
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

/// <summary>
/// Collects the wire parameter declarations of one system / observer as its parameter
/// types describe themselves. The param index of each declaration is what the host's
/// pushed data is keyed by.
/// </summary>
public sealed class ParamDescriber
{
    internal readonly ModHost Host;
    internal readonly List<ParamDeclT> Params = new();
    int _locals;
    int _last = -1;

    internal ParamDescriber(ModHost host) => Host = host;

    /// <summary>Append a wire parameter; the describing parameter gets its index.</summary>
    internal void Add(ParamDeclT decl)
    {
        _last = Params.Count;
        Params.Add(decl);
    }

    /// <summary>A guest-only slot (a <see cref="Local{T}"/>), not on the wire.</summary>
    internal void AddLocal() => _last = _locals++;

    // The slot the parameter just described (-1 = none). Captured per parameter position
    // so TryCreate() can index the pushed data without re-running Describe.
    internal int Take()
    {
        var last = _last;
        _last = -1;
        return last;
    }
}

/// <summary>Per-run state shared by a run's parameters.</summary>
internal sealed class RunScope
{
    internal required ModHost Host;
    internal required Commands Commands;
    internal required ParamsView Input;
    internal required List<object> Locals;
    internal List<Action>? WriteBacks;
    // Mut<T> row storage, keyed (query param slot << 8 | term index).
    internal Dictionary<int, object>? MutColumns;
    internal ulong TriggerEntity;
    internal CompView? TriggerValue;

    internal void AfterRun(Action writeBack) => (WriteBacks ??= new List<Action>()).Add(writeBack);
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
        d.Add(new ParamDeclT { Kind = ParamKind.Res, TypeId = d.Host.Id<T>() });

    public static bool TryCreate(ParamContext c, out Res<T> value)
    {
        value = default;
        if (c.Scope.Input.Resource(c.Slot) is not { } v || v.Parse(c.Host.Json<T>()) is not { } parsed)
            return false;
        value = new Res<T>(parsed);
        return true;
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
        d.Add(new ParamDeclT { Kind = ParamKind.ResMut, TypeId = d.Host.Id<T>() });

    public static bool TryCreate(ParamContext c, out ResMut<T> value)
    {
        value = null!;
        var info = c.Host.Json<T>();
        if (c.Scope.Input.Resource(c.Slot) is not { } v || v.Parse(info) is not { } parsed)
            return false;
        var res = new ResMut<T>(parsed);
        // Compare against our own serialization, not the host's text: the two format
        // differently, and a spurious write marks the resource changed. The snapshot is
        // pooled (a T that is a class is mutated in place, so it must be taken now); only
        // a real change materializes a payload.
        var original = JsonScratch.Rent(parsed, info);
        var scope = c.Scope;
        scope.AfterRun(() =>
        {
            try
            {
                if (JsonScratch.Differs(res.Value, info, original.Span, out var now))
                    scope.Commands.ResourceSetRaw(scope.Host.Id<T>(), now);
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

/// <summary>
/// The write-back compares (ResMut, Mut): serialize into reused writers, compare in
/// place, and allocate a payload only for a value that really changed.
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

    /// <summary>True (and the new payload) when <paramref name="value"/> serializes differently from <paramref name="original"/>.</summary>
    internal static bool Differs<T>(T value, System.Text.Json.Serialization.Metadata.JsonTypeInfo<T> info, ReadOnlySpan<byte> original, out byte[] now)
    {
        var json = Write(value, info);
        if (json.SequenceEqual(original))
        {
            now = [];
            return false;
        }
        now = json.ToArray();
        return true;
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
        d.Add(new ParamDeclT { Kind = ParamKind.Events, TypeId = d.Host.Id<T>() });

    public static bool TryCreate(ParamContext c, out EventReader<T> value)
    {
        var info = c.Host.Json<T>();
        var values = c.Scope.Input.EventValues(c.Slot);
        if (values.Count == 0)
        {
            value = new EventReader<T>([]);
            return true;
        }
        var events = new T[values.Count];
        var n = 0;
        for (var i = 0; i < values.Count; i++)
            if (values[i].Parse(info) is { } e)
                events[n++] = e;
        if (n != events.Length)
            Array.Resize(ref events, n);
        value = new EventReader<T>(events);
        return true;
    }

    public ReadOnlySpan<T> Read() => _events ?? [];

    public int Count => _events?.Length ?? 0;

    public bool IsEmpty => Count == 0;

    public ReadOnlySpan<T>.Enumerator GetEnumerator() => Read().GetEnumerator();
}

/// <summary>
/// State that belongs to one system and survives between its runs (a counter, a timer,
/// the window it spawned). Guest-only: nothing on the wire.
/// </summary>
public sealed class Local<T> : ISystemParam<Local<T>> where T : new()
{
    public T Value = new();

    public static void Describe(ParamDescriber d) => d.AddLocal();

    public static bool TryCreate(ParamContext c, out Local<T> value)
    {
        var locals = c.Scope.Locals;
        while (locals.Count <= c.Slot)
            locals.Add(null!);
        if (locals[c.Slot] is not Local<T> local)
            locals[c.Slot] = local = new Local<T>();
        value = local;
        return true;
    }
}
