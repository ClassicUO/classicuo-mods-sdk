using System.Runtime.InteropServices;
using Google.FlatBuffers;
using ModAbi;

namespace CuoModSdk;

/// <summary>
/// Dispatch layer over the raw ABI — the twin of the Rust SDK's rust/src/p1/mod.rs.
///
/// Lifecycle (host drives, serially):
/// 1. <c>mod_setup(Handshake)</c> — the generated stub calls <see cref="Setup"/>: intern
///    the type-path table, run the mod's Setup, return the SetupReply.
/// 2. <c>mod_run</c> / <c>mod_observer</c> — build the parameters from the pushed data,
///    run the body, return the CommandBuffer (0 = none).
/// 3. <c>mod_spawned</c> — the host assigned real ids to the buffer's spawns.
/// 4. <c>mod_on_packet(dir, bytes)</c> — the packet handler: 0 pass, 1 block, else the
///    packed replacement.
///
/// Packed guest returns are <c>len&lt;&lt;32 | ptr</c> (0 = none).
/// </summary>
public static unsafe class ModRuntime
{
    /// <summary>
    /// The mod ABI version this SDK speaks. <see cref="Setup"/> throws when the host's
    /// Handshake.AbiVersion differs — a silent mismatch corrupts every buffer that follows.
    /// </summary>
    public const uint AbiVersion = 3;

    static List<Entry> _systems = new();
    static List<Entry> _observers = new();
    static PacketHandler? _onPacket;
    static ModHost? _host;

    // Temp ids are unique for the mod's lifetime, so a placeholder kept across runs
    // (in a field, a Local) still names exactly one spawn.
    static uint _nextTemp;
    static readonly Dictionary<uint, ulong> _resolved = new();

    internal static ModHost Host => _host ?? throw new InvalidOperationException("the mod is not set up yet");

    internal static uint NextTemp() => _nextTemp++;

    /// <summary>A placeholder's real id once the host resolved it; anything else unchanged.</summary>
    internal static ulong Resolve(ulong bits) =>
        (bits & Entity.Pending) != 0 && _resolved.TryGetValue((uint)bits, out var real) ? real : bits;

    /// <summary>Body of the generated <c>mod_setup</c> export (ModSdk.targets, from <c>&lt;CuoModType&gt;</c>).</summary>
    public static long Setup(int ptr, int len, Mod mod)
    {
        var hs = Handshake.GetRootAsHandshake(Wrap(ptr, len)).UnPack();
        if (hs.AbiVersion != AbiVersion)
            throw new InvalidOperationException(
                $"mod ABI mismatch: the client speaks v{hs.AbiVersion}, this mod was built for " +
                $"v{AbiVersion}: rebuild it against the current SDK");
        var host = new ModHost(hs);
        _host = host;
        var m = new ModBuilder(host);
        mod.Setup(m);
        m.Finish();
        _systems = m.Systems;
        _observers = m.Observers;
        _onPacket = m.PacketHandler;
        return Pack(SetupReply.Pack(Begin(), m.BuildReply()).Value);
    }

    internal static long Run(int sysId, int ptr, int len)
    {
        if ((uint)sysId >= (uint)_systems.Count)
            return 0;
        var input = new ParamsView(SystemInput.GetRootAsSystemInput(Wrap(ptr, len)));
        return Execute(_systems[sysId], input, 0, null);
    }

    internal static long Observer(int obsId, long entity, int ptr, int len)
    {
        if ((uint)obsId >= (uint)_observers.Count)
            return 0;
        var oi = ObserverInput.GetRootAsObserverInput(Wrap(ptr, len));
        var value = oi.Value is { } v ? new CompView(v) : (CompView?)null;
        return Execute(_observers[obsId], new ParamsView(oi), (ulong)entity, value);
    }

    static long Execute(Entry entry, ParamsView input, ulong triggerEntity, CompView? triggerValue)
    {
        var buffer = new CommandBufferBuilder();
        var scope = new RunScope
        {
            Host = Host,
            Commands = new Commands(buffer, Host),
            Input = input,
            Locals = entry.Locals,
            TriggerEntity = triggerEntity,
            TriggerValue = triggerValue,
        };
        entry.Run(scope);
        if (scope.WriteBacks != null)
            foreach (var wb in scope.WriteBacks)
                wb();
        scope.Commands.Flush();
        return buffer.IsEmpty ? 0L : Pack(buffer.Finish(Begin()));
    }

    internal static void Spawned(int ptr, int len)
    {
        var input = SpawnedInput.GetRootAsSpawnedInput(Wrap(ptr, len));
        for (var i = 0; i < input.SpawnedLength; i++)
        {
            var sr = input.Spawned(i)!.Value;
            _resolved[sr.TempId] = sr.Entity;
        }
    }

    internal static long OnPacket(int dir, int ptr, int len)
    {
        if (_onPacket == null)
            return 0;
        var packet = new ReadOnlySpan<byte>((void*)(nint)ptr, len).ToArray();
        var d = dir == 0 ? Packets.Direction.Incoming : Packets.Direction.Outgoing;
        return _onPacket(d, packet) switch
        {
            Packets.Verdict.Block => 1,
            Packets.Verdict.Replace r => PackBytes(r.Value),
            _ => 0,
        };
    }

    // ── FlatBuffers <-> arena plumbing ───────────────────────────────────────────
    static readonly FlatBufferBuilder _fbb = new(1024);

    // The input is copied out of the arena before any mod code runs: a mod_call result
    // lands in the arena too, and growing it may move earlier regions.
    static ByteBuffer Wrap(int ptr, int len)
    {
        var arr = new byte[len];
        new ReadOnlySpan<byte>((void*)(nint)ptr, len).CopyTo(arr);
        return new ByteBuffer(arr);
    }

    static FlatBufferBuilder Begin()
    {
        _fbb.Clear();
        return _fbb;
    }

    // Finish the root, copy the bytes into a fresh arena reservation, return len<<32 | ptr.
    static long Pack(int root)
    {
        _fbb.Finish(root);
        var bytes = _fbb.DataBuffer.ToArraySegment(_fbb.DataBuffer.Position, _fbb.Offset);
        return PackBytes(bytes);
    }

    static long PackBytes(ReadOnlySpan<byte> bytes)
    {
        var n = bytes.Length;
        var ptr = Arena.Alloc(n);
        bytes.CopyTo(new Span<byte>((void*)(nint)ptr, n));
        return ((long)n << 32) | (uint)ptr;
    }
}

// The generic ABI exports — everything except mod_setup, which must construct the mod's
// own type and is therefore GENERATED into the mod (ModSdk.targets, from <CuoModType>).
// Exported from this assembly into the mod's module via UnmanagedEntryPointsAssembly.
internal static class ModExports
{
    [UnmanagedCallersOnly(EntryPoint = "mod_alloc")]
    static int Alloc(int size) => Arena.Alloc(size);

    [UnmanagedCallersOnly(EntryPoint = "mod_arena_reset")]
    static void ArenaReset() => Arena.Reset();

    [UnmanagedCallersOnly(EntryPoint = "mod_run")]
    static long Run(int sysId, int ptr, int len) => ModRuntime.Run(sysId, ptr, len);

    [UnmanagedCallersOnly(EntryPoint = "mod_observer")]
    static long Observer(int obsId, long entity, int ptr, int len) => ModRuntime.Observer(obsId, entity, ptr, len);

    [UnmanagedCallersOnly(EntryPoint = "mod_spawned")]
    static void Spawned(int ptr, int len) => ModRuntime.Spawned(ptr, len);

    [UnmanagedCallersOnly(EntryPoint = "mod_on_packet")]
    static long OnPacket(int dir, int ptr, int len) => ModRuntime.OnPacket(dir, ptr, len);
}
