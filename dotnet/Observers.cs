using ModAbi;

namespace CuoModSdk;

/// <summary>
/// What an observer reacts to. The FIRST parameter of a <c>ModBuilder.AddObserver</c>
/// lambda must be one of these: <see cref="On{T}"/> (an event), <see cref="OnAdd{T}"/>,
/// <see cref="OnRemove{T}"/>.
/// </summary>
public interface IObserverTrigger<TSelf> where TSelf : struct, IObserverTrigger<TSelf>
{
    static abstract ObserverDeclT Describe(ParamDescriber d);

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

    public static ObserverDeclT Describe(ParamDescriber d) => new()
    {
        Kind = ObserverKind.Custom,
        TypeId = ModHost.NoneType,
        EventName = ModHost.PathOf<T>(),
    };

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

    public static ObserverDeclT Describe(ParamDescriber d) => new() { Kind = ObserverKind.Insert, TypeId = d.Host.Id<T>() };

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

    public static ObserverDeclT Describe(ParamDescriber d) => new() { Kind = ObserverKind.Remove, TypeId = d.Host.Id<T>() };

    public static bool TryCreate(ParamContext c, out OnRemove<T> value) => Trigger.Create<T, OnRemove<T>>(c, static (e, v) => new OnRemove<T>(e, v), out value);
}

static class Trigger
{
    // A payload that doesn't parse as T skips the run (a host/SDK shape drift, not mod logic).
    internal static bool Create<T, TTrigger>(ParamContext c, Func<Entity, T, TTrigger> make, out TTrigger value)
    {
        value = default!;
        var payload = c.Scope.TriggerValue;
        T parsed;
        try
        {
            parsed = payload is { } p ? p.Parse(c.Host.Json<T>())! : System.Text.Json.JsonSerializer.Deserialize("{}"u8, c.Host.Json<T>())!;
        }
        catch (System.Text.Json.JsonException)
        {
            return false;
        }
        value = make(new Entity(c.Scope.TriggerEntity), parsed);
        return true;
    }
}
