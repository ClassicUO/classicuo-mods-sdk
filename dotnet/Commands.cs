using System.Text.Json;

namespace CuoModSdk;

/// <summary>
/// Structural changes the host applies AFTER the callback returns — a component written
/// here is not visible to a <c>ctx.Component&lt;T&gt;()</c> read in the same call.
/// Payload types are the generated <c>CuoModSdk.Types</c> shapes; their type ids and JSON
/// metadata are resolved for you.
/// </summary>
public sealed class Commands : ISystemParam<Commands>
{
    readonly CommandBufferBuilder _buffer;
    readonly ModHost _host;
    EntityBuilder? _pending;

    internal Commands(CommandBufferBuilder buffer, ModHost host)
    {
        _buffer = buffer;
        _host = host;
    }

    /// <summary>Nothing on the wire: the SDK hands every system a command buffer anyway.</summary>
    public static void Describe(ParamDescriber d) { }

    public static Commands Create(ParamContext c) => c.Commands;

    /// <summary>Typed game actions the HOST performs (cast, target, pickup, say, …).</summary>
    public ActionsApi Actions => new(this);

    /// <summary>Print sysmessages / overhead text.</summary>
    public ChatApi Chat => new(this);

    /// <summary>
    /// Spawn an entity; chain <c>.With(..)</c> for its components. A
    /// <paramref name="name"/> makes the host hand back the real ecs id, readable from
    /// the NEXT call on as <c>ctx.Entity(name)</c>.
    /// </summary>
    public EntityBuilder Spawn(string? name = null)
    {
        Flush();
        return _pending = new EntityBuilder(this, name);
    }

    /// <summary>Insert / overwrite a component.</summary>
    public void Insert<T>(EntityRef entity, T value)
    {
        Flush();
        _buffer.Insert(entity, Payload(value));
    }

    /// <summary>Insert a zero-size marker component.</summary>
    public void Insert<T>(EntityRef entity)
    {
        Flush();
        _buffer.Insert(entity, Comp.Marker(_host.Id<T>()));
    }

    public void Remove<T>(EntityRef entity)
    {
        Flush();
        _buffer.Remove(entity, _host.Id<T>());
    }

    /// <summary>Despawn an entity and, host-side, its subtree.</summary>
    public void Despawn(EntityRef entity)
    {
        Flush();
        _buffer.Despawn(entity);
    }

    /// <summary>Despawn the entity spawned under <paramref name="name"/> and forget the name (no-op when unknown).</summary>
    public void Despawn(string name)
    {
        Flush();
        if (_host.Named.Remove(name, out var entity))
            _buffer.Despawn(entity);
    }

    /// <summary>Reparent (<see cref="EntityBuilder.ChildOf"/> is the usual way).</summary>
    public void AddChild(EntityRef parent, EntityRef child, uint index = uint.MaxValue)
    {
        Flush();
        _buffer.AddChild(parent, child, index);
    }

    /// <summary>Overwrite a singleton host resource.</summary>
    public void SetResource<T>(T value)
    {
        Flush();
        _buffer.ResourceSet(Payload(value));
    }

    /// <summary>Emit a host event; <paramref name="entity"/> 0 = global.</summary>
    public void Emit<T>(T @event, ulong entity = 0)
    {
        Flush();
        _buffer.EmitEvent(ModHost.PathOf<T>(), entity, JsonSerializer.Serialize(@event, _host.Json<T>()));
    }

    /// <summary>Swallow a mouse button for this frame (blocks downstream world / pickup handling).</summary>
    public void ConsumeMouse(byte button)
    {
        Flush();
        _buffer.ConsumeMouse(button);
    }

    /// <summary>Swallow a key for this frame.</summary>
    public void ConsumeKey(KeyCode key)
    {
        Flush();
        _buffer.ConsumeKey((uint)key);
    }

    internal ModHost Host => _host;

    internal Comp Payload<T>(T value) => Comp.Value(_host.Id<T>(), value, _host.Json<T>());

    /// <summary>Emit the open <c>Spawn(..)</c> chain, if any. Every other command — and the end of the call — does this first.</summary>
    internal void Flush()
    {
        var pending = _pending;
        if (pending == null)
            return;
        _pending = null;
        pending.Emit(_buffer);
    }

    internal void FlushIf(EntityBuilder builder)
    {
        if (_pending == builder)
            Flush();
    }
}

/// <summary>
/// An entity being spawned. The commands are emitted when the chain ends (the next
/// command, a use of this entity as a parent/ref, or the end of the callback), so the
/// spawn always precedes anything referring to it.
/// </summary>
public sealed class EntityBuilder
{
    readonly Commands _commands;
    readonly string? _name;
    readonly List<Comp> _comps = new();
    EntityRef? _parent;
    EntityRef? _self;

    internal EntityBuilder(Commands commands, string? name)
    {
        _commands = commands;
        _name = name;
    }

    /// <summary>Set a component (generated payload struct, or a bare enum like <c>Interaction</c>).</summary>
    public EntityBuilder With<T>(T value)
    {
        Open();
        _comps.Add(_commands.Payload(value));
        return this;
    }

    /// <summary>Set a zero-size marker component (<c>UiMovable</c>, <c>UiContainsByBounds</c>, …).</summary>
    public EntityBuilder With<T>()
    {
        Open();
        _comps.Add(Comp.Marker(_commands.Host.Id<T>()));
        return this;
    }

    /// <summary>Parent this entity under <paramref name="parent"/>.</summary>
    public EntityBuilder ChildOf(EntityRef parent)
    {
        Open();
        _parent = parent;
        return this;
    }

    /// <summary>Use the entity as a ref in another command (commits the chain).</summary>
    public static implicit operator EntityRef(EntityBuilder builder)
    {
        builder._commands.FlushIf(builder);
        return builder._self!.Value;
    }

    internal void Emit(CommandBufferBuilder buffer)
    {
        var comps = _comps.ToArray();
        var self = EntityRef.Temp(_name == null ? buffer.Spawn(comps) : buffer.SpawnNamed(_name, comps));
        _self = self;
        if (_parent is { } parent)
            buffer.AddChild(parent, self);
    }

    void Open()
    {
        if (_self != null)
            throw new InvalidOperationException(
                "this Spawn(..) chain was already committed — finish the .With(..) chain before issuing another command");
    }
}
