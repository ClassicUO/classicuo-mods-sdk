using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using Google.FlatBuffers;
using ModAbi;
using Utf8 = System.Text.Encoding;

namespace CuoModSdk;

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

    /// <summary>utf8 JSON payload (source-gen metadata keeps this AOT-safe under ILC).</summary>
    public static Comp Value<T>(ushort typeId, T value, JsonTypeInfo<T> typeInfo) =>
        new(typeId, JsonSerializer.SerializeToUtf8Bytes(value, typeInfo));

    /// <summary>Already-serialized utf8 JSON.</summary>
    public static Comp FromBytes(ushort typeId, byte[] json) => new(typeId, json);

    /// <summary>A zero-sized tag / marker (no payload; the host reads absent data as <c>{}</c>).</summary>
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
    internal long Entity;
    internal uint TempId;
    internal Comp[]? Comps;
    internal ushort[]? TypeIds;
    internal Comp Value;
    internal string? EventName;
    internal ulong EventEntity;
    internal byte[]? Data;
}

/// <summary>
/// The structural changes one system / observer run wants applied. The host applies
/// them in order AFTER the call returns.
/// </summary>
internal sealed class CommandBufferBuilder
{
    readonly List<PendingCmd> _cmds = new();

    public bool IsEmpty => _cmds.Count == 0;

    /// <summary>Spawn under a temp id the caller allocated (<see cref="ModRuntime.NextTemp"/>).</summary>
    public void Spawn(uint tempId, Comp[] comps) =>
        _cmds.Add(new PendingCmd { Type = Cmd.SpawnCmd, TempId = tempId, Comps = comps });

    public void Insert(long entity, params Comp[] comps) =>
        _cmds.Add(new PendingCmd { Type = Cmd.InsertCmd, Entity = entity, Comps = comps });

    public void Remove(long entity, params ushort[] typeIds) =>
        _cmds.Add(new PendingCmd { Type = Cmd.RemoveCmd, Entity = entity, TypeIds = typeIds });

    public void Despawn(long entity) =>
        _cmds.Add(new PendingCmd { Type = Cmd.DespawnCmd, Entity = entity });

    public void ResourceSet(Comp value) =>
        _cmds.Add(new PendingCmd { Type = Cmd.ResourceSetCmd, Value = value });

    /// <summary>An event (utf8 JSON payload) under its type path; <paramref name="entity"/> 0 = global.</summary>
    public void EmitEvent(string path, ulong entity, string json) =>
        _cmds.Add(new PendingCmd
        {
            Type = Cmd.EmitEventCmd,
            EventName = path,
            EventEntity = entity,
            Data = json.Length == 0 ? null : Utf8.UTF8.GetBytes(json),
        });

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
                Cmd.SpawnCmd => SpawnCmd.CreateSpawnCmd(b, c.TempId, CompsVector(b, c.Comps, SpawnCmd.CreateCompsVector)).Value,
                Cmd.InsertCmd => InsertCmd.CreateInsertCmd(b, c.Entity, CompsVector(b, c.Comps, InsertCmd.CreateCompsVector)).Value,
                Cmd.RemoveCmd => RemoveCmd.CreateRemoveCmd(b, c.Entity,
                    c.TypeIds is { Length: > 0 } ids ? RemoveCmd.CreateTypeIdsVectorBlock(b, ids) : default).Value,
                Cmd.DespawnCmd => DespawnCmd.CreateDespawnCmd(b, c.Entity).Value,
                Cmd.ResourceSetCmd => ResourceSetCmd.CreateResourceSetCmd(b, c.Value.Write(b)).Value,
                Cmd.EmitEventCmd => EmitEventCmd.CreateEmitEventCmd(b, b.CreateString(c.EventName), c.EventEntity,
                    c.Data == null ? default : EmitEventCmd.CreateDataVectorBlock(b, c.Data)).Value,
                _ => throw new InvalidOperationException($"unknown command {c.Type}"),
            };
        }
        var typeVec = types.Length == 0 ? default : CommandBuffer.CreateCmdsTypeVector(b, types);
        var cmdVec = offsets.Length == 0 ? default : CommandBuffer.CreateCmdsVector(b, offsets);
        return CommandBuffer.CreateCommandBuffer(b, typeVec, cmdVec).Value;
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
