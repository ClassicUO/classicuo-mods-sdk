using ModAbi;

namespace CuoModSdk;

/// <summary>Which way a packet travels. Values match <c>ModAbi.PacketDirection</c>.</summary>
public enum PacketDirection : byte
{
    /// <summary>Server to client.</summary>
    Incoming = 0,
    /// <summary>Client to server.</summary>
    Outgoing = 1,
}

/// <summary>
/// The packet a <see cref="ModBuilder.AddPacketObserver(PacketDirection, ReadOnlySpan{byte}, Func{Packet, Verdict})"/>
/// observer sees: the full wire bytes, id first. <see cref="Data"/> is only valid during
/// the observer run — copy it (<c>ToArray()</c>) to keep it.
/// </summary>
public readonly struct Packet
{
    readonly ArraySegment<byte> _data;

    internal Packet(PacketDirection direction, ArraySegment<byte> data)
    {
        Direction = direction;
        _data = data;
    }

    public PacketDirection Direction { get; }

    public byte Id => _data[0];

    public ReadOnlySpan<byte> Data => _data;
}

/// <summary>What a packet observer decides: <see cref="Pass"/>, <see cref="Block"/> or <see cref="Replace"/>.</summary>
public readonly struct Verdict
{
    internal readonly PacketVerdict Kind;
    internal readonly byte[]? Replacement;

    Verdict(PacketVerdict kind, byte[]? replacement)
    {
        Kind = kind;
        Replacement = replacement;
    }

    /// <summary>Let it through (as the previous mod left it).</summary>
    public static Verdict Pass => default;

    /// <summary>Drop it: the client never handles / sends it, later observers never see it.</summary>
    public static Verdict Block => new(PacketVerdict.Block, null);

    /// <summary>Forward these bytes (full wire bytes, id first) instead.</summary>
    public static Verdict Replace(byte[] packet) => new(PacketVerdict.Replace, packet);
}
