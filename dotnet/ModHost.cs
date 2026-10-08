using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;
using CuoModSdk.Types;

namespace CuoModSdk;

/// <summary>The per-guest state the SDK threads through every run: the JSON contexts that can resolve a payload type.</summary>
internal sealed class ModHost
{
    readonly List<JsonSerializerContext> _json = new();

    internal void UseJson(JsonSerializerContext context) => _json.Add(context);

    internal static string PathOf<T>() =>
        TypePaths.TryOf<T>(out var path)
            ? path
            : throw new InvalidOperationException(
                $"{typeof(T).Name} is not a host payload type — only CuoModSdk.Types.* types " +
                "carry a registry type-path (regenerate with `make gen-mod-sdk` if the host has it)");

    /// <summary>
    /// Source-gen JSON metadata for <typeparamref name="T"/>: the generated context for
    /// host payload types, else any context the mod handed to <c>ModBuilder.UseJson</c>.
    /// <c>GetTypeInfo(Type)</c> keeps this reflection-free under ILC.
    /// </summary>
    internal JsonTypeInfo<T> Json<T>() => JsonOf<T>.Info ??= ResolveJson<T>();

    // Resolved once per type (Json<T> runs per row per term): a component hosts one
    // ModHost for its lifetime (a reload re-instantiates it, statics and all).
    static class JsonOf<T>
    {
        internal static JsonTypeInfo<T>? Info;
    }

    JsonTypeInfo<T> ResolveJson<T>()
    {
        if (ModTypesJsonContext.Default.GetTypeInfo(typeof(T)) is JsonTypeInfo<T> generated)
            return generated;
        foreach (var context in _json)
            if (context.GetTypeInfo(typeof(T)) is JsonTypeInfo<T> own)
                return own;
        throw new InvalidOperationException(
            $"no JSON metadata for {typeof(T).FullName}: add [JsonSerializable(typeof({typeof(T).Name}))] " +
            "to a JsonSerializerContext of your own and register it with ModBuilder.UseJson(...)");
    }
}
