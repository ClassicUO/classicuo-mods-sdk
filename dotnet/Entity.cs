namespace CuoModSdk;

/// <summary>
/// An entity id. An entity you spawn has its real id as soon as the spawn is emitted
/// (<c>commands.spawn</c> returns it), so compare / hash / store entities freely.
/// </summary>
public readonly struct Entity : IEquatable<Entity>
{
    readonly ulong _id;

    public Entity(ulong id) => _id = id;

    public ulong Id => _id;

    public static implicit operator Entity(ulong id) => new(id);

    public bool Equals(Entity other) => _id == other._id;

    public override bool Equals(object? obj) => obj is Entity e && Equals(e);

    public override int GetHashCode() => _id.GetHashCode();

    public static bool operator ==(Entity a, Entity b) => a.Equals(b);

    public static bool operator !=(Entity a, Entity b) => !a.Equals(b);

    public override string ToString() => $"Entity({_id})";
}
