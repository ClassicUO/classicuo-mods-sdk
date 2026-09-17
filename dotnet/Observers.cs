using ModAbi;

namespace CuoModSdk;

/// <summary>
/// What an observer reacts to. The FIRST parameter of a <c>ModBuilder.AddObserver</c>
/// lambda must be one of these — the same shape as a host <c>TinyEcs.Bevy</c> observer.
/// </summary>
public interface IObserverTrigger<TSelf> where TSelf : struct, IObserverTrigger<TSelf>
{
    static abstract void Describe(ObserverDescriber d);

    static abstract TSelf Create(ParamContext c);
}

/// <summary>Collects the wire declaration of one observer as its trigger type describes itself.</summary>
public sealed class ObserverDescriber
{
    const ushort NoneType = 0xFFFF;

    readonly ModHost _host;
    internal ObserverDecl? Decl;

    internal ObserverDescriber(ModHost host) => _host = host;

    internal void Event<T>() => Decl = new ObserverDecl
    {
        Kind = ObserverKind.Custom,
        TypeId = NoneType,
        EventName = ModHost.PathOf<T>(),
    };

    internal void Insert<T>() => Decl = new ObserverDecl { Kind = ObserverKind.Insert, TypeId = _host.Id<T>() };

    internal void Remove<T>() => Decl = new ObserverDecl { Kind = ObserverKind.Remove, TypeId = _host.Id<T>() };

    internal void Spawn() => Decl = new ObserverDecl { Kind = ObserverKind.Spawn, TypeId = NoneType };

    internal void Despawn() => Decl = new ObserverDecl { Kind = ObserverKind.Despawn, TypeId = NoneType };
}

/// <summary>
/// A host event fired (<typeparamref name="T"/> = the generated event payload type).
/// Runs synchronously when the host emits it — no per-frame polling, no one-frame lag.
/// </summary>
public readonly struct On<T> : IObserverTrigger<On<T>>
{
    On(ulong entity, T @event)
    {
        Entity = entity;
        Event = @event;
    }

    /// <summary>The entity the event was emitted on (0 = global).</summary>
    public ulong Entity { get; }

    public T Event { get; }

    public static void Describe(ObserverDescriber d) => d.Event<T>();

    public static On<T> Create(ParamContext c) => new(c.Observer.Entity, c.Payload<T>());
}

/// <summary><typeparamref name="T"/> was inserted on an entity; <see cref="Value"/> is the new component.</summary>
public readonly struct OnInsert<T> : IObserverTrigger<OnInsert<T>>
{
    OnInsert(ulong entity, T value)
    {
        Entity = entity;
        Value = value;
    }

    public ulong Entity { get; }

    public T Value { get; }

    public static void Describe(ObserverDescriber d) => d.Insert<T>();

    public static OnInsert<T> Create(ParamContext c) => new(c.Observer.Entity, c.Payload<T>());
}

/// <summary><typeparamref name="T"/> was removed from an entity; <see cref="Value"/> is its last value.</summary>
public readonly struct OnRemove<T> : IObserverTrigger<OnRemove<T>>
{
    OnRemove(ulong entity, T value)
    {
        Entity = entity;
        Value = value;
    }

    public ulong Entity { get; }

    public T Value { get; }

    public static void Describe(ObserverDescriber d) => d.Remove<T>();

    public static OnRemove<T> Create(ParamContext c) => new(c.Observer.Entity, c.Payload<T>());
}

/// <summary>An entity was spawned.</summary>
public readonly struct OnSpawn : IObserverTrigger<OnSpawn>
{
    OnSpawn(ulong entity) => Entity = entity;

    public ulong Entity { get; }

    public static void Describe(ObserverDescriber d) => d.Spawn();

    public static OnSpawn Create(ParamContext c) => new(c.Observer.Entity);
}

/// <summary>An entity was despawned.</summary>
public readonly struct OnDespawn : IObserverTrigger<OnDespawn>
{
    OnDespawn(ulong entity) => Entity = entity;

    public ulong Entity { get; }

    public static void Describe(ObserverDescriber d) => d.Despawn();

    public static OnDespawn Create(ParamContext c) => new(c.Observer.Entity);
}
