namespace CuoModSdk;

/// <summary>
/// A mod. Subclass it, override <see cref="Setup"/>, and name the subclass in the
/// csproj's <c>&lt;CuoModType&gt;</c> — ModSdk.targets generates the <c>mod_setup</c>
/// export that instantiates it. Everything else (arena, FlatBuffers, the other ABI
/// exports) is the SDK's.
///
/// One instance per guest, created once at setup: mod state goes in instance fields,
/// not statics. The guest is single-threaded, so no locking.
/// </summary>
public abstract class Mod
{
    /// <summary>Declare systems, observers, hotkeys and the packet filter. Runs once, before the first tick.</summary>
    public abstract void Setup(ModBuilder m);
}

/// <summary>
/// Host stage a system runs in — the mod-side mirror of <c>TinyEcs.Bevy.Stage</c>.
/// Values match <c>ModAbi.Schedule</c> (abi/mod-abi.fbs) so a mod never needs
/// <c>using ModAbi</c>; keep the two in lockstep. For a host-named custom stage use
/// <c>SystemHandle.InStage(string)</c> instead.
/// </summary>
public enum Stage : byte
{
    /// <summary>Once, after the host registered the mod (storage is readable here). Wire <c>ModStartup</c>.</summary>
    Startup = 0,
    First = 1,
    PreUpdate = 2,
    Update = 3,
    PostUpdate = 4,
    Last = 5,
}
