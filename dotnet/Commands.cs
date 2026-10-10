namespace CuoModSdk;

/// <summary>
/// Changes to the world — spawn, insert, remove, despawn, send events — applied by the
/// host in order AFTER the system returns. Payload types are the generated
/// <c>CuoModSdk.Types</c> shapes; each crosses as its typed record. A spawned entity's id
/// is real as soon as the spawn is emitted.
/// </summary>
public sealed unsafe class Commands : ISystemParam<Commands>
{
    int _handle;
    EntityBuilder? _pending;

    // The open spawn's components, lowered (records back to back in _recs), replayed
    // through the entity builder when the chain closes.
    readonly List<(TypedCodec Codec, int Offset)> _ops = new();
    byte[] _recs = new byte[256];
    int _recLen;

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
        ResetSpawn();
    }

    void ResetSpawn()
    {
        _ops.Clear();
        _recLen = 0;
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
        ResetSpawn();
        return _pending = new EntityBuilder(this);
    }

    /// <summary>Insert / overwrite a component (a tag's value is ignored).</summary>
    public void Insert<T>(Entity entity, T value)
    {
        Flush();
        if (Typed<T>.IsTag)
        {
            TypedAbi.Insert(Handle, entity.Id, TagOf<T>(), null);
            return;
        }
        var codec = Typed<T>.Record;
        var rec = stackalloc byte[codec.Size];
        codec.LowerInto(value, rec);
        TypedAbi.Insert(Handle, entity.Id, codec, rec);
    }

    /// <summary>Insert a zero-size marker component.</summary>
    public void Insert<T>(Entity entity)
    {
        Flush();
        TypedAbi.Insert(Handle, entity.Id, TagOf<T>(), null);
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
        (Typed<T>.Event ?? throw new InvalidOperationException($"{ModHost.PathOf<T>()} is not an event")).Send(Handle, @event);
    }

    /// <summary>
    /// Overwrite a writable host resource, applied with this run's other commands. No
    /// declaration needed (<see cref="ResMut{T}"/> is the parameter form when the system
    /// also reads it).
    /// </summary>
    public void SetResource<T>(T value)
    {
        Flush();
        var codec = Typed<T>.Record;
        var rec = stackalloc byte[codec.Size];
        codec.LowerInto(value, rec);
        codec.SetResource(Handle, rec);
    }

    static TagCodec TagOf<T>() =>
        Typed<T>.Tag ?? throw new InvalidOperationException(
            Typed<T>.IsTag
                ? $"{ModHost.PathOf<T>()} can't be inserted by a mod (a host-owned marker)"
                : $"{ModHost.PathOf<T>()} is not a tag: insert it with a value");

    /// <summary>A component of the open spawn.</summary>
    internal void AddToSpawn<T>(T value)
    {
        if (Typed<T>.IsTag)
        {
            _ops.Add((TagOf<T>(), 0));
            return;
        }
        var codec = Typed<T>.Record;
        int offset;
        fixed (byte* rec = Reserve(codec.Size, out offset))
            codec.LowerInto(value, rec);
        _ops.Add((codec, offset));
    }

    /// <summary>A zero-size marker of the open spawn.</summary>
    internal void AddMarkerToSpawn<T>() => _ops.Add((TagOf<T>(), 0));

    Span<byte> Reserve(int size, out int offset)
    {
        // Records hold f32 / u64 fields: keep every one 8-aligned within the buffer.
        offset = (_recLen + 7) & ~7;
        if (_recs.Length < offset + size)
            Array.Resize(ref _recs, Math.Max(offset + size, _recs.Length * 2));
        _recLen = offset + size;
        return _recs.AsSpan(offset, size);
    }

    ulong SpawnBuilt()
    {
        var b = TypedAbi.Spawn(Handle);
        var id = TypedAbi.Id(b);
        fixed (byte* recs = _recs)
            foreach (var (codec, offset) in _ops)
            {
                var next = codec.Build(b, recs + offset);
                TypedAbi.Drop(b);
                b = next;
            }
        TypedAbi.Drop(b);
        return id;
    }

    /// <summary>Emit the open <c>Spawn()</c> chain, if any. Every other command — and the end of the run — does this first.</summary>
    internal void Flush()
    {
        var pending = _pending;
        if (pending == null)
            return;
        _pending = null;
        var id = SpawnBuilt();
        ResetSpawn();
        pending.Emit(id);
    }

    internal bool IsPending(EntityBuilder builder) => _pending == builder;
}

/// <summary>
/// An entity being spawned. The spawn is emitted when the chain ends (the next command,
/// the end of the run, or when its <see cref="Id"/> is asked for), so its components
/// arrive together. A <c>.With</c> after that becomes an insert.
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
            _commands.AddToSpawn(value);
        else
            _commands.Insert(new Entity(_id), value);
        return this;
    }

    /// <summary>Set a zero-size marker component (<c>UiMovable</c>, <c>UiContainsByBounds</c>, …).</summary>
    public EntityBuilder With<T>()
    {
        if (_commands.IsPending(this))
            _commands.AddMarkerToSpawn<T>();
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
