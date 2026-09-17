using System.Text.Json.Serialization;
using ModAbi;

namespace CuoModSdk;

/// <summary>
/// What a mod's <see cref="Mod.Setup"/> populates: systems, observers, hotkeys and the
/// optional packet filters.
///
/// <para>Systems and observers are LAMBDAS whose PARAMETER TYPES declare what they need
/// — the same shape as a host <c>TinyEcs.Bevy</c> plugin:</para>
/// <code>
/// m.AddSystem((Query&lt;Data&lt;Hits&gt;, Filter&lt;With&lt;Player&gt;, Changed&lt;Hits&gt;&gt;&gt; q, Commands cmds) =&gt; …)
///  .InStage(Stage.Update);
///
/// m.AddObserver((On&lt;ModHotkeyFired&gt; t, Commands cmds) =&gt; …);
/// </code>
/// <para>Type ids are interned here — a mod names generated <c>CuoModSdk.Types</c>
/// payload types and never sees a <c>ushort</c>.</para>
/// </summary>
public sealed class ModBuilder
{
    readonly ModHost _host;
    readonly List<SystemDecl> _sysDecls = new();
    readonly List<ObserverDecl> _obsDecls = new();
    readonly List<Types.ModHotkeyBinding> _hotkeys = new();

    internal readonly List<Action<SystemInputView, Commands>> SystemFns = new();
    internal readonly List<Action<ObserverInputView, Commands>> ObserverFns = new();
    internal PacketFilter? Filter;
    internal PacketFilter? FilterOut;

    internal ModBuilder(ModHost host) => _host = host;

    /// <summary>
    /// Register a source-generated JSON context for the mod's OWN types (storage blobs,
    /// custom event payloads). Host payload types need no registration. Reflection-based
    /// STJ does not survive the wasi-wasm ILC publish, hence the context.
    /// </summary>
    public void UseJson(JsonSerializerContext context) => _host.UseJson(context);

    /// <summary>
    /// Register a hotkey the host should fire back as <c>cuo:input/hotkey</c>. All
    /// bindings are written as one <c>cuo:input/mod-hotkeys</c> resource at startup —
    /// per mod, so this never disturbs another mod's bindings, and re-publishing (call
    /// <c>ctx.Commands.SetResource</c> with a fresh DTO) rebinds at runtime.
    /// <para><paramref name="consume"/> is per binding: <c>true</c> = this mod owns the
    /// combo (the key is swallowed and the host profile action bound to the same combo
    /// does NOT fire), <c>false</c> = pass-through, both fire.</para>
    /// React with an <c>AddObserver((On&lt;ModHotkeyFired&gt; t, …) =&gt; …)</c>.
    /// </summary>
    public void Hotkey(string name, KeyCode key, bool consume = false, bool ctrl = false, bool shift = false, bool alt = false)
        => _hotkeys.Add(new Types.ModHotkeyBinding
        {
            Name = name, Key = (int)key, Ctrl = ctrl, Shift = shift, Alt = alt, Consume = consume,
        });

    /// <summary>
    /// Inspect every incoming server packet before the host parses it. Return
    /// <c>true</c> to BLOCK the packet. Runs outside the ECS — no commands.
    /// </summary>
    public void OnPacketIn(PacketFilter filter) => Filter = filter;

    /// <summary>
    /// Inspect every packet the client is about to SEND. Return <c>true</c> to BLOCK it
    /// so it never reaches the socket. Runs on the host's send path, outside the ECS —
    /// no commands.
    /// <para>A mod's own <c>ctx.Net.Send</c> bypasses this filter: a mod can neither see
    /// nor block its own sends.</para>
    /// </summary>
    public void OnPacketOut(PacketFilter filter) => FilterOut = filter;

    // ── AddSystem ────────────────────────────────────────────────────────────────
    // One overload per arity. Each parameter type describes itself once (here) and is
    // rebuilt per call from the pushed input — the query ordinals resolved during
    // Describe are captured, so nothing is re-derived at run time.

    public SystemHandle AddSystem<P1>(Action<P1> body)
        where P1 : ISystemParam<P1>
    {
        var d = Describer();
        P1.Describe(d);
        var q1 = d.Take();
        var host = _host;
        return AddSystem(d, (input, cmds) => body(
            P1.Create(new ParamContext(host, cmds, input, default, q1))));
    }

    public SystemHandle AddSystem<P1, P2>(Action<P1, P2> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
    {
        var d = Describer();
        P1.Describe(d); var q1 = d.Take();
        P2.Describe(d); var q2 = d.Take();
        var host = _host;
        return AddSystem(d, (input, cmds) => body(
            P1.Create(new ParamContext(host, cmds, input, default, q1)),
            P2.Create(new ParamContext(host, cmds, input, default, q2))));
    }

    public SystemHandle AddSystem<P1, P2, P3>(Action<P1, P2, P3> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
    {
        var d = Describer();
        P1.Describe(d); var q1 = d.Take();
        P2.Describe(d); var q2 = d.Take();
        P3.Describe(d); var q3 = d.Take();
        var host = _host;
        return AddSystem(d, (input, cmds) => body(
            P1.Create(new ParamContext(host, cmds, input, default, q1)),
            P2.Create(new ParamContext(host, cmds, input, default, q2)),
            P3.Create(new ParamContext(host, cmds, input, default, q3))));
    }

    public SystemHandle AddSystem<P1, P2, P3, P4>(Action<P1, P2, P3, P4> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
    {
        var d = Describer();
        P1.Describe(d); var q1 = d.Take();
        P2.Describe(d); var q2 = d.Take();
        P3.Describe(d); var q3 = d.Take();
        P4.Describe(d); var q4 = d.Take();
        var host = _host;
        return AddSystem(d, (input, cmds) => body(
            P1.Create(new ParamContext(host, cmds, input, default, q1)),
            P2.Create(new ParamContext(host, cmds, input, default, q2)),
            P3.Create(new ParamContext(host, cmds, input, default, q3)),
            P4.Create(new ParamContext(host, cmds, input, default, q4))));
    }

    public SystemHandle AddSystem<P1, P2, P3, P4, P5>(Action<P1, P2, P3, P4, P5> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
        where P5 : ISystemParam<P5>
    {
        var d = Describer();
        P1.Describe(d); var q1 = d.Take();
        P2.Describe(d); var q2 = d.Take();
        P3.Describe(d); var q3 = d.Take();
        P4.Describe(d); var q4 = d.Take();
        P5.Describe(d); var q5 = d.Take();
        var host = _host;
        return AddSystem(d, (input, cmds) => body(
            P1.Create(new ParamContext(host, cmds, input, default, q1)),
            P2.Create(new ParamContext(host, cmds, input, default, q2)),
            P3.Create(new ParamContext(host, cmds, input, default, q3)),
            P4.Create(new ParamContext(host, cmds, input, default, q4)),
            P5.Create(new ParamContext(host, cmds, input, default, q5))));
    }

    public SystemHandle AddSystem<P1, P2, P3, P4, P5, P6>(Action<P1, P2, P3, P4, P5, P6> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
        where P5 : ISystemParam<P5>
        where P6 : ISystemParam<P6>
    {
        var d = Describer();
        P1.Describe(d); var q1 = d.Take();
        P2.Describe(d); var q2 = d.Take();
        P3.Describe(d); var q3 = d.Take();
        P4.Describe(d); var q4 = d.Take();
        P5.Describe(d); var q5 = d.Take();
        P6.Describe(d); var q6 = d.Take();
        var host = _host;
        return AddSystem(d, (input, cmds) => body(
            P1.Create(new ParamContext(host, cmds, input, default, q1)),
            P2.Create(new ParamContext(host, cmds, input, default, q2)),
            P3.Create(new ParamContext(host, cmds, input, default, q3)),
            P4.Create(new ParamContext(host, cmds, input, default, q4)),
            P5.Create(new ParamContext(host, cmds, input, default, q5)),
            P6.Create(new ParamContext(host, cmds, input, default, q6))));
    }

    // ── AddObserver ──────────────────────────────────────────────────────────────
    // The trigger is always the first parameter, like TinyEcs.Bevy's AddObserver.

    public void AddObserver<TTrigger>(Action<TTrigger> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
    {
        var host = _host;
        AddObserver<TTrigger>((input, cmds) => body(
            TTrigger.Create(new ParamContext(host, cmds, default, input, -1))));
    }

    public void AddObserver<TTrigger, P2>(Action<TTrigger, P2> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P2 : ISystemParam<P2>
    {
        var host = _host;
        AddObserver<TTrigger>((input, cmds) => body(
            TTrigger.Create(new ParamContext(host, cmds, default, input, -1)),
            P2.Create(new ParamContext(host, cmds, default, input, -1))));
    }

    public void AddObserver<TTrigger, P2, P3>(Action<TTrigger, P2, P3> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
    {
        var host = _host;
        AddObserver<TTrigger>((input, cmds) => body(
            TTrigger.Create(new ParamContext(host, cmds, default, input, -1)),
            P2.Create(new ParamContext(host, cmds, default, input, -1)),
            P3.Create(new ParamContext(host, cmds, default, input, -1))));
    }

    public void AddObserver<TTrigger, P2, P3, P4>(Action<TTrigger, P2, P3, P4> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
    {
        var host = _host;
        AddObserver<TTrigger>((input, cmds) => body(
            TTrigger.Create(new ParamContext(host, cmds, default, input, -1)),
            P2.Create(new ParamContext(host, cmds, default, input, -1)),
            P3.Create(new ParamContext(host, cmds, default, input, -1)),
            P4.Create(new ParamContext(host, cmds, default, input, -1))));
    }

    public void AddObserver<TTrigger, P2, P3, P4, P5>(Action<TTrigger, P2, P3, P4, P5> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
        where P5 : ISystemParam<P5>
    {
        var host = _host;
        AddObserver<TTrigger>((input, cmds) => body(
            TTrigger.Create(new ParamContext(host, cmds, default, input, -1)),
            P2.Create(new ParamContext(host, cmds, default, input, -1)),
            P3.Create(new ParamContext(host, cmds, default, input, -1)),
            P4.Create(new ParamContext(host, cmds, default, input, -1)),
            P5.Create(new ParamContext(host, cmds, default, input, -1))));
    }

    // ── declaration plumbing ─────────────────────────────────────────────────────

    ParamDescriber Describer() => new(_host);

    SystemHandle AddSystem(ParamDescriber d, Action<SystemInputView, Commands> run)
    {
        var id = (uint)_sysDecls.Count;
        var decl = new SystemDecl
        {
            Id = id,
            // Diagnostics only host-side, but After/Before resolve by name — so it must
            // be unique, and .Label(..) is how a mod makes it meaningful.
            Name = $"sys-{id}",
            Schedule = ModAbi.Schedule.Update,
            Params = d.Params,
        };
        _sysDecls.Add(decl);
        SystemFns.Add(run);
        return new SystemHandle(decl);
    }

    void AddObserver<TTrigger>(Action<ObserverInputView, Commands> run)
        where TTrigger : struct, IObserverTrigger<TTrigger>
    {
        var describer = new ObserverDescriber(_host);
        TTrigger.Describe(describer);
        var decl = describer.Decl!;
        decl.Id = (uint)_obsDecls.Count;
        _obsDecls.Add(decl);
        ObserverFns.Add(run);
    }

    /// <summary>Emit the declarations the collected hotkeys imply. Called once, after the mod's Setup returned.</summary>
    internal void Finish()
    {
        if (_hotkeys.Count == 0)
            return;

        // ConsumeKeys stays false: consume is decided per binding (see Hotkey).
        var bindings = new Types.ModHotkeyBindingsDto { Bindings = _hotkeys.ToArray() };
        AddSystem((Commands cmds) => cmds.SetResource(bindings)).InStage(Stage.Startup).Label("hotkeys");
    }

    internal SetupReply BuildReply() => new()
    {
        Systems = _sysDecls.Count == 0 ? null : _sysDecls,
        Observers = _obsDecls.Count == 0 ? null : _obsDecls,
        WantsFilter = Filter != null,
        WantsFilterOut = FilterOut != null,
    };
}

/// <summary>
/// A registered system. Configure it in place — there is no <c>Build()</c>; the system
/// is live the moment <c>AddSystem</c> returned. Default stage is <see cref="Stage.Update"/>.
/// </summary>
public readonly struct SystemHandle
{
    readonly SystemDecl _decl;

    internal SystemHandle(SystemDecl decl) => _decl = decl;

    internal uint Id => _decl.Id;

    /// <summary>Run in a host stage.</summary>
    public SystemHandle InStage(Stage stage)
    {
        _decl.Schedule = (ModAbi.Schedule)(byte)stage;
        _decl.CustomStage = null;
        return this;
    }

    /// <summary>Run in a host-named custom stage.</summary>
    public SystemHandle InStage(string customStage)
    {
        _decl.Schedule = ModAbi.Schedule.Custom;
        _decl.CustomStage = customStage;
        return this;
    }

    /// <summary>Run at most once per <paramref name="ms"/> host-milliseconds. The host gates BEFORE evaluating the queries, so a throttled system is free.</summary>
    public SystemHandle Every(uint ms)
    {
        _decl.IntervalMs = ms;
        return this;
    }

    public SystemHandle After(SystemHandle system)
    {
        (_decl.After ??= new List<uint>()).Add(system.Id);
        return this;
    }

    public SystemHandle Before(SystemHandle system)
    {
        (_decl.Before ??= new List<uint>()).Add(system.Id);
        return this;
    }

    /// <summary>Name the system for host diagnostics (default <c>sys-N</c>). Must stay unique within the mod.</summary>
    public SystemHandle Label(string name)
    {
        _decl.Name = name;
        return this;
    }
}
