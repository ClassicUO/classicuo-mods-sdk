using System.Runtime.InteropServices;
using Ecs = ModWorld.wit.Imports.tinyecs.modding.v0_1_0.IEcsImports;

namespace CuoModSdk;

/// <summary>
/// The component's exports on top of <see cref="ModBuilder"/>. A mod is a component of
/// its OWN world: <c>setup</c> plus one export per system / observer it declares, named
/// like the system (wasvy-style; see ModDescribe.cs for how the build derives them).
///
/// Lifecycle (the host drives, serially):
/// 1. <c>setup(app)</c> — generated into the mod: run the mod's Setup, check it
///    registered exactly the systems the build exported, declare them on the host.
/// 2. <c>&lt;system&gt;(params…)</c> — the generated export hands its param handles (and
///    trigger / packet) to <see cref="RunSystem"/> / <see cref="RunObserver"/> /
///    <see cref="RunPacket"/> with the system's index; the body runs, the handles drop.
/// </summary>
public static unsafe class ModRuntime
{
    static List<Entry> _entries = new();
    static readonly ModHost _host = new();

    internal static ModHost Host => _host;

    /// <summary>
    /// Body of the generated <c>setup</c> export. <paramref name="exported"/> is the
    /// build-time signature of every export (<see cref="Entry.Signature"/>), in
    /// registration order: the exports dispatch by index, so Setup must register the
    /// same systems, in the same order, as it did when the mod was built.
    /// </summary>
    [System.ComponentModel.EditorBrowsable(System.ComponentModel.EditorBrowsableState.Never)]
    public static void Setup(int appHandle, Mod mod, string[] exported)
    {
        using var app = new Ecs.App(new Ecs.App.THandle(appHandle));
        var m = new ModBuilder(_host);
        mod.Setup(m);
        m.Finish();
        _entries = m.Entries;
        var mismatch = Mismatch(_entries, exported);
        if (mismatch != null)
        {
            CuoModSdk.Host.Log(mismatch);
            throw new InvalidOperationException(mismatch);
        }
        m.Declare(app);
    }

    static string? Mismatch(List<Entry> entries, string[] exported)
    {
        for (var i = 0; i < Math.Max(entries.Count, exported.Length); i++)
        {
            var now = i < entries.Count ? entries[i].Signature() : "(none)";
            var built = i < exported.Length ? exported[i] : "(none)";
            if (now != built)
                return $"mod setup registered system #{i} as {now}, but the mod was built exporting {built}: " +
                       "Setup must register the same systems in the same order on every run (no registration " +
                       "that depends on runtime state); rebuild the mod after changing its systems";
        }
        return null;
    }

    // ── per-run state (runs never nest: the host drives the exports serially) ──
    static readonly RunScope _scope = new();
    static byte[] _kinds = new byte[8];
    static int[] _handles = new int[8];
    static byte[] _packet = new byte[512];

    static Entry Lift(int index, int* handles, int count)
    {
        var entry = _entries[index];
        if (_kinds.Length < count)
        {
            _kinds = new byte[count];
            _handles = new int[count];
        }
        for (var i = 0; i < count; i++)
        {
            _kinds[i] = entry.Params[i].Kind switch
            {
                ParamKind.Commands => EcsAbi.ParamCommands,
                ParamKind.Query => EcsAbi.ParamQuery,
                ParamKind.Events => EcsAbi.ParamEvents,
                _ => EcsAbi.ParamRes,
            };
            _handles[i] = handles[i];
        }
        return entry;
    }

    static Verdict Execute(Entry entry, int paramCount, ulong triggerEntity, nint value, int valueLen, Packet packet, bool ownsValue)
    {
        var scope = _scope;
        scope.Begin(_kinds, _handles, paramCount, entry.Locals);
        scope.Entry = entry;
        scope.TriggerEntity = triggerEntity;
        scope.TriggerValue = value;
        scope.TriggerValueLen = valueLen;
        scope.Packet = packet;
        try
        {
            entry.Run(scope);
            scope.End();
            return scope.Verdict;
        }
        finally
        {
            scope.Release();
            if (ownsValue)
                EcsAbi.Free(value, valueLen);
            for (var i = 0; i < paramCount; i++)
                EcsAbi.Drop(_kinds[i], _handles[i]);
        }
    }

    /// <summary>A scheduled system's export: <c>func(&lt;params&gt;)</c>.</summary>
    [System.ComponentModel.EditorBrowsable(System.ComponentModel.EditorBrowsableState.Never)]
    public static void RunSystem(int index, int* handles, int count) =>
        Execute(Lift(index, handles, count), count, 0, 0, 0, default, false);

    /// <summary>An observer's export: <c>func(trigger: trigger-data, &lt;params&gt;)</c>.</summary>
    [System.ComponentModel.EditorBrowsable(System.ComponentModel.EditorBrowsableState.Never)]
    public static void RunObserver(int index, long entity, nint value, int valueLen, int* handles, int count) =>
        Execute(Lift(index, handles, count), count, (ulong)entity, value, valueLen, default, true);

    // The verdict's return area: tag u8 @0, replacement list<u8> @4/@8.
    [StructLayout(LayoutKind.Sequential)]
    struct VerdictRet
    {
        public int Tag, Ptr, Len;
    }

    [System.Runtime.CompilerServices.FixedAddressValueType]
    static VerdictRet _verdictRet;

    /// <summary>An on-packet observer's export: <c>func(direction, packet: list&lt;u8&gt;, &lt;params&gt;) -&gt; verdict</c>.</summary>
    [System.ComponentModel.EditorBrowsable(System.ComponentModel.EditorBrowsableState.Never)]
    public static nint RunPacket(int index, int direction, nint packet, int packetLen, int* handles, int count)
    {
        var entry = Lift(index, handles, count);
        // Copy into a reused buffer (the Packet the observer sees stays valid for its run).
        if (_packet.Length < packetLen)
            _packet = new byte[Math.Max(packetLen, _packet.Length * 2)];
        new ReadOnlySpan<byte>((void*)packet, packetLen).CopyTo(_packet);
        EcsAbi.Free(packet, packetLen);
        var p = new Packet((PacketDirection)direction, new ArraySegment<byte>(_packet, 0, packetLen));
        var verdict = Execute(entry, count, 0, 0, 0, p, false);

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

    /// <summary><c>cabi_post_&lt;packet observer&gt;</c>: frees the replacement bytes.</summary>
    [System.ComponentModel.EditorBrowsable(System.ComponentModel.EditorBrowsableState.Never)]
    public static void PostPacket(nint ret)
    {
        var r = (VerdictRet*)ret;
        if ((byte)r->Tag == (byte)VerdictKind.Replace)
            NativeMemory.Free((void*)(nint)r->Ptr);
    }
}
