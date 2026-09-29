using System.Runtime.InteropServices;
using Google.FlatBuffers;
using ModAbi;

namespace CuoModSdk;

/// <summary>Packet filter (either direction): return <c>true</c> to BLOCK the packet.</summary>
public delegate bool PacketFilter(byte id, ReadOnlySpan<byte> data);

/// <summary>
/// Dispatch layer over the raw ABI — the twin of cuo-mod-sdk runtime.rs.
///
/// Lifecycle (host drives, serially):
/// 1. <c>mod_setup(Handshake)</c> — the mod's stub calls <see cref="Setup"/>, which
///    interns the type-path table, runs the mod's setup fn to collect systems /
///    observers / packet filter, and returns the SetupReply.
/// 2. <c>mod_run</c> / <c>mod_observer</c> — dispatch to the stored callback; return
///    its CommandBuffer (or 0 for none).
/// 3. <c>mod_filter(id, bytes)</c> / <c>mod_filter_out(id, bytes)</c> — run the
///    incoming / outgoing packet filter; nonzero blocks.
///
/// Boundary plumbing (Google.FlatBuffers): per-call inputs are read in place through
/// the struct accessors; returns are written with one reused FlatBufferBuilder.
/// Packed guest returns are <c>len&lt;&lt;32 | ptr</c> (0 = none).
/// </summary>
public static unsafe class ModRuntime
{
    // Callback ids are list indices: ModBuilder assigns sequential ids from 0.
    static List<Action<SystemInputView, Commands>> _systems = new();
    static List<Action<ObserverInputView, Commands>> _observers = new();
    static PacketFilter? _filter;
    static PacketFilter? _filterOut;
    static ModHost? _host;

    /// <summary>
    /// The mod ABI version this SDK is compiled against. <see cref="Setup"/> throws when
    /// the host's Handshake.AbiVersion differs — a silent mismatch corrupts every buffer
    /// that follows, so the guest fails loud at the one point the host can still report it.
    /// </summary>
    public const uint AbiVersion = 2;

    // temp id -> name, from the LAST command buffer handed to the host. Consumed by
    // mod_spawned (which the host calls synchronously right after applying it).
    static List<(uint TempId, string Name)>? _pendingNames;

    /// <summary>Body of the <c>mod_spawned</c> export: pair the temp ids the host just resolved with the names Spawn(name) recorded.</summary>
    internal static void Spawned(int ptr, int len)
    {
        var pending = _pendingNames;
        _pendingNames = null;
        if (pending == null || _host == null)
            return;

        var input = SpawnedInput.GetRootAsSpawnedInput(Wrap(ptr, len));
        for (var i = 0; i < input.SpawnedLength; i++)
        {
            var sr = input.Spawned(i)!.Value;
            foreach (var (tempId, name) in pending)
                if (tempId == sr.TempId)
                    _host.Named[name] = sr.Entity;
        }
    }

    /// <summary>
    /// Body of the generated <c>mod_setup</c> export (ModSdk.targets writes it from the
    /// csproj's <c>&lt;CuoModType&gt;</c>): hand the mod its builder, keep what it
    /// declared, reply with the declarations.
    /// </summary>
    public static long Setup(int ptr, int len, Mod mod)
    {
        var hs = Handshake.GetRootAsHandshake(Wrap(ptr, len)).UnPack();
        if (hs.AbiVersion != AbiVersion)
            throw new InvalidOperationException(
                $"mod ABI mismatch: host speaks v{hs.AbiVersion}, this mod was built against " +
                $"v{AbiVersion} — rebuild the mod against the current CuoModSdk");
        var host = new ModHost(hs);
        var m = new ModBuilder(host);
        mod.Setup(m);
        m.Finish();
        _host = host;
        _systems = m.SystemFns;
        _observers = m.ObserverFns;
        _filter = m.Filter;
        _filterOut = m.FilterOut;
        var reply = m.BuildReply();
        return Pack(SetupReply.Pack(Begin(), reply).Value);
    }

    internal static long Run(int sysId, int ptr, int len)
    {
        if ((uint)sysId >= (uint)_systems.Count)
            return 0;
        var input = new SystemInputView(SystemInput.GetRootAsSystemInput(Wrap(ptr, len)));
        _host!.Tick = input.Tick;
        var buffer = new CommandBufferBuilder();
        var cmds = new Commands(buffer, _host);
        _systems[sysId](input, cmds);
        cmds.Flush();
        return buffer.IsEmpty ? 0L : PackAndStash(buffer);
    }

    internal static long Observer(int obsId, long entity, int ptr, int len)
    {
        if ((uint)obsId >= (uint)_observers.Count)
            return 0;
        var input = new ObserverInputView(ObserverInput.GetRootAsObserverInput(Wrap(ptr, len)), (ulong)entity);
        var buffer = new CommandBufferBuilder();
        var cmds = new Commands(buffer, _host!);
        _observers[obsId](input, cmds);
        cmds.Flush();
        return buffer.IsEmpty ? 0L : PackAndStash(buffer);
    }

    internal static int Filter(int id, int ptr, int len) =>
        _filter != null && _filter((byte)id, new ReadOnlySpan<byte>((void*)(nint)ptr, len)) ? 1 : 0;

    internal static int FilterOut(int id, int ptr, int len) =>
        _filterOut != null && _filterOut((byte)id, new ReadOnlySpan<byte>((void*)(nint)ptr, len)) ? 1 : 0;

    // Serialize the buffer AND stash its SpawnNamed bindings — the host resolves them
    // in the mod_spawned call it makes right after applying this buffer.
    static long PackAndStash(CommandBufferBuilder cmds)
    {
        var names = cmds.TakePendingNames();
        var packed = Pack(cmds.Finish(Begin()));
        if (names != null)
            _pendingNames = names;
        return packed;
    }

    // ── FlatBuffers <-> arena plumbing ───────────────────────────────────────────
    static readonly FlatBufferBuilder _fbb = new(1024);

    // The input is copied out of the arena: ByteBuffer wants a managed array, and the
    // views over it are only used for the duration of this call.
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

    // Finish the root, copy the finished bytes into a fresh arena reservation, return the
    // packed len<<32 | ptr. The arena still holds the (already-read) input at lower
    // offsets; the bump pointer places this after it.
    static long Pack(int root)
    {
        _fbb.Finish(root);
        var bytes = _fbb.DataBuffer.ToArraySegment(_fbb.DataBuffer.Position, _fbb.Offset);
        var n = bytes.Count;
        var ptr = Arena.Alloc(n);
        bytes.AsSpan().CopyTo(new Span<byte>((void*)(nint)ptr, n));
        return ((long)n << 32) | (uint)ptr;
    }
}

// The generic ABI exports — everything except mod_setup, which must construct the mod's
// own type and is therefore GENERATED into the mod (ModSdk.targets, from <CuoModType>):
// nothing in this assembly references the mod, so that stub is also ILC's only root for
// the mod's code. Exported from this assembly into the mod's module via
// UnmanagedEntryPointsAssembly (ModSdk.targets).
// mod_observer / mod_filter / mod_filter_out are resolved as optional by the host; exporting them with
// nothing registered is harmless (dispatch returns 0).
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

    [UnmanagedCallersOnly(EntryPoint = "mod_filter")]
    static int Filter(int id, int ptr, int len) => ModRuntime.Filter(id, ptr, len);

    [UnmanagedCallersOnly(EntryPoint = "mod_filter_out")]
    static int FilterOut(int id, int ptr, int len) => ModRuntime.FilterOut(id, ptr, len);

    [UnmanagedCallersOnly(EntryPoint = "mod_spawned")]
    static void Spawned(int ptr, int len) => ModRuntime.Spawned(ptr, len);
}
