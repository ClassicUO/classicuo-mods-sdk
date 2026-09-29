using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using Google.FlatBuffers;
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
    internal readonly ushort TypeId;
    internal readonly byte[]? Data;
    Comp(ushort typeId, byte[]? data)
    {
        TypeId = typeId;
        Data = data;
    }

    /// <summary>
    /// utf8 JSON payload serialized from a typed value (the wire is JSON in both
    /// directions). JsonTypeInfo overload only: source-gen metadata keeps this AOT-safe
    /// under ILC. A bare enum payload (cuo:ui/interaction) rides the same path — the
    /// generated context writes it as its number.
    /// </summary>
    public static Comp Value<T>(ushort typeId, T value, JsonTypeInfo<T> typeInfo) =>
        new(typeId, JsonSerializer.SerializeToUtf8Bytes(value, typeInfo));

    // No Typed() builder: Encoding.Typed (the phase-2 registry SetFlat path) is not
    // implemented host-side — the applier skips such a payload and logs it. Add the
    // builder back with the host support, not before.

    /// <summary>A zero-sized tag / marker (no payload; host reads absent data as presence).</summary>
    public static Comp Marker(ushort typeId) => new(typeId, null);

    internal Offset<CompValue> Write(FlatBufferBuilder b)
    {
        var data = Data == null ? default : CompValue.CreateDataVectorBlock(b, Data);
        return CompValue.CreateCompValue(b, TypeId, ModAbi.Encoding.Json, data);
    }
}

// One recorded command; Finish writes it. Only the fields its Type uses are set.
internal struct PendingCmd
{
    internal Cmd Type;
    internal long Entity, Child;
    internal uint U32;
    internal byte Button;
    internal Comp[]? Comps;
    internal ushort[]? TypeIds;
    internal Comp Value;
    internal string? EventName;
    internal ulong EventEntity;
    internal byte[]? Data;
}

/// <summary>
/// Accumulates the structural changes a system/observer wants applied. The host applies
/// them AFTER the call returns — a component written here is NOT visible to a later
/// <c>component_get</c> read in the same call.
/// </summary>
internal sealed class CommandBufferBuilder
{
    readonly List<PendingCmd> _cmds = new();
    uint _nextTemp;
    // temp id -> caller-chosen name, for SpawnNamed. Moved into ModRuntime when the
    // buffer is returned to the host, then resolved to real ecs ids by mod_spawned.
    List<(uint TempId, string Name)>? _pendingNames;

    public bool IsEmpty => _cmds.Count == 0;

    /// <summary>Spawn an entity carrying <paramref name="comps"/>; returns its <see cref="TempId"/> for use as a ref in later commands.</summary>
    public TempId Spawn(params Comp[] comps)
    {
        var temp = _nextTemp++;
        _cmds.Add(new PendingCmd { Type = Cmd.SpawnCmd, U32 = temp, Comps = comps });
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
        _cmds.Add(new PendingCmd { Type = Cmd.InsertCmd, Entity = entity.Wire, Comps = comps });

    /// <summary>Remove components (by interned type id) from an entity.</summary>
    public void Remove(EntityRef entity, params ushort[] typeIds) =>
        _cmds.Add(new PendingCmd { Type = Cmd.RemoveCmd, Entity = entity.Wire, TypeIds = typeIds });

    /// <summary>Despawn an entity (and, host-side, its subtree).</summary>
    public void Despawn(EntityRef entity) =>
        _cmds.Add(new PendingCmd { Type = Cmd.DespawnCmd, Entity = entity.Wire });

    /// <summary>Parent <paramref name="child"/> under <paramref name="parent"/> at <paramref name="index"/> (default append). A child ref must follow its spawn.</summary>
    public void AddChild(EntityRef parent, EntityRef child, uint index = uint.MaxValue) =>
        _cmds.Add(new PendingCmd { Type = Cmd.AddChildCmd, Entity = parent.Wire, Child = child.Wire, U32 = index });

    /// <summary>Overwrite a singleton resource.</summary>
    public void ResourceSet(Comp value) =>
        _cmds.Add(new PendingCmd { Type = Cmd.ResourceSetCmd, Value = value });

    /// <summary>Emit a custom event (utf8 JSON payload). <paramref name="entity"/> = 0 for a global event.</summary>
    public void EmitEvent(string eventName, ulong entity, string json = "") =>
        _cmds.Add(new PendingCmd
        {
            Type = Cmd.EmitEventCmd,
            EventName = eventName,
            EventEntity = entity,
            Data = json.Length == 0 ? null : Utf8.UTF8.GetBytes(json),
        });

    /// <summary>Consume a mouse button for this frame (block downstream world/pickup handling).</summary>
    public void ConsumeMouse(byte button) =>
        _cmds.Add(new PendingCmd { Type = Cmd.ConsumeMouseCmd, Button = button });

    /// <summary>Consume a key for this frame.</summary>
    public void ConsumeKey(uint key) =>
        _cmds.Add(new PendingCmd { Type = Cmd.ConsumeKeyCmd, U32 = key });

    /// <summary>Write the recorded commands into <paramref name="b"/>; returns the CommandBuffer root offset.</summary>
    internal int Finish(FlatBufferBuilder b)
    {
        var types = new Cmd[_cmds.Count];
        var offsets = new int[_cmds.Count];
        for (var i = 0; i < _cmds.Count; i++)
        {
            var c = _cmds[i];
            types[i] = c.Type;
            offsets[i] = c.Type switch
            {
                Cmd.SpawnCmd => SpawnCmd.CreateSpawnCmd(b, c.U32, CompsVector(b, c.Comps, SpawnCmd.CreateCompsVector)).Value,
                Cmd.InsertCmd => InsertCmd.CreateInsertCmd(b, c.Entity, CompsVector(b, c.Comps, InsertCmd.CreateCompsVector)).Value,
                Cmd.RemoveCmd => RemoveCmd.CreateRemoveCmd(b, c.Entity,
                    c.TypeIds is { Length: > 0 } ids ? RemoveCmd.CreateTypeIdsVectorBlock(b, ids) : default).Value,
                Cmd.DespawnCmd => DespawnCmd.CreateDespawnCmd(b, c.Entity).Value,
                Cmd.AddChildCmd => AddChildCmd.CreateAddChildCmd(b, c.Entity, c.Child, c.U32).Value,
                Cmd.ResourceSetCmd => ResourceSetCmd.CreateResourceSetCmd(b, c.Value.Write(b)).Value,
                Cmd.EmitEventCmd => EmitEventCmd.CreateEmitEventCmd(b, b.CreateString(c.EventName), c.EventEntity,
                    c.Data == null ? default : EmitEventCmd.CreateDataVectorBlock(b, c.Data)).Value,
                Cmd.ConsumeMouseCmd => ConsumeMouseCmd.CreateConsumeMouseCmd(b, c.Button).Value,
                Cmd.ConsumeKeyCmd => ConsumeKeyCmd.CreateConsumeKeyCmd(b, c.U32).Value,
                _ => throw new InvalidOperationException($"unknown command {c.Type}"),
            };
        }
        var typeVec = types.Length == 0 ? default : CommandBuffer.CreateCmdsTypeVector(b, types);
        var cmdVec = offsets.Length == 0 ? default : CommandBuffer.CreateCmdsVector(b, offsets);
        return CommandBuffer.CreateCommandBuffer(b, typeVec, cmdVec).Value;
    }

    // Hand the pending SpawnNamed bindings to ModRuntime, which stashes them until the
    // host's mod_spawned callback resolves them.
    internal List<(uint TempId, string Name)>? TakePendingNames()
    {
        var names = _pendingNames;
        _pendingNames = null;
        return names;
    }

    static VectorOffset CompsVector(FlatBufferBuilder b, Comp[]? comps, Func<FlatBufferBuilder, Offset<CompValue>[], VectorOffset> create)
    {
        if (comps is not { Length: > 0 })
            return default;
        var offs = new Offset<CompValue>[comps.Length];
        for (var i = 0; i < comps.Length; i++)
            offs[i] = comps[i].Write(b);
        return create(b, offs);
    }
}
