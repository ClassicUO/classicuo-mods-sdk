using System.Text.Json;
using ModAbi;

namespace CuoModSdk;

/// <summary>
/// Changes to the world — spawn, insert, remove, despawn, send events — applied by the
/// host in order AFTER the system returns. Payload types are the generated
/// <c>CuoModSdk.Types</c> shapes; their type ids and JSON metadata are resolved for you.
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

    public static void Describe(ParamDescriber d) => d.Add(new ParamDeclT { Kind = ParamKind.Commands, TypeId = ModHost.NoneType });

    public static bool TryCreate(ParamContext c, out Commands value)
    {
        value = c.Scope.Commands;
        return true;
    }

    /// <summary>Journal / overhead text, through the <c>cuo:chat/message</c> event.</summary>
    public ChatApi Chat => new(this);

    /// <summary>
    /// Spawn an entity; chain <c>.With(..)</c> for its components and
    /// <c>.ChildOf(parent)</c> to parent it. The builder converts to the
    /// <see cref="CuoModSdk.Entity"/> (a placeholder until the host assigned the real id).
    /// </summary>
    public EntityBuilder Spawn()
    {
        Flush();
        return _pending = new EntityBuilder(this, ModRuntime.NextTemp());
    }

    /// <summary>Insert / overwrite a component.</summary>
    public void Insert<T>(Entity entity, T value)
    {
        Flush();
        _buffer.Insert(entity.Wire, Payload(value));
    }

    /// <summary>Insert a zero-size marker component.</summary>
    public void Insert<T>(Entity entity)
    {
        Flush();
        _buffer.Insert(entity.Wire, Comp.Marker(_host.Id<T>()));
    }

    public void Remove<T>(Entity entity)
    {
        Flush();
        _buffer.Remove(entity.Wire, _host.Id<T>());
    }

    /// <summary>Despawn an entity and its children.</summary>
    public void Despawn(Entity entity)
    {
        Flush();
        _buffer.Despawn(entity.Wire);
    }

    /// <summary>Send an event: observers of it fire, <see cref="EventReader{T}"/>s see it. <paramref name="target"/> aims it at an entity.</summary>
    public void Send<T>(T @event, Entity target = default)
    {
        Flush();
        _buffer.EmitEvent(ModHost.PathOf<T>(), target.Id, JsonSerializer.Serialize(@event, _host.Json<T>()));
    }

    /// <summary>Overwrite a resource (<see cref="ResMut{T}"/> is the parameter form).</summary>
    public void SetResource<T>(T value)
    {
        Flush();
        _buffer.ResourceSet(Payload(value));
    }

    internal ModHost Host => _host;

    internal Comp Payload<T>(T value) => Comp.Value(_host.Id<T>(), value, _host.Json<T>());

    internal void ResourceSetRaw(ushort typeId, byte[] json)
    {
        Flush();
        _buffer.ResourceSet(Comp.FromBytes(typeId, json));
    }

    internal void InsertRaw(Entity entity, Comp comp)
    {
        Flush();
        _buffer.Insert(entity.Wire, comp);
    }

    /// <summary>Emit the open <c>Spawn()</c> chain, if any. Every other command — and the end of the run — does this first.</summary>
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
/// An entity being spawned. Its SpawnCmd is emitted when the chain ends (the next
/// command, or the end of the run), so it always precedes anything referring to it.
/// A <c>.With</c> after that becomes an insert.
/// </summary>
public sealed class EntityBuilder
{
    readonly Commands _commands;
    readonly uint _temp;
    readonly List<Comp> _comps = new();
    bool _emitted;

    internal EntityBuilder(Commands commands, uint temp)
    {
        _commands = commands;
        _temp = temp;
    }

    /// <summary>The spawned entity (a placeholder until the host assigned the real id).</summary>
    public Entity Id => new(Entity.Pending | _temp);

    /// <summary>Set a component (generated payload type, or a bare enum like <c>Interaction</c>).</summary>
    public EntityBuilder With<T>(T value)
    {
        var comp = _commands.Payload(value);
        if (_emitted)
            _commands.InsertRaw(Id, comp);
        else
            _comps.Add(comp);
        return this;
    }

    /// <summary>Set a zero-size marker component (<c>UiMovable</c>, <c>UiContainsByBounds</c>, …).</summary>
    public EntityBuilder With<T>()
    {
        var comp = Comp.Marker(_commands.Host.Id<T>());
        if (_emitted)
            _commands.InsertRaw(Id, comp);
        else
            _comps.Add(comp);
        return this;
    }

    /// <summary>Put this entity under <paramref name="parent"/> (<c>cuo:ecs/child-of</c>).</summary>
    public EntityBuilder ChildOf(Entity parent) => With(new Types.ChildOfDto { Parent = parent.Id });

    public static implicit operator Entity(EntityBuilder builder) => builder.Id;

    internal void Emit(CommandBufferBuilder buffer)
    {
        _emitted = true;
        buffer.Spawn(_temp, _comps.ToArray());
    }
}

/// <summary>Chat output, through the <c>cuo:chat/message</c> event. Reached as <c>Commands.Chat</c>.</summary>
public readonly struct ChatApi
{
    readonly Commands _commands;

    internal ChatApi(Commands commands) => _commands = commands;

    // Font 3 unicode: the shape server speech arrives in. The DTO's zero value (ascii
    // font 0) is the big gothic face, unlike any other text on screen.
    const byte SpeechFont = 3;

    /// <summary>A sysmessage in the journal, like the server would send.</summary>
    public void System(string text, ushort hue = 0x5B, byte font = SpeechFont, bool unicode = true) =>
        _commands.Send(new Types.ModChatMessage
        {
            Text = text, Name = "", Hue = hue, Font = font, IsUnicode = unicode, Kind = 1,
        });

    /// <summary>Text over a live entity the client knows (0 shows nothing; pass the player's own serial for a self line).</summary>
    public void Overhead(string text, ushort hue = 0x3B2, uint serial = 0, string name = "",
        byte font = SpeechFont, bool unicode = true) =>
        _commands.Send(new Types.ModChatMessage
        {
            Text = text, Name = name, Hue = hue, Serial = serial, Font = font, IsUnicode = unicode, Kind = 0,
        });
}
