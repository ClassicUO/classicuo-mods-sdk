using System.Runtime.InteropServices;
using Ecs = ModWorld.wit.Imports.tinyecs.modding.v0_1_0.IEcsImports;

namespace CuoModSdk;

/// <summary>
/// The component's exports (world <c>cuo:modding/mod</c>) on top of <see cref="ModBuilder"/>.
///
/// Lifecycle (the host drives, serially):
/// 1. <c>setup(app)</c> — the export generated into the mod (ModSdk.targets, from
///    <c>&lt;CuoModType&gt;</c>) calls <see cref="Setup"/>: run the mod's Setup, then
///    declare every system / observer on the host.
/// 2. <c>run</c> / <c>observe</c> / <c>observe-packet</c> — look the system up by name,
///    build its parameters from the param handles, run the body, drop the handles.
///
/// The exports lift their arguments by hand (no per-call strings or lists): the system
/// name is matched as UTF-8, the params land in reused arrays.
/// </summary>
public static unsafe class ModRuntime
{
    static List<Entry> _entries = new();
    // FNV-1a of the UTF-8 name -> entry index (collisions fall back to a scan).
    static readonly Dictionary<ulong, int> _byName = new();
    static readonly ModHost _host = new();

    internal static ModHost Host => _host;

    /// <summary>Body of the generated <c>setup</c> export (ModSdk.targets, from <c>&lt;CuoModType&gt;</c>).</summary>
    public static void Setup(int appHandle, Mod mod)
    {
        using var app = new Ecs.App(new Ecs.App.THandle(appHandle));
        var m = new ModBuilder(_host);
        mod.Setup(m);
        m.Finish();
        _entries = m.Entries;
        _byName.Clear();
        for (var i = 0; i < _entries.Count; i++)
            _byName[Hash(_entries[i].NameUtf8)] = i;
        m.Declare(app);
    }

    static ulong Hash(ReadOnlySpan<byte> s)
    {
        var h = 14695981039346656037UL;
        foreach (var b in s)
            h = (h ^ b) * 1099511628211UL;
        return h;
    }

    static Entry? Find(ReadOnlySpan<byte> name)
    {
        if (_byName.TryGetValue(Hash(name), out var i) && _entries[i].NameUtf8.AsSpan().SequenceEqual(name))
            return _entries[i];
        foreach (var e in _entries)
            if (e.NameUtf8.AsSpan().SequenceEqual(name))
                return e;
        return null;
    }

    // ── per-run state (runs never nest: the host drives the exports serially) ──
    static readonly RunScope _scope = new();
    static byte[] _kinds = new byte[8];
    static int[] _handles = new int[8];
    static byte[] _packet = new byte[512];

    // Lifts list<param> (8 bytes each: tag u8 @0, own handle i32 @4) and frees the list.
    static void LiftParams(nint list, int count)
    {
        if (_kinds.Length < count)
        {
            _kinds = new byte[count];
            _handles = new int[count];
        }
        for (var i = 0; i < count; i++)
        {
            _kinds[i] = *(byte*)(list + i * 8);
            _handles[i] = *(int*)(list + i * 8 + 4);
        }
        if (count > 0)
            NativeMemory.Free((void*)list);
    }

    static Verdict Execute(nint name, int nameLen, nint paramList, int paramCount, ulong triggerEntity, nint value, int valueLen, Packet packet, bool isObserver)
    {
        LiftParams(paramList, paramCount);
        var entry = Find(new ReadOnlySpan<byte>((void*)name, nameLen));
        EcsAbi.Free(name, nameLen);
        var scope = _scope;
        scope.Begin(_kinds, _handles, paramCount, entry?.Locals);
        scope.TriggerEntity = triggerEntity;
        scope.TriggerValue = value;
        scope.TriggerValueLen = valueLen;
        scope.Packet = packet;
        try
        {
            if (entry != null)
                entry.Run(scope);
            scope.End();
            return scope.Verdict;
        }
        finally
        {
            scope.Release();
            if (isObserver)
                EcsAbi.Free(value, valueLen);
            for (var i = 0; i < paramCount; i++)
                EcsAbi.Drop(_kinds[i], _handles[i]);
        }
    }

    internal static void Run(nint name, int nameLen, nint paramList, int paramCount) =>
        Execute(name, nameLen, paramList, paramCount, 0, 0, 0, default, false);

    internal static void Observe(nint name, int nameLen, long entity, nint value, int valueLen, nint paramList, int paramCount) =>
        Execute(name, nameLen, paramList, paramCount, (ulong)entity, value, valueLen, default, true);

    // The verdict's return area: tag u8 @0, replacement list<u8> @4/@8.
    [StructLayout(LayoutKind.Sequential)]
    struct VerdictRet
    {
        public int Tag, Ptr, Len;
    }

    [System.Runtime.CompilerServices.FixedAddressValueType]
    static VerdictRet _verdictRet;

    internal static nint ObservePacket(nint name, int nameLen, int direction, nint packet, int packetLen, nint paramList, int paramCount)
    {
        // Copy into a reused buffer (the Packet the observer sees stays valid for its run).
        if (_packet.Length < packetLen)
            _packet = new byte[Math.Max(packetLen, _packet.Length * 2)];
        new ReadOnlySpan<byte>((void*)packet, packetLen).CopyTo(_packet);
        EcsAbi.Free(packet, packetLen);
        var p = new Packet((PacketDirection)direction, new ArraySegment<byte>(_packet, 0, packetLen));
        var verdict = Execute(name, nameLen, paramList, paramCount, 0, 0, 0, p, true);

        fixed (VerdictRet* ret = &_verdictRet)
        {
            *ret = default;
            ret->Tag = (byte)verdict.Kind;
            if (verdict.Kind == VerdictKind.Replace)
            {
                var bytes = verdict.Replacement!;
                var mem = (byte*)NativeMemory.Alloc((nuint)Math.Max(1, bytes.Length));
                bytes.CopyTo(new Span<byte>(mem, bytes.Length));
                ret->Ptr = (int)(nint)mem;
                ret->Len = bytes.Length;
            }
            return (nint)ret;
        }
    }

    internal static void PostObservePacket(nint ret)
    {
        var r = (VerdictRet*)ret;
        if ((byte)r->Tag == (byte)VerdictKind.Replace)
            NativeMemory.Free((void*)(nint)r->Ptr);
    }
}

// The generic exports — everything except `setup`, which must construct the mod's own
// type and is therefore GENERATED into the mod (ModSdk.targets, from <CuoModType>).
// Exported from this assembly into the mod's component via UnmanagedEntryPointsAssembly.
// Signatures are the canonical-ABI flattening of the WIT exports.
internal static class WitExports
{
    // run: func(system: string, params: list<param>)
    [UnmanagedCallersOnly(EntryPoint = "run")]
    static void Run(nint name, int nameLen, nint paramList, int paramCount) =>
        ModRuntime.Run(name, nameLen, paramList, paramCount);

    // observe: func(system: string, trigger: trigger-data { entity: u64, value: string }, params: list<param>)
    [UnmanagedCallersOnly(EntryPoint = "observe")]
    static void Observe(nint name, int nameLen, long entity, nint value, int valueLen, nint paramList, int paramCount) =>
        ModRuntime.Observe(name, nameLen, entity, value, valueLen, paramList, paramCount);

    // observe-packet: func(system: string, direction: packet-direction, packet: list<u8>, params: list<param>) -> verdict
    [UnmanagedCallersOnly(EntryPoint = "observe-packet")]
    static nint ObservePacket(nint name, int nameLen, int direction, nint packet, int packetLen, nint paramList, int paramCount) =>
        ModRuntime.ObservePacket(name, nameLen, direction, packet, packetLen, paramList, paramCount);

    [UnmanagedCallersOnly(EntryPoint = "cabi_post_observe-packet")]
    static void PostObservePacket(nint ret) => ModRuntime.PostObservePacket(ret);
}
