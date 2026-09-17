using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;
using CuoModSdk.Types;
using ModAbi;

namespace CuoModSdk;

/// <summary>
/// The per-guest instance state the SDK threads through every callback: the interned
/// type-path table from the handshake, the JSON contexts that can resolve a payload
/// type, the name→entity bindings the host resolves for <c>Spawn(name)</c>, and the
/// last tick pushed by the host.
///
/// Instance, not statics: it is created by <see cref="ModRuntime.Setup"/> and reachable
/// only through <see cref="ModBuilder"/> / <see cref="ModContext"/>, so nothing a mod
/// writes can see (or need) global SDK state.
/// </summary>
internal sealed class ModHost
{
    readonly Dictionary<string, ushort> _typeIds = new();
    readonly List<JsonSerializerContext> _json = new();

    /// <summary>Real ecs ids of entities spawned by name, resolved through <c>mod_spawned</c>.</summary>
    internal readonly Dictionary<string, ulong> Named = new();

    /// <summary>Host frame tick from the last <c>mod_run</c> input (observers get no tick of their own).</summary>
    internal ulong Tick;

    internal ModHost(Handshake hs)
    {
        if (hs.TypePaths != null)
            foreach (var tp in hs.TypePaths)
                if (tp.Path != null)
                    _typeIds[tp.Path] = tp.Id;
    }

    internal void UseJson(JsonSerializerContext context) => _json.Add(context);

    /// <summary>
    /// Interned id for a registry type path. THROWS when the host doesn't register it —
    /// a sentinel id would make every later command on that type a silent no-op (UI that
    /// never appears, reads that never resolve), which costs far more to diagnose.
    /// </summary>
    internal ushort Id(string path) =>
        _typeIds.TryGetValue(path, out var id)
            ? id
            : throw new InvalidOperationException(
                $"the host does not register the type path '{path}' " +
                "(see src/ClassicUO.Ecs/Modding/CuoModdingRegistry.cs)");

    internal ushort Id<T>() => Id(PathOf<T>());

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
    internal JsonTypeInfo<T> Json<T>()
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
