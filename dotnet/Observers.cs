namespace CuoModSdk;

/// <summary>
/// What an observer reacts to. The FIRST parameter of a <c>ModBuilder.AddObserver</c>
/// lambda must be one of these: <see cref="On{T}"/> (an event), <see cref="OnAdd{T}"/>,
/// <see cref="OnRemove{T}"/>.
/// </summary>
public interface IObserverTrigger<TSelf> where TSelf : struct, IObserverTrigger<TSelf>
{
    static abstract ObserverTrigger Describe(ParamDescriber d);

    static abstract bool TryCreate(ParamContext c, out TSelf value);
}

/// <summary>
/// The event <typeparamref name="T"/> was sent (<c>On&lt;UiClick&gt;</c>,
/// <c>On&lt;ModHotkeyFired&gt;</c>, …). Runs synchronously when the host emits it.
/// </summary>
public readonly struct On<T> : IObserverTrigger<On<T>>
{
    On(Entity entity, T @event)
    {
        Entity = entity;
        Event = @event;
    }

    /// <summary>The entity the event is aimed at (a clicked UI node); default for a global event.</summary>
    public Entity Entity { get; }

    public T Event { get; }

    public static ObserverTrigger Describe(ParamDescriber d) => new(ObserverKind.OnEvent, ModHost.PathOf<T>(), Trigger.Of<T>());

    public static bool TryCreate(ParamContext c, out On<T> value) => Trigger.Create<T, On<T>>(c, static (e, v) => new On<T>(e, v), out value);
}

/// <summary><typeparamref name="T"/> was added to an entity; <see cref="Value"/> is the new component.</summary>
public readonly struct OnAdd<T> : IObserverTrigger<OnAdd<T>>
{
    OnAdd(Entity entity, T value)
    {
        Entity = entity;
        Value = value;
    }

    public Entity Entity { get; }

    public T Value { get; }

    public static ObserverTrigger Describe(ParamDescriber d) => new(ObserverKind.OnAdd, ModHost.PathOf<T>(), Trigger.Of<T>());

    public static bool TryCreate(ParamContext c, out OnAdd<T> value) => Trigger.Create<T, OnAdd<T>>(c, static (e, v) => new OnAdd<T>(e, v), out value);
}

/// <summary><typeparamref name="T"/> was removed from an entity (or it despawned); <see cref="Value"/> is its last value.</summary>
public readonly struct OnRemove<T> : IObserverTrigger<OnRemove<T>>
{
    OnRemove(Entity entity, T value)
    {
        Entity = entity;
        Value = value;
    }

    public Entity Entity { get; }

    public T Value { get; }

    public static ObserverTrigger Describe(ParamDescriber d) => new(ObserverKind.OnRemove, ModHost.PathOf<T>(), Trigger.Of<T>());

    public static bool TryCreate(ParamContext c, out OnRemove<T> value) => Trigger.Create<T, OnRemove<T>>(c, static (e, v) => new OnRemove<T>(e, v), out value);
}

/// <summary>What an observer is woken by (WIT <c>trigger</c>). Opaque to a mod.</summary>
public sealed class ObserverTrigger
{
    internal readonly ObserverKind Kind;
    internal readonly string Path;
    internal readonly PacketDirection Direction;
    internal readonly byte[] Ids;
    // The component / event's record: the export takes `entity, value: <record>` (a tag:
    // `entity` alone). Null for a packet observer.
    internal readonly TypedTrigger? Typed;

    internal ObserverTrigger(ObserverKind kind, string path, TypedTrigger typed)
    {
        Kind = kind;
        Path = path;
        Ids = [];
        Typed = typed;
    }

    internal ObserverTrigger(PacketDirection direction, byte[] ids)
    {
        Kind = ObserverKind.OnPacket;
        Path = "";
        Direction = direction;
        Ids = ids;
    }
}

internal enum ObserverKind : byte { OnAdd, OnRemove, OnEvent, OnPacket }

static unsafe class Trigger
{
    internal static TypedTrigger Of<T>() =>
        TypedCodecs.TriggerOf<T>() ?? throw new InvalidOperationException($"{ModHost.PathOf<T>()} is not a component or event: it can't trigger an observer");

    internal static bool Create<T, TTrigger>(ParamContext c, Func<Entity, T, TTrigger> make, out TTrigger value)
    {
        var scope = c.Scope;
        // The record the export's flat params were stored back into (none for a tag).
        var parsed = scope.TriggerValue != 0 ? Typed<T>.Record.Lift((byte*)scope.TriggerValue) : default!;
        value = make(new Entity(scope.TriggerEntity), parsed);
        return true;
    }
}
