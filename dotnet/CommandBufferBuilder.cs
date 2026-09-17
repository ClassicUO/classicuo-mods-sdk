using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using ModAbi;
using Utf8 = System.Text.Encoding;

namespace CuoModSdk;

/// <summary>
/// A guest-chosen temporary id for an entity spawned earlier in the same
/// <see cref="CommandBufferBuilder"/>. Encodes to a negative wire ref (<c>-(temp_id) - 1</c>).
/// </summary>
internal readonly struct TempId
{
    internal readonly uint Raw;
    internal TempId(uint raw) => Raw = raw;
}

/// <summary>
/// An entity a command refers to: a real ecs id (implicit from <c>ulong</c>) or an
/// entity spawned earlier in this same buffer (implicit from
/// <see cref="EntityBuilder"/>, encoded as the negative wire ref <c>-(temp) - 1</c>).
/// </summary>
public readonly struct EntityRef
{
    internal readonly long Wire;
    EntityRef(long wire) => Wire = wire;

    internal static EntityRef Temp(TempId t) => new(-(long)t.Raw - 1);

    public static implicit operator EntityRef(ulong ecsId) => new((long)ecsId);
}

/// <summary>A component / resource payload to write, under an interned type id.</summary>
internal readonly struct Comp
{
    // Named Wire (not Value) so the typed Value<T>() factory below can own that name —
    // it mirrors EntityRef.Wire, the other "already-encoded" field on this seam.
    internal readonly CompValue Wire;
    Comp(CompValue value) => Wire = value;

    /// <summary>
    /// utf8 JSON payload serialized from a typed value (the wire is JSON in both
    /// directions). JsonTypeInfo overload only: source-gen metadata keeps this AOT-safe
    /// under ILC. A bare enum payload (cuo:ui/interaction) rides the same path — the
    /// generated context writes it as its number.
    /// </summary>
    public static Comp Value<T>(ushort typeId, T value, JsonTypeInfo<T> typeInfo) =>
        new(new CompValue
        {
            TypeId = typeId,
            Encoding = Encoding.Json,
            Data = JsonSerializer.SerializeToUtf8Bytes(value, typeInfo),
        });

    // No Typed() builder: Encoding.Typed (the phase-2 registry SetFlat path) is not
    // implemented host-side — the applier skips such a payload and logs it. Add the
    // builder back with the host support, not before.

    /// <summary>A zero-sized tag / marker (no payload; host reads absent data as presence).</summary>
    public static Comp Marker(ushort typeId) =>
        new(new CompValue { TypeId = typeId, Encoding = Encoding.Json, Data = null });
}

/// <summary>
/// Accumulates the structural changes a system/observer wants applied. The host applies
/// them AFTER the call returns — a component written here is NOT visible to a later
/// <c>component_get</c> read in the same call.
/// </summary>
internal sealed class CommandBufferBuilder
{
    readonly List<Cmd> _cmds = new();
    uint _nextTemp;
    // temp id -> caller-chosen name, for SpawnNamed. Moved into ModRuntime when the
    // buffer is returned to the host, then resolved to real ecs ids by mod_spawned.
    List<(uint TempId, string Name)>? _pendingNames;

    public bool IsEmpty => _cmds.Count == 0;

    /// <summary>Spawn an entity carrying <paramref name="comps"/>; returns its <see cref="TempId"/> for use as a ref in later commands.</summary>
    public TempId Spawn(params Comp[] comps)
    {
        var temp = _nextTemp++;
        _cmds.Add(new Cmd(new SpawnCmd { TempId = temp, Comps = CompValues(comps) }));
        return new TempId(temp);
    }

    /// <summary>
    /// Spawn an entity carrying <paramref name="comps"/> and remember it under
    /// <paramref name="name"/>. Once the host has applied this buffer it calls back
    /// through <c>mod_spawned</c>, after which <c>ModContext.Entity(name)</c> returns
    /// the real ecs id (across frames, until <c>ModContext.Forget</c>).
    /// </summary>
    public TempId SpawnNamed(string name, params Comp[] comps)
    {
        var temp = Spawn(comps);
        (_pendingNames ??= new List<(uint, string)>()).Add((temp.Raw, name));
        return temp;
    }

    /// <summary>Insert / overwrite components on an entity (covers component.set).</summary>
    public void Insert(EntityRef entity, params Comp[] comps) =>
        _cmds.Add(new Cmd(new InsertCmd { Entity = entity.Wire, Comps = CompValues(comps) }));

    /// <summary>Remove components (by interned type id) from an entity.</summary>
    public void Remove(EntityRef entity, params ushort[] typeIds) =>
        _cmds.Add(new Cmd(new RemoveCmd { Entity = entity.Wire, TypeIds = typeIds.Length == 0 ? null : typeIds }));

    /// <summary>Despawn an entity (and, host-side, its subtree).</summary>
    public void Despawn(EntityRef entity) =>
        _cmds.Add(new Cmd(new DespawnCmd { Entity = entity.Wire }));

    /// <summary>Parent <paramref name="child"/> under <paramref name="parent"/> at <paramref name="index"/> (default append). A child ref must follow its spawn.</summary>
    public void AddChild(EntityRef parent, EntityRef child, uint index = uint.MaxValue) =>
        _cmds.Add(new Cmd(new AddChildCmd { Parent = parent.Wire, Child = child.Wire, Index = index }));

    /// <summary>Overwrite a singleton resource.</summary>
    public void ResourceSet(Comp value) =>
        _cmds.Add(new Cmd(new ResourceSetCmd { Value = value.Wire }));

    /// <summary>Emit a custom event (utf8 JSON payload). <paramref name="entity"/> = 0 for a global event.</summary>
    public void EmitEvent(string eventName, ulong entity, string json = "") =>
        _cmds.Add(new Cmd(new EmitEventCmd
        {
            EventName = eventName,
            Entity = entity,
            Data = json.Length == 0 ? null : Utf8.UTF8.GetBytes(json),
        }));

    /// <summary>Consume a mouse button for this frame (block downstream world/pickup handling).</summary>
    public void ConsumeMouse(byte button) =>
        _cmds.Add(new Cmd(new ConsumeMouseCmd { Button = button }));

    /// <summary>Consume a key for this frame.</summary>
    public void ConsumeKey(uint key) =>
        _cmds.Add(new Cmd(new ConsumeKeyCmd { Key = key }));

    internal CommandBuffer Finish() => new() { Cmds = _cmds.Count == 0 ? null : _cmds };

    // Hand the pending SpawnNamed bindings to ModRuntime, which stashes them until the
    // host's mod_spawned callback resolves them.
    internal List<(uint TempId, string Name)>? TakePendingNames()
    {
        var names = _pendingNames;
        _pendingNames = null;
        return names;
    }

    static List<CompValue>? CompValues(Comp[] comps)
    {
        if (comps.Length == 0)
            return null;
        var list = new List<CompValue>(comps.Length);
        foreach (var c in comps)
            list.Add(c.Wire);
        return list;
    }
}
