using ModAbi;

namespace CuoModSdk;

/// <summary>
/// A type usable as a system / observer PARAMETER. The parameter list of the lambda a
/// mod hands to <c>ModBuilder.AddSystem</c> IS the declaration of what the system needs
/// — the same shape as a host <c>TinyEcs.Bevy</c> system.
///
/// <para><see cref="Describe"/> runs once, at <c>AddSystem</c> time, and appends this
/// parameter's wire declaration (a <c>Query</c> param with its terms; nothing for
/// <see cref="Commands"/> / <see cref="ModContext"/> / <see cref="Res{T}"/>).
/// <see cref="Create"/> runs per call and builds the instance from the current
/// <c>mod_run</c> / <c>mod_observer</c> input.</para>
///
/// <para>Static abstract members, not <c>new()</c> + instance <c>Fetch</c> like the host:
/// the guest's params are value copies of a pushed snapshot, so there is nothing to
/// keep alive between calls, and closed generics keep the whole resolution
/// reflection-free under the NativeAOT-LLVM publish.</para>
/// </summary>
public interface ISystemParam<TSelf> where TSelf : ISystemParam<TSelf>
{
    /// <summary>Append this parameter's wire declaration. A param that needs no host data declares nothing.</summary>
    static abstract void Describe(ParamDescriber d);

    /// <summary>Build the parameter for one call.</summary>
    static abstract TSelf Create(ParamContext c);
}

/// <summary>
/// Collects the wire parameter declarations of one system as its parameter types
/// describe themselves. Only the SDK's own params write to it — a hand-written
/// <see cref="ISystemParam{TSelf}"/> describes nothing, which is correct for a param
/// that reads no host data.
/// </summary>
public sealed class ParamDescriber
{
    readonly ModHost _host;
    internal readonly List<ParamDecl> Params = new();
    int _queries;
    int _last = -1;

    internal ParamDescriber(ModHost host)
    {
        _host = host;
        // Every system gets a Commands param: the SDK always hands the body a command
        // buffer, and an unused one costs the host nothing.
        Params.Add(new ParamDecl { Kind = ParamKind.Commands });
    }

    internal QueryTermSink BeginQuery() => new(_host);

    internal void EndQuery(QueryTermSink sink)
    {
        Params.Add(new ParamDecl { Kind = ParamKind.Query, Query = new QueryDecl { Terms = sink.Terms } });
        _last = _queries++;
    }

    // The query ordinal the parameter just described (-1 = it declared no query).
    // ModBuilder captures it per parameter position so Create() can index
    // SystemInput.queries without re-running Describe.
    internal int Take()
    {
        var last = _last;
        _last = -1;
        return last;
    }
}

/// <summary>
/// The one call's worth of state a parameter is built from. Opaque to a mod: it exists
/// so <see cref="ISystemParam{TSelf}.Create"/> has a single argument.
/// </summary>
public readonly struct ParamContext
{
    internal readonly ModHost Host;
    internal readonly Commands Commands;
    internal readonly SystemInputView Input;
    internal readonly ObserverInputView Observer;
    internal readonly int QueryIndex;

    internal ParamContext(ModHost host, Commands commands, SystemInputView input, ObserverInputView observer, int queryIndex)
    {
        Host = host;
        Commands = commands;
        Input = input;
        Observer = observer;
        QueryIndex = queryIndex;
    }

    // Rows of the query this parameter declared. Null in an observer, which carries none.
    internal QueryRowsView? Rows => QueryIndex < 0 ? null : Input.Query(QueryIndex);

    internal T Payload<T>() => Observer.Value is { } v ? v.Parse(Host.Json<T>())! : default!;
}

/// <summary>
/// A read-only host singleton, pulled through <c>resource_get</c> when the parameter is
/// built. <c>default</c> when the host has no such resource. Writes go the other way —
/// <c>Commands.SetResource</c>.
/// </summary>
public readonly struct Res<T> : ISystemParam<Res<T>>
{
    readonly T? _value;

    Res(T? value) => _value = value;

    /// <summary>Nothing on the wire: a resource is pulled, not pushed.</summary>
    public static void Describe(ParamDescriber d) { }

    public static Res<T> Create(ParamContext c) => new(Imports.ResourceGet(c.Host.Id<T>(), c.Host.Json<T>()));

    public T? Value => _value;
}
