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

    public static ObserverTrigger Describe(ParamDescriber d) => new(ObserverKind.OnEvent, ModHost.PathOf<T>());

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

    public static ObserverTrigger Describe(ParamDescriber d) => new(ObserverKind.OnAdd, ModHost.PathOf<T>());

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

    public static ObserverTrigger Describe(ParamDescriber d) => new(ObserverKind.OnRemove, ModHost.PathOf<T>());

    public static bool TryCreate(ParamContext c, out OnRemove<T> value) => Trigger.Create<T, OnRemove<T>>(c, static (e, v) => new OnRemove<T>(e, v), out value);
}

/// <summary>What an observer is woken by (WIT <c>trigger</c>). Opaque to a mod.</summary>
public sealed class ObserverTrigger
{
    internal readonly ObserverKind Kind;
    internal readonly string Path;
    internal readonly PacketDirection Direction;
    internal readonly byte[] Ids;

    internal ObserverTrigger(ObserverKind kind, string path, PacketDirection direction = default, byte[]? ids = null)
    {
        Kind = kind;
        Path = path;
        Direction = direction;
        Ids = ids ?? [];
    }
}

internal enum ObserverKind : byte { OnAdd, OnRemove, OnEvent, OnPacket }

static unsafe class Trigger
{
    // A payload that doesn't parse as T skips the run (a host/SDK shape drift, not mod logic).
    internal static bool Create<T, TTrigger>(ParamContext c, Func<Entity, T, TTrigger> make, out TTrigger value)
    {
        value = default!;
        var scope = c.Scope;
        T parsed;
        try
        {
            var json = new ReadOnlySpan<byte>((void*)scope.TriggerValue, scope.TriggerValueLen);
            parsed = Payload.Parse(json, c.Host.Json<T>())!;
        }
        catch (System.Text.Json.JsonException)
        {
            return false;
        }
        value = make(new Entity(scope.TriggerEntity), parsed);
        return true;
    }
}
