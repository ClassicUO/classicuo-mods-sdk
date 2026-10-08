using System.Text.Json.Serialization;
using Ecs = ModWorld.wit.Imports.tinyecs.modding.v0_1_0.IEcsImports;

namespace CuoModSdk;

/// <summary>
/// What a mod's <see cref="Mod.Setup"/> populates: systems, observers (packet observers
/// included) and hotkeys.
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
    readonly List<Types.ModHotkeyBinding> _hotkeys = new();

    /// <summary>Systems and observers, in declaration order.</summary>
    internal readonly List<Entry> Entries = new();
    int _systems, _observers;

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

    // ── AddPacketObserver ────────────────────────────────────────────────────────
    // The tinyecs-mod `on-packet` trigger: runs synchronously before the client handles
    // (incoming) or sends (outgoing) a packet whose id is in `ids` (empty = every id).
    // Mods run in load order, a mod's packet observers in declaration order; each sees
    // the previous replacement, Block stops the chain. Packets this mod sent skip them.
    // A run skipped for a missing parameter passes.

    public void AddPacketObserver(PacketDirection direction, ReadOnlySpan<byte> ids, Func<Packet, Verdict> body)
    {
        var d = Describer();
        AddObserver(PacketDecl(direction, ids), d, scope =>
        {
            scope.Verdict = body(scope.Packet);
        });
    }

    public void AddPacketObserver<P1>(PacketDirection direction, ReadOnlySpan<byte> ids, Func<Packet, P1, Verdict> body)
        where P1 : ISystemParam<P1>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        AddObserver(PacketDecl(direction, ids), d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            scope.Verdict = body(scope.Packet, p1);
        });
    }

    public void AddPacketObserver<P1, P2>(PacketDirection direction, ReadOnlySpan<byte> ids, Func<Packet, P1, P2, Verdict> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        AddObserver(PacketDecl(direction, ids), d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            scope.Verdict = body(scope.Packet, p1, p2);
        });
    }

    public void AddPacketObserver<P1, P2, P3>(PacketDirection direction, ReadOnlySpan<byte> ids, Func<Packet, P1, P2, P3, Verdict> body)
        where P1 : ISystemParam<P1>
        where P2 : ISystemParam<P2>
        where P3 : ISystemParam<P3>
    {
        var d = Describer();
        P1.Describe(d); var s1 = d.Take();
        P2.Describe(d); var s2 = d.Take();
        P3.Describe(d); var s3 = d.Take();
        AddObserver(PacketDecl(direction, ids), d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            scope.Verdict = body(scope.Packet, p1, p2, p3);
        });
    }

    public void AddPacketObserver<P1, P2, P3, P4>(PacketDirection direction, ReadOnlySpan<byte> ids, Func<Packet, P1, P2, P3, P4, Verdict> body)
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
        AddObserver(PacketDecl(direction, ids), d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            scope.Verdict = body(scope.Packet, p1, p2, p3, p4);
        });
    }

    public void AddPacketObserver<P1, P2, P3, P4, P5>(PacketDirection direction, ReadOnlySpan<byte> ids, Func<Packet, P1, P2, P3, P4, P5, Verdict> body)
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
        AddObserver(PacketDecl(direction, ids), d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            if (!P5.TryCreate(new ParamContext(scope, s5), out var p5)) return;
            scope.Verdict = body(scope.Packet, p1, p2, p3, p4, p5);
        });
    }

    public void AddPacketObserver<P1, P2, P3, P4, P5, P6>(PacketDirection direction, ReadOnlySpan<byte> ids, Func<Packet, P1, P2, P3, P4, P5, P6, Verdict> body)
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
        AddObserver(PacketDecl(direction, ids), d, scope =>
        {
            if (!P1.TryCreate(new ParamContext(scope, s1), out var p1)) return;
            if (!P2.TryCreate(new ParamContext(scope, s2), out var p2)) return;
            if (!P3.TryCreate(new ParamContext(scope, s3), out var p3)) return;
            if (!P4.TryCreate(new ParamContext(scope, s4), out var p4)) return;
            if (!P5.TryCreate(new ParamContext(scope, s5), out var p5)) return;
            if (!P6.TryCreate(new ParamContext(scope, s6), out var p6)) return;
            scope.Verdict = body(scope.Packet, p1, p2, p3, p4, p5, p6);
        });
    }

    // ── declaration plumbing ─────────────────────────────────────────────────────

    ParamDescriber Describer() => new(_host);

    static ObserverTrigger PacketDecl(PacketDirection direction, ReadOnlySpan<byte> ids) =>
        new(ObserverKind.OnPacket, "", direction, ids.ToArray());

    SystemHandle AddSystem(ParamDescriber d, Action<RunScope> run)
    {
        // Host diagnostics + the name `run` is called with; must stay unique within the mod.
        var entry = new Entry(run, $"sys-{_systems++}", d.Params) { Schedule = Stage.Update };
        Entries.Add(entry);
        return new SystemHandle(entry);
    }

    void AddObserver(ObserverTrigger trigger, ParamDescriber d, Action<RunScope> run) =>
        Entries.Add(new Entry(run, $"obs-{_observers++}", d.Params) { Trigger = trigger });

    /// <summary>
    /// Declares that this mod writes resource <typeparamref name="T"/> with
    /// <see cref="Commands.SetResource{T}"/>. The component contract has no
    /// resource-set command: the SDK queues those writes and applies them through its own
    /// systems (one per stage the mod runs in, after the mod's systems there, plus
    /// <see cref="Stage.Last"/>) that hold the declared resources writable. A write from
    /// a system lands at the end of that system's stage; from an observer, at the next
    /// such point.
    /// </summary>
    public void WritesResource<T>()
    {
        var path = ModHost.PathOf<T>();
        if (!ResourceWrites.Paths.Contains(path))
            ResourceWrites.Paths.Add(path);
    }

    /// <summary>Declarations the collected hotkeys and resource writes imply. Called once, after the mod's Setup returned.</summary>
    internal void Finish()
    {
        // Only the mod's own SetResource calls need a flush in every stage; the hotkey
        // set is written once, by the Startup flush.
        var everyStage = ResourceWrites.Paths.Count > 0;
        if (_hotkeys.Count > 0)
        {
            // The binding set is this mod's slice of cuo:input/mod-hotkeys, written by the
            // Startup flush whether or not the host has a value yet.
            WritesResource<Types.ModHotkeyBindingsDto>();
            ResourceWrites.Queue(new Types.ModHotkeyBindingsDto { Bindings = _hotkeys.ToArray() });
        }
        var paths = ResourceWrites.Paths;
        if (paths.Count == 0)
            return;
        var mods = Entries.ToArray();
        for (var stage = Stage.Startup; stage <= Stage.Last; stage++)
        {
            var here = Array.FindAll(mods, e => e.Trigger == null && e.Schedule == stage);
            if (stage != Stage.Startup && (!everyStage || (here.Length == 0 && stage != Stage.Last)))
                continue;
            var d = Describer();
            foreach (var path in paths)
                d.Add(new ParamDecl { Kind = ParamKind.ResMut, Path = path });
            var flush = AddSystem(d, ResourceWrites.Flush).InStage(stage).Label($"sdk-resource-writes-{stage}");
            foreach (var e in here)
                flush.After(new SystemHandle(e));
        }
    }

    /// <summary>Declares every entry on the host (called once, after <see cref="Finish"/>).</summary>
    internal void Declare(Ecs.App app)
    {
        // Every handle first: `after` / `before` borrow the other system.
        var systems = new Ecs.System[Entries.Count];
        for (var i = 0; i < systems.Length; i++)
            systems[i] = new Ecs.System(Entries[i].Name);
        try
        {
            for (var i = 0; i < systems.Length; i++)
            {
                var e = Entries[i];
                var sys = systems[i];
                foreach (var other in e.After)
                    sys.After(systems[Entries.IndexOf(other)]);
                foreach (var other in e.Before)
                    sys.Before(systems[Entries.IndexOf(other)]);
                foreach (var p in e.Params)
                {
                    switch (p.Kind)
                    {
                        case ParamKind.Commands: sys.AddCommands(); break;
                        case ParamKind.Query: sys.AddQuery(p.Terms!.ConvertAll(Term)); break;
                        case ParamKind.Res: sys.AddRes(p.Path); break;
                        case ParamKind.ResMut: sys.AddResMut(p.Path); break;
                        case ParamKind.Events: sys.AddEvents(p.Path); break;
                    }
                }
                if (e.Trigger is { } t)
                    app.AddObserver(Trigger(t), sys);
                else
                    app.AddSystems((Ecs.Schedule)(byte)e.Schedule, [sys]);
            }
        }
        finally
        {
            foreach (var sys in systems)
                sys.Dispose();
        }
    }

    static Ecs.Term Term(QueryTerm t) => t.Kind switch
    {
        TermKind.Ref => Ecs.Term.@ref(t.Path),
        TermKind.Mut => Ecs.Term.Mut(t.Path),
        TermKind.With => Ecs.Term.With(t.Path),
        TermKind.Without => Ecs.Term.Without(t.Path),
        TermKind.Changed => Ecs.Term.Changed(t.Path),
        _ => Ecs.Term.Added(t.Path),
    };

    static Ecs.Trigger Trigger(ObserverTrigger t) => t.Kind switch
    {
        ObserverKind.OnAdd => Ecs.Trigger.OnAdd(t.Path),
        ObserverKind.OnRemove => Ecs.Trigger.OnRemove(t.Path),
        ObserverKind.OnEvent => Ecs.Trigger.OnEvent(t.Path),
        _ => Ecs.Trigger.OnPacket(new Ecs.PacketFilter((Ecs.PacketDirection)(byte)t.Direction, t.Ids)),
    };
}

/// <summary>A registered system / observer: its body, declaration and <see cref="Local{T}"/> slots.</summary>
internal sealed class Entry(Action<RunScope> run, string name, List<ParamDecl> @params)
{
    internal readonly Action<RunScope> Run = run;
    internal readonly List<object> Locals = new();
    internal readonly List<ParamDecl> Params = @params;
    internal string Name = name;
    internal byte[] NameUtf8 = System.Text.Encoding.UTF8.GetBytes(name);
    internal Stage Schedule;
    internal ObserverTrigger? Trigger;
    internal readonly List<Entry> After = new();
    internal readonly List<Entry> Before = new();
}

/// <summary>
/// A registered system. Configure it in place — there is no <c>Build()</c>. Default
/// stage is <see cref="Stage.Update"/>.
/// </summary>
public readonly struct SystemHandle
{
    readonly Entry _entry;

    internal SystemHandle(Entry entry) => _entry = entry;

    /// <summary>Run in a host stage.</summary>
    public SystemHandle InStage(Stage stage)
    {
        _entry.Schedule = stage;
        return this;
    }

    public SystemHandle After(SystemHandle system)
    {
        _entry.After.Add(system._entry);
        return this;
    }

    public SystemHandle Before(SystemHandle system)
    {
        _entry.Before.Add(system._entry);
        return this;
    }

    /// <summary>Name the system for host diagnostics (default <c>sys-N</c>). Must stay unique within the mod.</summary>
    public SystemHandle Label(string name)
    {
        _entry.Name = name;
        _entry.NameUtf8 = System.Text.Encoding.UTF8.GetBytes(name);
        return this;
    }
}
