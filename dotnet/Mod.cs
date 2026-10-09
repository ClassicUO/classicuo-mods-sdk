namespace CuoModSdk;

/// <summary>
/// A mod. Subclass it, override <see cref="Setup"/>, and name the subclass in the
/// csproj's <c>&lt;CuoModType&gt;</c>. The component's exports are generated from it at
/// build time: <c>setup</c>, and one export per system / observer Setup registers, named
/// like the system (<see cref="SystemHandle.Label"/>, default <c>systemN</c> /
/// <c>observerN</c> in registration order). To learn them the build RUNS Setup on the
/// build machine (ModDescribe.cs), so Setup (and the constructor) must only register —
/// no host calls — and must register the same systems in the same order every time.
///
/// One instance per guest, created once at setup: mod state goes in instance fields,
/// not statics. The guest is single-threaded, so no locking.
/// </summary>
public abstract class Mod
{
    /// <summary>Declare systems, observers (packet observers too) and hotkeys. Runs once, before the first tick.</summary>
    public abstract void Setup(ModBuilder m);
}

/// <summary>
/// Host stage a system runs in — the mod-side mirror of <c>TinyEcs.Bevy.Stage</c>.
/// Values match the WIT <c>schedule</c> enum (wit/deps/tinyecs-mod); keep the two in lockstep.
/// </summary>
public enum Stage : byte
{
    /// <summary>Once, after the mod loads (storage is readable here). Wire <c>ModStartup</c>.</summary>
    Startup = 0,
    First = 1,
    PreUpdate = 2,
    Update = 3,
    PostUpdate = 4,
    Last = 5,
}
