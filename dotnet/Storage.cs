using System.Text.Json;

namespace CuoModSdk;

/// <summary>
/// Typed sugar over <see cref="Host.StorageGet"/> / <see cref="Host.StorageSet"/>: one JSON
/// blob per mod, global or per character. Writes go to disk immediately — save on a user
/// action, not every frame. The mod's own types need a source-generated JSON context
/// registered with <c>ModBuilder.UseJson</c>.
/// </summary>
public static class Storage
{
    /// <summary>The stored blob parsed as <typeparamref name="T"/>; <c>default</c> when nothing is stored or it doesn't parse.</summary>
    public static T? Load<T>(Host.Scope scope)
    {
        var raw = Host.StorageGet(scope);
        if (raw.Length == 0)
            return default;
        try
        {
            return JsonSerializer.Deserialize(raw, ModRuntime.Host.Json<T>());
        }
        catch (JsonException)
        {
            return default;
        }
    }

    /// <summary>Stores <paramref name="value"/> as the scope's blob.</summary>
    public static void Save<T>(Host.Scope scope, T value) =>
        Host.StorageSet(scope, JsonSerializer.Serialize(value, ModRuntime.Host.Json<T>()));
}
