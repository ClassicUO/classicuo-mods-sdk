using System.Runtime.InteropServices;
using FlatSharp;
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
/// Boundary plumbing: inputs are read with FlatSharp, returns are written with
/// FlatSharp (it reads its own deduplicated vtables fine — the Rust SDK hand-rolls a
/// wire reader only because planus can't follow FlatSharp's backward vtables).
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

        var input = Parse(SpawnedInput.Serializer, ptr, len);
        if (input.Spawned == null)
            return;
        foreach (var sr in input.Spawned)
            foreach (var (tempId, name) in pending)
                if (tempId == sr.TempId)
                    _host.Named[name] = sr.Entity;
    }

    /// <summary>
    /// Body of the generated <c>mod_setup</c> export (ModSdk.targets writes it from the
    /// csproj's <c>&lt;CuoModType&gt;</c>): hand the mod its builder, keep what it
    /// declared, reply with the declarations.
    /// </summary>
    public static long Setup(int ptr, int len, Mod mod)
    {
        var hs = Parse(Handshake.Serializer, ptr, len);
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
        return Pack(SetupReply.Serializer, m.BuildReply());
    }

    internal static long Run(int sysId, int ptr, int len)
    {
        if ((uint)sysId >= (uint)_systems.Count)
            return 0;
        var input = new SystemInputView(Parse(SystemInput.Serializer, ptr, len));
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
        var input = new ObserverInputView(Parse(ObserverInput.Serializer, ptr, len), (ulong)entity);
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
        var packed = Pack(CommandBuffer.Serializer, cmds.Finish());
        if (names != null)
            _pendingNames = names;
        return packed;
    }

    // ── FlatSharp <-> arena plumbing ─────────────────────────────────────────────
    static byte[] _scratch = new byte[1024];

    static T Parse<T>(ISerializer<T> serializer, int ptr, int len) where T : class
    {
        var arr = new byte[len];
        new ReadOnlySpan<byte>((void*)(nint)ptr, len).CopyTo(arr);
        return serializer.Parse(arr.AsMemory(0, len));
    }

    // Serialize into the scratch buffer, copy into a fresh arena reservation, return the
    // packed len<<32 | ptr. The arena still holds the (already-parsed) input at lower
    // offsets; the bump pointer places this after it.
    static long Pack<T>(ISerializer<T> serializer, T value) where T : class
    {
        var max = serializer.GetMaxSize(value);
        if (_scratch.Length < max)
            _scratch = new byte[max];
        var n = serializer.Write(_scratch, value);
        var ptr = Arena.Alloc(n);
        new ReadOnlySpan<byte>(_scratch, 0, n).CopyTo(new Span<byte>((void*)(nint)ptr, n));
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
