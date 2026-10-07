using System.Text.Json.Serialization;
using ModAbi;

namespace CuoModSdk;

/// <summary>An intercepted packet (full wire bytes, id first): pass it, block it, or replace it.</summary>
public delegate Packets.Verdict PacketHandler(Packets.Direction direction, ReadOnlySpan<byte> packet);

/// <summary>
/// What a mod's <see cref="Mod.Setup"/> populates: systems, observers, hotkeys and the
/// packet handler.
///
/// <para>Systems and observers are LAMBDAS whose PARAMETER TYPES declare what they need
/// — the same shape as a host <c>TinyEcs.Bevy</c> plugin (and the Rust SDK):</para>
/// <code>
/// m.AddSystem((Query&lt;Data&lt;Hits&gt;, Filter&lt;With&lt;Player&gt;, Changed&lt;Hits&gt;&gt;&gt; q, Commands cmds) =&gt; …);
/// m.AddObserver((On&lt;ModClick&gt; click, Commands cmds) =&gt; …);
/// </code>
/// <para>Parameters: <see cref="Query{TData, TFilter}"/>, <see cref="Commands"/>,
/// <see cref="Res{T}"/>, <see cref="ResMut{T}"/>, <see cref="EventReader{T}"/>,
/// <see cref="Local{T}"/>, <see cref="Opt{P}"/>. A system whose <c>Res</c> is missing is
/// skipped. Host functions are plain calls: <see cref="Host"/>, <see cref="Assets"/>,
/// <see cref="Actions"/>, <see cref="Packets"/>.</para>
/// </summary>
public sealed class ModBuilder
{
    readonly ModHost _host;
    readonly List<SystemDeclT> _sysDecls = new();
    readonly List<ObserverDeclT> _obsDecls = new();
    readonly List<Types.ModHotkeyBinding> _hotkeys = new();

    internal readonly List<Entry> Systems = new();
    internal readonly List<Entry> Observers = new();
    internal PacketHandler? PacketHandler;

    internal ModBuilder(ModHost host) => _host = host;

    /// <summary>
    /// Register a source-generated JSON context for the mod's OWN types (storage blobs,
    /// custom event payloads). Host payload types need no registration. Reflection-based
    /// STJ does not survive the wasi-wasm ILC publish, hence the context.
    /// </summary>
    public void UseJson(JsonSerializerContext context) => _host.UseJson(context);

    /// <summary>
    /// A key binding the host fires back as the <c>cuo:input/hotkey</c> event
    /// (<c>AddObserver((On&lt;ModHotkeyFired&gt; t, …) =&gt; …)</c>). All bindings are
    /// published as this mod's <c>cuo:input/mod-hotkeys</c> resource at startup.
    /// <para><paramref name="consume"/>: <c>true</c> = this mod owns the combo (the host's
    /// own binding for it does not fire), <c>false</c> = both fire.</para>
    /// </summary>
    public void Hotkey(string name, KeyCode key, bool consume = false, bool ctrl = false, bool shift = false, bool alt = false)
        => _hotkeys.Add(new Types.ModHotkeyBinding
        {
            Name = name, Key = (int)key, Ctrl = ctrl, Shift = shift, Alt = alt, Consume = consume,
        });

    /// <summary><see cref="Hotkey"/> on a mouse button (3 middle, 4/5 X1/X2, 8/9 wheel up/down).</summary>
    public void HotkeyMouse(string name, int mouseButton, bool consume = false, bool ctrl = false, bool shift = false, bool alt = false)
        => _hotkeys.Add(new Types.ModHotkeyBinding
        {
            Name = name, Mouse = mouseButton, Ctrl = ctrl, Shift = shift, Alt = alt, Consume = consume,
        });

    /// <summary>
    /// The packet handler: sees each packet whose id was passed to
    /// <see cref="Packets.Intercept"/> (call it here, in setup) before the client handles
    /// (incoming) or sends (outgoing) it. Packets this mod injects skip it.
    /// </summary>
    public void OnPacket(PacketHandler handler) => PacketHandler = handler;

    // ── AddSystem ────────────────────────────────────────────────────────────────
    // One overload per arity (all the same shape).
    // Each parameter type describes itself once (here); the slot it was given is
    // captured, so nothing is re-derived per run.

    public SystemHandle AddSystem(Action body)
    {
        var d = Describer();
        return AddSystem(d, scope =>
        {
            body();
        });
    }

    public SystemHandle AddSystem<P1>(Action<P1> body)
        where P1 : ISystemParam<P1>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        return AddSystem(d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            body(p1);
        });
    }

    public SystemHandle AddSystem<P1, P2>(Action<P1, P2> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        return AddSystem(d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            body(p1, p2);
        });
    }

    public SystemHandle AddSystem<P1, P2, P3>(Action<P1, P2, P3> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        return AddSystem(d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            body(p1, p2, p3);
        });
    }

    public SystemHandle AddSystem<P1, P2, P3, P4>(Action<P1, P2, P3, P4> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        P4.Describe(d); var s4 = d.Take();
        return AddSystem(d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            body(p1, p2, p3, p4);
        });
    }

    public SystemHandle AddSystem<P1, P2, P3, P4, P5>(Action<P1, P2, P3, P4, P5> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
        where P5 : ISystemParam<P5>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        P4.Describe(d); var s4 = d.Take();
        P5.Describe(d); var s5 = d.Take();
        return AddSystem(d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            if (!P5.TryCreate(new ParamContext(scope, s5), out var p5)) return;
            body(p1, p2, p3, p4, p5);
        });
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
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        P4.Describe(d); var s4 = d.Take();
        P5.Describe(d); var s5 = d.Take();
        P6.Describe(d); var s6 = d.Take();
        return AddSystem(d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            if (!P5.TryCreate(new ParamContext(scope, s5), out var p5)) return;
            if (!P6.TryCreate(new ParamContext(scope, s6), out var p6)) return;
            body(p1, p2, p3, p4, p5, p6);
        });
    }

    public SystemHandle AddSystem<P1, P2, P3, P4, P5, P6, P7>(Action<P1, P2, P3, P4, P5, P6, P7> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
        where P5 : ISystemParam<P5>
        where P6 : ISystemParam<P6>
        where P7 : ISystemParam<P7>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        P4.Describe(d); var s4 = d.Take();
        P5.Describe(d); var s5 = d.Take();
        P6.Describe(d); var s6 = d.Take();
        P7.Describe(d); var s7 = d.Take();
        return AddSystem(d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            if (!P5.TryCreate(new ParamContext(scope, s5), out var p5)) return;
            if (!P6.TryCreate(new ParamContext(scope, s6), out var p6)) return;
            if (!P7.TryCreate(new ParamContext(scope, s7), out var p7)) return;
            body(p1, p2, p3, p4, p5, p6, p7);
        });
    }

    public SystemHandle AddSystem<P1, P2, P3, P4, P5, P6, P7, P8>(Action<P1, P2, P3, P4, P5, P6, P7, P8> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
        where P5 : ISystemParam<P5>
        where P6 : ISystemParam<P6>
        where P7 : ISystemParam<P7>
        where P8 : ISystemParam<P8>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        P4.Describe(d); var s4 = d.Take();
        P5.Describe(d); var s5 = d.Take();
        P6.Describe(d); var s6 = d.Take();
        P7.Describe(d); var s7 = d.Take();
        P8.Describe(d); var s8 = d.Take();
        return AddSystem(d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            if (!P5.TryCreate(new ParamContext(scope, s5), out var p5)) return;
            if (!P6.TryCreate(new ParamContext(scope, s6), out var p6)) return;
            if (!P7.TryCreate(new ParamContext(scope, s7), out var p7)) return;
            if (!P8.TryCreate(new ParamContext(scope, s8), out var p8)) return;
            body(p1, p2, p3, p4, p5, p6, p7, p8);
        });
    }

    // ── AddObserver ──────────────────────────────────────────────────────────────
    // The trigger is always the first parameter, like TinyEcs.Bevy's AddObserver.

    public void AddObserver<TTrigger>(Action<TTrigger> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
    {
        var d = Describer();
        var decl = TTrigger.Describe(d);
        AddObserver(decl, d, scope =>
        {
            if (!TTrigger.TryCreate(new ParamContext(scope, -1), out var t)) return;
            body(t);
        });
    }

    public void AddObserver<TTrigger, P1>(Action<TTrigger, P1> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P1 : ISystemParam<P1>
    {
        var d = Describer();
        var decl = TTrigger.Describe(d);
        P1.Describe(d); var s1 = d.Take();
        AddObserver(decl, d, scope =>
        {
            if (!TTrigger.TryCreate(new ParamContext(scope, -1), out var t)) return;
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            body(t, p1);
        });
    }

    public void AddObserver<TTrigger, P1, P2>(Action<TTrigger, P1, P2> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
    {
        var d = Describer();
        var decl = TTrigger.Describe(d);
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        AddObserver(decl, d, scope =>
        {
            if (!TTrigger.TryCreate(new ParamContext(scope, -1), out var t)) return;
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            body(t, p1, p2);
        });
    }

    public void AddObserver<TTrigger, P1, P2, P3>(Action<TTrigger, P1, P2, P3> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
    {
        var d = Describer();
        var decl = TTrigger.Describe(d);
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        AddObserver(decl, d, scope =>
        {
            if (!TTrigger.TryCreate(new ParamContext(scope, -1), out var t)) return;
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            body(t, p1, p2, p3);
        });
    }

    public void AddObserver<TTrigger, P1, P2, P3, P4>(Action<TTrigger, P1, P2, P3, P4> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
    {
        var d = Describer();
        var decl = TTrigger.Describe(d);
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        P4.Describe(d); var s4 = d.Take();
        AddObserver(decl, d, scope =>
        {
            if (!TTrigger.TryCreate(new ParamContext(scope, -1), out var t)) return;
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            body(t, p1, p2, p3, p4);
        });
    }

    public void AddObserver<TTrigger, P1, P2, P3, P4, P5>(Action<TTrigger, P1, P2, P3, P4, P5> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
        where P5 : ISystemParam<P5>
    {
        var d = Describer();
        var decl = TTrigger.Describe(d);
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        P4.Describe(d); var s4 = d.Take();
        P5.Describe(d); var s5 = d.Take();
        AddObserver(decl, d, scope =>
        {
            if (!TTrigger.TryCreate(new ParamContext(scope, -1), out var t)) return;
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            if (!P5.TryCreate(new ParamContext(scope, s5), out var p5)) return;
            body(t, p1, p2, p3, p4, p5);
        });
    }

    public void AddObserver<TTrigger, P1, P2, P3, P4, P5, P6>(Action<TTrigger, P1, P2, P3, P4, P5, P6> body)
        where TTrigger : struct, IObserverTrigger<TTrigger>
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
        where P4 : ISystemParam<P4>
        where P5 : ISystemParam<P5>
        where P6 : ISystemParam<P6>
    {
        var d = Describer();
        var decl = TTrigger.Describe(d);
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        P4.Describe(d); var s4 = d.Take();
        P5.Describe(d); var s5 = d.Take();
        P6.Describe(d); var s6 = d.Take();
        AddObserver(decl, d, scope =>
        {
            if (!TTrigger.TryCreate(new ParamContext(scope, -1), out var t)) return;
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            if (!P5.TryCreate(new ParamContext(scope, s5), out var p5)) return;
            if (!P6.TryCreate(new ParamContext(scope, s6), out var p6)) return;
            body(t, p1, p2, p3, p4, p5, p6);
        });
    }

    // ── declaration plumbing ─────────────────────────────────────────────────────

    ParamDescriber Describer() => new(_host);

    SystemHandle AddSystem(ParamDescriber d, Action<RunScope> run)
    {
        var id = (uint)_sysDecls.Count;
        var decl = new SystemDeclT
        {
            Id = id,
            // Host diagnostics; must stay unique within the mod.
            Name = $"sys-{id}",
            Schedule = ModAbi.Schedule.Update,
            Params = d.Params,
        };
        _sysDecls.Add(decl);
        Systems.Add(new Entry(run));
        return new SystemHandle(decl);
    }

    void AddObserver(ObserverDeclT decl, ParamDescriber d, Action<RunScope> run)
    {
        decl.Id = (uint)_obsDecls.Count;
        decl.Params = d.Params;
        _obsDecls.Add(decl);
        Observers.Add(new Entry(run));
    }

    /// <summary>Declarations the collected hotkeys imply. Called once, after the mod's Setup returned.</summary>
    internal void Finish()
    {
        if (_hotkeys.Count == 0)
            return;
        var bindings = new Types.ModHotkeyBindingsDto { Bindings = _hotkeys.ToArray() };
        AddSystem((Commands cmds) => cmds.SetResource(bindings)).InStage(Stage.Startup).Label("hotkeys");
    }

    internal SetupReplyT BuildReply() => new()
    {
        Systems = _sysDecls,
        Observers = _obsDecls,
    };
}

/// <summary>A registered system / observer body and its <see cref="Local{T}"/> slots.</summary>
internal sealed class Entry(Action<RunScope> run)
{
    internal readonly Action<RunScope> Run = run;
    internal readonly List<object> Locals = new();
}

/// <summary>
/// A registered system. Configure it in place — there is no <c>Build()</c>. Default
/// stage is <see cref="Stage.Update"/>.
/// </summary>
public readonly struct SystemHandle
{
    readonly SystemDeclT _decl;

    internal SystemHandle(SystemDeclT decl) => _decl = decl;

    internal uint Id => _decl.Id;

    /// <summary>Run in a host stage.</summary>
    public SystemHandle InStage(Stage stage)
    {
        _decl.Schedule = (ModAbi.Schedule)(byte)stage;
        _decl.CustomStage = null;
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
