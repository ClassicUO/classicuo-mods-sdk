namespace CuoModSdk;

/// <summary>
/// The PULL side of the host: everything a system or observer can ask for right now,
/// synchronously, without going through the command buffer. Add it to a system's
/// parameter list when you need it.
///
/// <para>Its twin is <see cref="Commands"/>, the PUSH side (spawn / insert / despawn /
/// emit / actions), which is applied after the call returns.</para>
/// </summary>
public readonly struct ModContext : ISystemParam<ModContext>
{
    readonly ModHost _host;

    internal ModContext(ModHost host) => _host = host;

    /// <summary>Nothing on the wire: every member is a mid-call host RPC.</summary>
    public static void Describe(ParamDescriber d) { }

    public static ModContext Create(ParamContext c) => new(c.Host);

    /// <summary>Host frame tick in milliseconds. Inside an observer this is the last tick the host pushed.</summary>
    public ulong Tick => _host.Tick;

    /// <summary>Append a line to the host log.</summary>
    public void Log(string message) => Imports.Log(message);

    /// <summary>Raw packet escape hatch + serial lookup. Prefer <c>Commands.Actions</c>.</summary>
    public NetApi Net => new();

    /// <summary>Gump sizes, text measurement, clilocs, parent/child walks.</summary>
    public UiApi Ui => new();

    /// <summary>This mod's persistent JSON blob.</summary>
    public StorageApi Storage => new(_host);

    /// <summary>
    /// Real ecs id of an entity spawned as <c>Commands.Spawn(name)</c>. <c>null</c> until
    /// the host resolves it (the call after the spawn), and after <see cref="Forget"/>.
    /// Nullable on purpose: entity id 0 is a valid row and comparing against it has
    /// produced phantom matches before.
    /// </summary>
    public ulong? Entity(string name) => _host.Named.TryGetValue(name, out var id) ? id : null;

    /// <summary>Drop a name binding (the entity itself is untouched — <c>Commands.Despawn(name)</c> does both).</summary>
    public void Forget(string name) => _host.Named.Remove(name);

    /// <summary>Read one component off any entity, outside the query (<c>default</c> when absent). A mid-call host RPC — prefer a query term.</summary>
    public T? Component<T>(ulong entity) => Imports.ComponentGet(entity, _host.Id<T>(), _host.Json<T>());

    /// <summary>Read a singleton host resource (<c>default</c> when absent). <see cref="Res{T}"/> is the parameter form.</summary>
    public T? Resource<T>() => Imports.ResourceGet(_host.Id<T>(), _host.Json<T>());
}

/// <summary>Network side of the host.</summary>
public readonly struct NetApi
{
    /// <summary>Send raw framed packet bytes to the server — the escape hatch for
    /// anything <see cref="ActionsApi"/> doesn't cover. You frame the bytes yourself,
    /// and the host's own client state is NOT updated.</summary>
    public void Send(ReadOnlySpan<byte> packet) => Imports.NetSend(packet);

    /// <summary>Entity id behind a UO serial (0 = not mapped).</summary>
    public ulong ResolveSerial(uint serial) => Imports.ResolveSerial(serial);
}

/// <summary>Asset-side lookups the host answers synchronously.</summary>
public readonly struct UiApi
{
    /// <summary>Gump dimensions ((0,0) when unknown).</summary>
    public (int Width, int Height) GumpSize(uint gumpId) => Imports.GumpSize(gumpId);

    /// <summary>Rendered pixel width of <paramref name="text"/> in a UO font.</summary>
    public int MeasureText(uint font, string text) => Imports.MeasureText(font, text);

    /// <summary>Cliloc string ("" when unknown).</summary>
    public string ResolveCliloc(uint id) => Imports.ResolveCliloc(id);

    /// <summary>Child entity ids, in order.</summary>
    public ulong[] Children(ulong entity) => Imports.EntityChildren(entity);

    /// <summary>Parent entity id (0 = root).</summary>
    public ulong Parent(ulong entity) => Imports.EntityParent(entity);
}

/// <summary>Chat output, through the <c>cuo:chat/message</c> event. Reached as <c>Commands.Chat</c>.</summary>
public readonly struct ChatApi
{
    readonly Commands _commands;

    internal ChatApi(Commands commands) => _commands = commands;

    // Font/unicode default to the shape a client-side line has always had: a fake
    // 0xAE unicode speech packet with font 3, which is what the server's own speech
    // arrives as. Leaving the DTO at its zero value means ascii font 0 — the big
    // gothic fonts.mul face — which is not what any other text on screen looks like.
    const byte SpeechFont = 3;

    /// <summary>A sysmessage in the journal, like the server would send.</summary>
    public void System(string text, ushort hue = 0x5B, byte font = SpeechFont, bool unicode = true) =>
        _commands.Emit(new Types.ModChatMessage
        {
            Text = text, Name = "", Hue = hue, Font = font, IsUnicode = unicode, Kind = 1,
        });

    /// <summary>Text over an entity. <paramref name="serial"/> must be a live entity the
    /// client knows — the host anchors the line by serial and drops it on the next frame
    /// otherwise, so 0 shows nothing (pass the player's own serial for a self line).</summary>
    public void Overhead(string text, ushort hue = 0x3B2, uint serial = 0, string name = "",
        byte font = SpeechFont, bool unicode = true) =>
        _commands.Emit(new Types.ModChatMessage
        {
            Text = text, Name = name, Hue = hue, Serial = serial, Font = font, IsUnicode = unicode, Kind = 0,
        });
}

/// <summary>
/// This mod's persistent blob, one JSON document at
/// <c>&lt;client&gt;/Data/Mods/&lt;mod&gt;/storage.json</c>. Write-through — call
/// <see cref="Set"/> on a user action, not every frame. Only readable once the host has
/// registered the mod, i.e. from <see cref="Stage.Startup"/> on.
/// </summary>
public readonly struct StorageApi
{
    readonly ModHost _host;

    internal StorageApi(ModHost host) => _host = host;

    /// <summary>The stored value, or <c>default</c> when nothing is stored (or it no longer parses).</summary>
    public T? Get<T>() => Storage.Get(_host.Json<T>());

    public void Set<T>(T value) => Storage.Set(value, _host.Json<T>());

    /// <summary>The raw blob ("" when unset) — for a mod that owns its own format.</summary>
    public string GetRaw() => Storage.Get();

    public void SetRaw(string json) => Storage.Set(json);
}
