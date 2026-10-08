using System.Text.Json;

namespace CuoModSdk;

/// <summary>
/// Changes to the world — spawn, insert, remove, despawn, send events — applied by the
/// host in order AFTER the system returns. Payload types are the generated
/// <c>CuoModSdk.Types</c> shapes; their type paths and JSON metadata are resolved for you.
/// A spawned entity's id is real as soon as the spawn is emitted.
/// </summary>
public sealed class Commands : ISystemParam<Commands>
{
    int _handle;
    EntityBuilder? _pending;
    readonly Utf8JsonWriter _writer = new(EcsAbi.Json);

    internal Commands() { }

    public static void Describe(ParamDescriber d) => d.Add(new ParamDecl { Kind = ParamKind.Commands });

    public static bool TryCreate(ParamContext c, out Commands value)
    {
        value = c.Scope.Commands;
        return true;
    }

    internal void Begin(int handle) => _handle = handle;

    internal void End()
    {
        _pending = null;
        _handle = 0;
        EcsAbi.BundleClear();
    }

    int Handle => _handle != 0
        ? _handle
        : throw new InvalidOperationException("Commands used outside a run that declared a Commands parameter");

    /// <summary>Journal / overhead text, through the <c>cuo:chat/message</c> event.</summary>
    public ChatApi Chat => new(this);

    /// <summary>
    /// Spawn an entity; chain <c>.With(..)</c> for its components and
    /// <c>.ChildOf(parent)</c> to parent it. The builder converts to the
    /// <see cref="CuoModSdk.Entity"/> (asking for it emits the spawn).
    /// </summary>
    public EntityBuilder Spawn()
    {
        Flush();
        return _pending = new EntityBuilder(this);
    }

    /// <summary>Insert / overwrite a component.</summary>
    public void Insert<T>(Entity entity, T value)
    {
        Flush();
        AddToBundle(value);
        EcsAbi.Insert(Handle, entity.Id);
    }

    /// <summary>Insert a zero-size marker component.</summary>
    public void Insert<T>(Entity entity)
    {
        Flush();
        EcsAbi.BundleAddMarker(ModHost.PathOf<T>());
        EcsAbi.Insert(Handle, entity.Id);
    }

    public void Remove<T>(Entity entity)
    {
        Flush();
        EcsAbi.Remove(Handle, entity.Id, ModHost.PathOf<T>());
    }

    /// <summary>Despawn an entity and its children.</summary>
    public void Despawn(Entity entity)
    {
        Flush();
        EcsAbi.Despawn(Handle, entity.Id);
    }

    /// <summary>Send an event: observers of it fire, <see cref="EventReader{T}"/>s see it.</summary>
    public void Send<T>(T @event)
    {
        Flush();
        EcsAbi.BundleClear();
        var start = WriteJson(@event);
        EcsAbi.Send(Handle, ModHost.PathOf<T>(), start);
        EcsAbi.BundleClear();
    }

    /// <summary>
    /// Overwrite a resource. Declare it once in Setup with
    /// <see cref="ModBuilder.WritesResource{T}"/>; the write lands at the end of this
    /// system's stage (<see cref="ResMut{T}"/> is the parameter form, applied when the
    /// system returns).
    /// </summary>
    public void SetResource<T>(T value) => ResourceWrites.Queue(value);

    // Serializes into EcsAbi.Json; returns where the value starts.
    int WriteJson<T>(T value)
    {
        var start = EcsAbi.Json.WrittenCount;
        _writer.Reset(EcsAbi.Json);
        JsonSerializer.Serialize(_writer, value, ModRuntime.Host.Json<T>());
        _writer.Flush();
        return start;
    }

    internal void AddToBundle<T>(T value) => EcsAbi.BundleAdd(ModHost.PathOf<T>(), WriteJson(value));

    /// <summary>Mut write-back: an insert, so it lands after this run's own commands.</summary>
    internal void InsertJson(ulong entity, string path, ReadOnlySpan<byte> json)
    {
        Flush();
        var start = EcsAbi.Json.WrittenCount;
        json.CopyTo(EcsAbi.Json.GetSpan(json.Length));
        EcsAbi.Json.Advance(json.Length);
        EcsAbi.BundleAdd(path, start);
        EcsAbi.Insert(Handle, entity);
    }

    /// <summary>Emit the open <c>Spawn()</c> chain, if any. Every other command — and the end of the run — does this first.</summary>
    internal void Flush()
    {
        var pending = _pending;
        if (pending == null)
            return;
        _pending = null;
        pending.Emit(EcsAbi.Spawn(Handle));
    }

    internal bool IsPending(EntityBuilder builder) => _pending == builder;
}

/// <summary>
/// An entity being spawned. The spawn is emitted when the chain ends (the next command,
/// the end of the run, or when its <see cref="Id"/> is asked for), so its components
/// arrive in one bundle. A <c>.With</c> after that becomes an insert.
/// </summary>
public sealed class EntityBuilder
{
    readonly Commands _commands;
    ulong _id;

    internal EntityBuilder(Commands commands) => _commands = commands;

    /// <summary>The spawned entity (emits the spawn if it is still open).</summary>
    public Entity Id
    {
        get
        {
            if (_commands.IsPending(this))
                _commands.Flush();
            return new Entity(_id);
        }
    }

    /// <summary>Set a component (generated payload type, or a bare enum like <c>Interaction</c>).</summary>
    public EntityBuilder With<T>(T value)
    {
        if (_commands.IsPending(this))
            _commands.AddToBundle(value);
        else
            _commands.Insert(new Entity(_id), value);
        return this;
    }

    /// <summary>Set a zero-size marker component (<c>UiMovable</c>, <c>UiContainsByBounds</c>, …).</summary>
    public EntityBuilder With<T>()
    {
        if (_commands.IsPending(this))
            EcsAbi.BundleAddMarker(ModHost.PathOf<T>());
        else
            _commands.Insert<T>(new Entity(_id));
        return this;
    }

    /// <summary>Put this entity under <paramref name="parent"/> (<c>cuo:ecs/child-of</c>).</summary>
    public EntityBuilder ChildOf(Entity parent) => With(new Types.ChildOfDto { Parent = parent.Id });

    public static implicit operator Entity(EntityBuilder builder) => builder.Id;

    internal void Emit(ulong id) => _id = id;
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

/// <summary>The <see cref="Commands.SetResource{T}"/> queue and the SDK systems that apply it.</summary>
internal static class ResourceWrites
{
    /// <summary>The declared writable resources, in the flush systems' param order.</summary>
    internal static readonly List<string> Paths = new();
    static readonly List<(int Slot, byte[] Json)> _queue = new();

    internal static void Queue<T>(T value)
    {
        var path = ModHost.PathOf<T>();
        var slot = Paths.IndexOf(path);
        if (slot < 0)
            throw new InvalidOperationException(
                $"SetResource<{typeof(T).Name}>: declare m.WritesResource<{typeof(T).Name}>() in Setup " +
                $"('{path}' must be held writable by the SDK's flush systems)");
        _queue.Add((slot, JsonSerializer.SerializeToUtf8Bytes(value, ModRuntime.Host.Json<T>())));
    }

    internal static void Flush(RunScope scope)
    {
        foreach (var (slot, json) in _queue)
            EcsAbi.ResSet(scope.Handle(slot), json);
        _queue.Clear();
    }
}
