using System.Buffers;
using System.Globalization;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace CuoModSdk;

/// <summary>
/// <c>env.mod_call</c> + the JSON encoding of WIT values (docs/p1-wire.md). The generated
/// Cuo.g.cs (<see cref="Host"/>, <see cref="Assets"/>, <see cref="Actions"/>,
/// <see cref="Packets"/>) builds on these; a mod never calls them directly.
/// </summary>
internal static unsafe class Wit
{
    [DllImport("env", EntryPoint = "mod_call"), WasmImportLinkage]
    static extern long ModCall(int namePtr, int nameLen, int argsPtr, int argsLen);

    static readonly ArrayBufferWriter<byte> _args = new(256);
    static readonly Utf8JsonWriter _writer = new(_args);

    /// <summary>Starts a call: the returned writer is inside the args array.</summary>
    internal static Utf8JsonWriter Begin()
    {
        _args.ResetWrittenCount();
        _writer.Reset(_args);
        _writer.WriteStartArray();
        return _writer;
    }

    /// <summary>
    /// Calls <paramref name="name"/> with the args written since <see cref="Begin"/>; the
    /// result (an Undefined element for a function without one). The host traps on an
    /// unknown name or malformed args — a bug, not a condition to handle.
    /// </summary>
    internal static JsonElement Call(string name)
    {
        _writer.WriteEndArray();
        _writer.Flush();
        var nameBytes = NameUtf8(name);
        long packed;
        fixed (byte* n = nameBytes)
        fixed (byte* a = _args.WrittenSpan)
            packed = ModCall((int)(nint)n, nameBytes.Length, (int)(nint)a, _args.WrittenCount);
        if (packed == 0)
            return default;
        // The result lives in the arena: ParseValue copies it into the element's own
        // document before anything can grow the arena (one copy, not ToArray + Clone).
        var len = (int)((ulong)packed >> 32);
        var reader = new Utf8JsonReader(new ReadOnlySpan<byte>((void*)(nint)(uint)packed, len));
        return JsonElement.ParseValue(ref reader);
    }

    // Function names are the generated bindings' string constants: encoded once each.
    static readonly Dictionary<string, byte[]> _names = new(StringComparer.Ordinal);

    static byte[] NameUtf8(string name)
    {
        if (!_names.TryGetValue(name, out var bytes))
            _names[name] = bytes = System.Text.Encoding.UTF8.GetBytes(name);
        return bytes;
    }

    internal static Exception Bad(string type, string? value) =>
        new InvalidOperationException($"mod_call: unexpected {type} value '{value}'");

    /// <summary>Absent (no result / missing record field) or JSON null.</summary>
    internal static bool IsNull(JsonElement e) => e.ValueKind is JsonValueKind.Undefined or JsonValueKind.Null;

    /// <summary>A record field; a missing key reads as absent (an empty option).</summary>
    internal static JsonElement Field(JsonElement obj, string name) =>
        obj.ValueKind == JsonValueKind.Object && obj.TryGetProperty(name, out var v) ? v : default;

    /// <summary>The single <c>(case, payload)</c> of a variant object.</summary>
    internal static (string Case, JsonElement Payload) Case(JsonElement e)
    {
        foreach (var p in e.EnumerateObject())
            return (p.Name, p.Value);
        throw Bad("variant", e.GetRawText());
    }

    // 64-bit integers cross as decimal strings (JSON numbers lose precision past 2^53).
    internal static void WriteInt64String(Utf8JsonWriter w, ulong v) => w.WriteStringValue(v.ToString(CultureInfo.InvariantCulture));
    internal static void WriteInt64String(Utf8JsonWriter w, long v) => w.WriteStringValue(v.ToString(CultureInfo.InvariantCulture));
    internal static ulong ReadUInt64(JsonElement e) => ulong.Parse(e.GetString()!, CultureInfo.InvariantCulture);
    internal static long ReadInt64(JsonElement e) => long.Parse(e.GetString()!, CultureInfo.InvariantCulture);

    internal static System.Text.Rune ReadRune(JsonElement e) => System.Text.Rune.GetRuneAt(e.GetString()!, 0);

    internal static byte[] ReadBytes(JsonElement e) => e.GetBytesFromBase64();

    internal static T[] ReadList<T>(JsonElement e, Func<JsonElement, T> read)
    {
        var a = new T[e.GetArrayLength()];
        var i = 0;
        foreach (var x in e.EnumerateArray())
            a[i++] = read(x);
        return a;
    }
}
