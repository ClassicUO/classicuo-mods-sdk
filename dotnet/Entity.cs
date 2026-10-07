namespace CuoModSdk;

/// <summary>
/// An entity id. An entity you just spawned gets its real id when your system returns;
/// until then it holds a placeholder (<c>1&lt;&lt;63 | temp id</c>) that commands and
/// components naming it (<see cref="Types.ChildOfDto"/>) in the same run still accept.
/// Compare / hash / store entities freely: equality and <see cref="Id"/> see the real id
/// as soon as the host assigned it.
/// </summary>
public readonly struct Entity : IEquatable<Entity>
{
    internal const ulong Pending = 1UL << 63;

    readonly ulong _bits;

    public Entity(ulong id) => _bits = id;

    /// <summary>The real ecs id (the placeholder while the spawn is still pending).</summary>
    public ulong Id => ModRuntime.Resolve(_bits);

    public static implicit operator Entity(ulong id) => new(id);

    public bool Equals(Entity other) => Id == other.Id;

    public override bool Equals(object? obj) => obj is Entity e && Equals(e);

    public override int GetHashCode() => Id.GetHashCode();

    public static bool operator ==(Entity a, Entity b) => a.Equals(b);

    public static bool operator !=(Entity a, Entity b) => !a.Equals(b);

    public override string ToString() => $"Entity({Id})";

    /// <summary>The command-buffer ref: the real id, or <c>-(temp) - 1</c> for an entity spawned earlier in this buffer.</summary>
    internal long Wire
    {
        get
        {
            var id = Id;
            return (id & Pending) != 0 ? -(long)(uint)id - 1 : (long)id;
        }
    }
}
