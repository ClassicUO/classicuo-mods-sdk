// The cuo:modding host functions for C# mods — world cuo:modding/mod's `host`, `packets`,
// `assets` and `actions` imports (../wit/cuo-mod.wit) as plain static calls on the
// generated canonical-ABI glue in Wit/ (regen-wit.sh). The public shapes (enums, record
// structs, the Destination variant) are the SDK's own and stable; only the bodies call
// into Wit/. Keep in step with the WIT by hand: a WIT change regenerates Wit/ and the
// compiler points at every body that no longer fits.
#nullable enable
#pragma warning disable CS1591
using System.Runtime.InteropServices;
using G = ModWorld.wit.Imports.cuo.modding.v0_1_0;

namespace CuoModSdk;

public static partial class Host
{

    /// <summary>
    /// Where `storage-get` / `storage-set` keep the mod's blob.
    /// </summary>
    public enum Scope
    {
        /// <summary>
        /// One blob for the mod, shared by every character.
        /// </summary>
        Global,
        /// <summary>
        /// One blob per character (empty until a character is logged in).
        /// </summary>
        Character,
    }

    /// <summary>
    /// Writes a line to the client log, prefixed with the mod name.
    /// </summary>
    public static void Log(string message)
    {
        G.IHostImports.Log(message);
    }

    /// <summary>
    /// Pixel width of `text` in a UO font.
    /// </summary>
    public static uint MeasureText(ushort font, string text)
    {
        return G.IHostImports.MeasureText(font, text);
    }

    /// <summary>
    /// The entity of a UO serial (an item or mobile the client knows); none when
    /// unknown.
    /// </summary>
    public static ulong? ResolveSerial(uint serial)
    {
        return G.IHostImports.ResolveSerial(serial);
    }

    /// <summary>
    /// The mod's stored blob ("" when nothing was stored).
    /// </summary>
    public static string StorageGet(Scope scope)
    {
        return G.IHostImports.StorageGet((G.IHostImports.Scope)scope);
    }

    /// <summary>
    /// Replaces the mod's stored blob. Written to disk immediately.
    /// </summary>
    public static void StorageSet(Scope scope, string value)
    {
        G.IHostImports.StorageSet((G.IHostImports.Scope)scope, value);
    }
}

/// <summary>
/// Raw UO packets, both ways. A packet is the full wire bytes, id first. Prefer
/// world data and `actions` when they cover what you need: they keep the client's
/// own state in step. To see / block / rewrite packets, add an observer with the
/// `on-packet` trigger (tinyecs:modding/ecs): incoming = server to client, outgoing =
/// client to server.
/// </summary>
public static partial class Packets
{

    /// <summary>
    /// Sends a packet to the server, as if the client had sent it.
    /// </summary>
    public static void SendToServer(System.ReadOnlySpan<byte> packet)
    {
        G.IPacketsImports.SendToServer(MemoryMarshal.CreateSpan(ref MemoryMarshal.GetReference(packet), packet.Length));
    }

    /// <summary>
    /// Feeds a packet to the client, as if the server had sent it.
    /// </summary>
    public static void SendToClient(System.ReadOnlySpan<byte> packet)
    {
        G.IPacketsImports.SendToClient(MemoryMarshal.CreateSpan(ref MemoryMarshal.GetReference(packet), packet.Length));
    }
}

/// <summary>
/// The client's UO data files, read-only. Images come back decoded from the files
/// (not from the GPU), so they work before anything was drawn.
///
/// Conventions: pixels are RGBA8, straight alpha, row-major, `width * height * 4`
/// bytes. Colours are u32 0xAARRGGBB. A `hue` argument is the wire value: 0 = none,
/// 0x8000 bit = partial hue (only grey pixels recoloured).
/// </summary>
public static partial class Assets
{

    static G.IAssetsImports.AnimKey Lower(AnimKey k) => new(k.Body, k.Action, k.Direction);
    static Size Lift(G.IAssetsImports.Size v) => new() { Width = v.width, Height = v.height };
    static Image Lift(G.IAssetsImports.Image v) => new() { Width = v.width, Height = v.height, Rgba = v.rgba };
    static FrameInfo Lift(G.IAssetsImports.FrameInfo v) => new() { Width = v.width, Height = v.height, CenterX = v.centerX, CenterY = v.centerY };
    static StaticInfo Lift(G.IAssetsImports.StaticInfo v) => new()
    {
        Name = v.name, Flags = v.flags, Weight = v.weight, Layer = v.layer, Count = v.count,
        AnimId = v.animId, Hue = v.hue, LightIndex = v.lightIndex, Height = v.height,
    };
    static LandInfo Lift(G.IAssetsImports.LandInfo v) => new() { Name = v.name, Flags = v.flags, Texmap = v.texmap };
    static LandCell Lift(G.IAssetsImports.LandCell v) => new() { Graphic = v.graphic, Z = v.z };
    static StaticCell Lift(G.IAssetsImports.StaticCell v) => new() { Graphic = v.graphic, Z = v.z, Hue = v.hue };
    static MultiPart Lift(G.IAssetsImports.MultiPart v) => new() { Graphic = v.graphic, X = v.x, Y = v.y, Z = v.z };
    static Skill Lift(G.IAssetsImports.Skill v) => new() { Index = v.index, Name = v.name, HasAction = v.hasAction };

    public enum ImageKind
    {
        /// <summary>
        /// Land tile art, id 0..0x3FFF (44x44 diamond).
        /// </summary>
        Land,
        /// <summary>
        /// Item / static art, by graphic.
        /// </summary>
        Art,
        /// <summary>
        /// Gump image.
        /// </summary>
        Gump,
        /// <summary>
        /// Land texture (64x64 or 128x128), by tiledata texmap id.
        /// </summary>
        Texmap,
        /// <summary>
        /// Light shape, 0..99.
        /// </summary>
        Light,
    }

    public struct Size
    {
        public uint Width;
        public uint Height;
    }

    public struct Image
    {
        public uint Width;
        public uint Height;
        public byte[] Rgba;
    }

    /// <summary>
    /// Which mobile animation to read. `direction` is 0..7 (the host mirrors the
    /// ones UO stores flipped); `action` is the UO action group.
    /// </summary>
    public struct AnimKey
    {
        public ushort Body;
        public byte Action;
        public byte Direction;
    }

    public struct FrameInfo
    {
        public uint Width;
        public uint Height;
        /// <summary>
        /// Draw offset: the frame's anchor point relative to its top-left.
        /// </summary>
        public int CenterX;
        public int CenterY;
    }

    public struct StaticInfo
    {
        public string Name;
        public ulong Flags;
        public byte Weight;
        public byte Layer;
        public int Count;
        public ushort AnimId;
        public ushort Hue;
        public ushort LightIndex;
        public byte Height;
    }

    public struct LandInfo
    {
        public string Name;
        public ulong Flags;
        public ushort Texmap;
    }

    /// <summary>
    /// Map file data only (no items, no house customisation). `map` is the facet
    /// index.
    /// </summary>
    public struct LandCell
    {
        public ushort Graphic;
        public sbyte Z;
    }

    public struct StaticCell
    {
        public ushort Graphic;
        public sbyte Z;
        public ushort Hue;
    }

    /// <summary>
    /// The pieces of a multi (house, boat), offsets relative to its centre.
    /// </summary>
    public struct MultiPart
    {
        public ushort Graphic;
        public short X;
        public short Y;
        public short Z;
    }

    public struct Skill
    {
        public ushort Index;
        public string Name;
        public bool HasAction;
    }

    /// <summary>
    /// Width and height; none when the id is missing.
    /// </summary>
    public static Size? ImageSize(ImageKind kind, uint id)
    {
        return G.IAssetsImports.ImageSize((G.IAssetsImports.ImageKind)kind, id) is { } r ? Lift(r) : null;
    }

    /// <summary>
    /// Decoded pixels with `hue` applied; none when the id is missing.
    /// </summary>
    public static Image? ImagePixels(ImageKind kind, uint id, ushort hue)
    {
        return G.IAssetsImports.ImagePixels((G.IAssetsImports.ImageKind)kind, id, hue) is { } r ? Lift(r) : null;
    }

    /// <summary>
    /// Number of frames; none when the body has no such animation.
    /// </summary>
    public static uint? AnimFrames(AnimKey key)
    {
        return G.IAssetsImports.AnimFrames(Lower(key));
    }

    /// <summary>
    /// Size and anchor of one frame.
    /// </summary>
    public static FrameInfo? AnimFrame(AnimKey key, uint frame)
    {
        return G.IAssetsImports.AnimFrame(Lower(key), frame) is { } r ? Lift(r) : null;
    }

    /// <summary>
    /// Decoded pixels of one frame (already mirrored), with `hue` applied.
    /// </summary>
    public static Image? AnimPixels(AnimKey key, uint frame, ushort hue)
    {
        return G.IAssetsImports.AnimPixels(Lower(key), frame, hue) is { } r ? Lift(r) : null;
    }

    /// <summary>
    /// The tiledata row of an item / static graphic.
    /// </summary>
    public static StaticInfo? StaticTile(ushort graphic)
    {
        return G.IAssetsImports.StaticTile(graphic) is { } r ? Lift(r) : null;
    }

    /// <summary>
    /// The tiledata row of a land tile.
    /// </summary>
    public static LandInfo? LandTile(ushort id)
    {
        return G.IAssetsImports.LandTile(id) is { } r ? Lift(r) : null;
    }

    /// <summary>
    /// The 32-colour ramp of a hue (1-based, as on the wire), dark to light.
    /// </summary>
    public static uint[]? HueRamp(ushort hue)
    {
        return G.IAssetsImports.HueRamp(hue);
    }

    /// <summary>
    /// A cliloc by id with its `~n_ARG~` slots filled from `args`; empty when
    /// unknown.
    /// </summary>
    public static string Cliloc(uint id, System.ReadOnlySpan<string> args)
    {
        return G.IAssetsImports.Cliloc(id, new System.Collections.Generic.List<string>(args.ToArray()));
    }

    public static Size? MapSize(byte map)
    {
        return G.IAssetsImports.MapSize(map) is { } r ? Lift(r) : null;
    }

    public static LandCell? MapLand(byte map, ushort x, ushort y)
    {
        return G.IAssetsImports.MapLand(map, x, y) is { } r ? Lift(r) : null;
    }

    public static StaticCell[] MapStatics(byte map, ushort x, ushort y)
    {
        return G.IAssetsImports.MapStatics(map, x, y).ConvertAll(Lift).ToArray();
    }

    public static MultiPart[] Multi(ushort id)
    {
        return G.IAssetsImports.Multi(id).ConvertAll(Lift).ToArray();
    }

    public static Skill[] Skills()
    {
        return G.IAssetsImports.Skills().ConvertAll(Lift).ToArray();
    }
}

/// <summary>
/// What the player can do. Each call goes through the client's own code path (the
/// same one a click or hotkey uses), so the client's state — held item, last target,
/// last object, war mode, the target queue — stays in step with the server. Calls
/// are fire-and-forget; the outcome shows up as world data and events.
///
/// Only primitives are here. Combinations (move every item of a type, target self,
/// bandage self, ...) are SDK helpers built on these.
/// </summary>
public static partial class Actions
{

    static G.IActionsImports.Point Lower(Point p) => new(p.X, p.Y, p.Z);
    static G.IActionsImports.Destination Lower(Destination d) => d switch
    {
        Destination.Container c => G.IActionsImports.Destination.Container(c.Value),
        Destination.ContainerAt c => G.IActionsImports.Destination.ContainerAt(c.Value),
        Destination.Ground c => G.IActionsImports.Destination.Ground(Lower(c.Value)),
        _ => throw new System.ArgumentOutOfRangeException(nameof(d)),
    };

    public enum Direction
    {
        North,
        NorthEast,
        East,
        SouthEast,
        South,
        SouthWest,
        West,
        NorthWest,
    }

    public enum SpeechKind
    {
        Regular,
        Emote,
        Whisper,
        Yell,
        Guild,
        Alliance,
    }

    public enum LockState
    {
        Up,
        Down,
        Locked,
    }

    public enum Stat
    {
        Str,
        Dex,
        Int,
    }

    public enum Virtue
    {
        Honor,
        Sacrifice,
        Valor,
    }

    public enum Ability
    {
        Primary,
        Secondary,
    }

    public enum Window
    {
        Paperdoll,
        Status,
        Journal,
        Skills,
        Backpack,
        WorldMap,
        Minimap,
        Options,
        Buffs,
        CombatBook,
        Party,
        Guild,
        Quest,
        Logout,
    }

    public struct Point
    {
        public ushort X;
        public ushort Y;
        public sbyte Z;
    }

    public abstract record Destination
    {
        private Destination() { }
        /// <summary>
        /// Into a container; the server picks the slot.
        /// </summary>
        public sealed record Container(uint Value) : Destination;
        /// <summary>
        /// Into a container at a slot (x, y).
        /// </summary>
        public sealed record ContainerAt((uint, ushort, ushort) Value) : Destination;
        /// <summary>
        /// On the ground.
        /// </summary>
        public sealed record Ground(Point Value) : Destination;
    }

    /// <summary>
    /// ── items ──
    /// </summary>
    public static void DoubleClick(uint target)
    {
        G.IActionsImports.DoubleClick(target);
    }

    public static void SingleClick(uint target)
    {
        G.IActionsImports.SingleClick(target);
    }

    /// <summary>
    /// Picks the item up onto the cursor (amount 0 = whole stack).
    /// </summary>
    public static void PickUp(uint item, ushort amount)
    {
        G.IActionsImports.PickUp(item, amount);
    }

    /// <summary>
    /// Drops the item held on the cursor.
    /// </summary>
    public static void Drop(Destination to)
    {
        G.IActionsImports.Drop(Lower(to));
    }

    /// <summary>
    /// Equips the item held on the cursor.
    /// </summary>
    public static void Equip(byte layer, uint mobile)
    {
        G.IActionsImports.Equip(layer, mobile);
    }

    /// <summary>
    /// Pick up + drop in one step, without touching the cursor.
    /// </summary>
    public static void MoveItem(uint item, ushort amount, Destination to)
    {
        G.IActionsImports.MoveItem(item, amount, Lower(to));
    }

    /// <summary>
    /// Pick up + equip in one step.
    /// </summary>
    public static void Wear(uint item, byte layer, uint mobile)
    {
        G.IActionsImports.Wear(item, layer, mobile);
    }

    /// <summary>
    /// Uses `item` on `target` (bandages, keys, ...).
    /// </summary>
    public static void UseItemOn(uint item, uint target)
    {
        G.IActionsImports.UseItemOn(item, target);
    }

    public static void EquipLastWeapon()
    {
        G.IActionsImports.EquipLastWeapon();
    }

    public static void Rename(uint mobile, string name)
    {
        G.IActionsImports.Rename(mobile, name);
    }

    public static void RequestProperties(uint target)
    {
        G.IActionsImports.RequestProperties(target);
    }

    public static void CloseContainer(uint container)
    {
        G.IActionsImports.CloseContainer(container);
    }

    /// <summary>
    /// ── movement ──
    /// </summary>
    public static void Walk(Direction dir, bool run)
    {
        G.IActionsImports.Walk((G.IActionsImports.Direction)dir, run);
    }

    /// <summary>
    /// Pathfinds to a tile.
    /// </summary>
    public static void WalkTo(Point at)
    {
        G.IActionsImports.WalkTo(Lower(at));
    }

    public static void CancelWalkTo()
    {
        G.IActionsImports.CancelWalkTo();
    }

    public static void OpenDoor()
    {
        G.IActionsImports.OpenDoor();
    }

    public static void Resync()
    {
        G.IActionsImports.Resync();
    }

    /// <summary>
    /// ── combat ──
    /// </summary>
    public static void SetWarMode(bool on)
    {
        G.IActionsImports.SetWarMode(on);
    }

    public static void Attack(uint mobile)
    {
        G.IActionsImports.Attack(mobile);
    }

    /// <summary>
    /// Toggles the weapon's special ability.
    /// </summary>
    public static void UseAbility(Ability which)
    {
        G.IActionsImports.UseAbility((G.IActionsImports.Ability)which);
    }

    public static void ClearAbility()
    {
        G.IActionsImports.ClearAbility();
    }

    public static void ToggleFlying()
    {
        G.IActionsImports.ToggleFlying();
    }

    /// <summary>
    /// ── targeting ──
    /// Answers the open target cursor, or queues the answer for the next one.
    /// </summary>
    public static void TargetObject(uint target)
    {
        G.IActionsImports.TargetObject(target);
    }

    public static void TargetLocation(Point at, ushort graphic)
    {
        G.IActionsImports.TargetLocation(Lower(at), graphic);
    }

    public static void TargetCancel()
    {
        G.IActionsImports.TargetCancel();
    }

    public static void ClearTargetQueue()
    {
        G.IActionsImports.ClearTargetQueue();
    }

    /// <summary>
    /// Shows a client-side target cursor; the pick arrives as `cuo:target/local-result`.
    /// </summary>
    public static void RequestTarget(string prompt)
    {
        G.IActionsImports.RequestTarget(prompt);
    }

    /// <summary>
    /// ── spells, skills, virtues ──
    /// </summary>
    public static void CastSpell(ushort id)
    {
        G.IActionsImports.CastSpell(id);
    }

    public static void OpenSpellbook(byte kind)
    {
        G.IActionsImports.OpenSpellbook(kind);
    }

    public static void UseSkill(ushort index)
    {
        G.IActionsImports.UseSkill(index);
    }

    public static void SetSkillLock(ushort index, LockState state)
    {
        G.IActionsImports.SetSkillLock(index, (G.IActionsImports.LockState)state);
    }

    public static void SetStatLock(Stat which, LockState state)
    {
        G.IActionsImports.SetStatLock((G.IActionsImports.Stat)which, (G.IActionsImports.LockState)state);
    }

    public static void InvokeVirtue(Virtue which)
    {
        G.IActionsImports.InvokeVirtue((G.IActionsImports.Virtue)which);
    }

    /// <summary>
    /// "bow" or "salute".
    /// </summary>
    public static void EmoteAction(string name)
    {
        G.IActionsImports.EmoteAction(name);
    }

    /// <summary>
    /// ── speech ──
    /// </summary>
    public static void Say(string text, SpeechKind kind, ushort? hue)
    {
        G.IActionsImports.Say(text, (G.IActionsImports.SpeechKind)kind, hue);
    }

    /// <summary>
    /// To the whole party, or one member.
    /// </summary>
    public static void PartySay(string text, uint? to)
    {
        G.IActionsImports.PartySay(text, to);
    }

    /// <summary>
    /// ── gumps, menus, prompts ──
    /// </summary>
    public static void GumpReply(uint gump, int button, System.ReadOnlySpan<uint> switches, System.ReadOnlySpan<(ushort, string)> texts)
    {
        G.IActionsImports.GumpReply(gump, button, MemoryMarshal.CreateSpan(ref MemoryMarshal.GetReference(switches), switches.Length), new System.Collections.Generic.List<(ushort, string)>(texts.ToArray()));
    }

    public static void GumpClose(uint gump)
    {
        G.IActionsImports.GumpClose(gump);
    }

    public static void RequestContextMenu(uint target)
    {
        G.IActionsImports.RequestContextMenu(target);
    }

    public static void ContextMenuReply(uint target, ushort index)
    {
        G.IActionsImports.ContextMenuReply(target, index);
    }

    public static void PromptReply(string text)
    {
        G.IActionsImports.PromptReply(text);
    }

    public static void PromptCancel()
    {
        G.IActionsImports.PromptCancel();
    }

    public static void MenuReply(uint menu, ushort menuId, ushort index, ushort graphic, ushort hue)
    {
        G.IActionsImports.MenuReply(menu, menuId, index, graphic, hue);
    }

    public static void TextEntryReply(uint dialog, byte parentId, byte button, string text, bool ok)
    {
        G.IActionsImports.TextEntryReply(dialog, parentId, button, text, ok);
    }

    public static void DyeReply(uint tub, ushort graphic, ushort hue)
    {
        G.IActionsImports.DyeReply(tub, graphic, hue);
    }

    public static void HelpRequest()
    {
        G.IActionsImports.HelpRequest();
    }

    /// <summary>
    /// ── party ──
    /// The server answers with a target cursor.
    /// </summary>
    public static void PartyInvite()
    {
        G.IActionsImports.PartyInvite();
    }

    /// <summary>
    /// None = pick by target.
    /// </summary>
    public static void PartyRemove(uint? member)
    {
        G.IActionsImports.PartyRemove(member);
    }

    public static void PartyAccept(uint leader)
    {
        G.IActionsImports.PartyAccept(leader);
    }

    public static void PartyDecline(uint leader)
    {
        G.IActionsImports.PartyDecline(leader);
    }

    public static void PartySetLoot(bool shareable)
    {
        G.IActionsImports.PartySetLoot(shareable);
    }

    /// <summary>
    /// ── trade, vendors ──
    /// </summary>
    public static void TradeAccept(uint container, bool accepted)
    {
        G.IActionsImports.TradeAccept(container, accepted);
    }

    public static void TradeCancel(uint container)
    {
        G.IActionsImports.TradeCancel(container);
    }

    public static void TradeSetGold(uint container, uint gold, uint platinum)
    {
        G.IActionsImports.TradeSetGold(container, gold, platinum);
    }

    public static void VendorBuy(uint vendor, System.ReadOnlySpan<(uint, ushort)> items)
    {
        G.IActionsImports.VendorBuy(vendor, new System.Collections.Generic.List<(uint, ushort)>(items.ToArray()));
    }

    public static void VendorSell(uint vendor, System.ReadOnlySpan<(uint, ushort)> items)
    {
        G.IActionsImports.VendorSell(vendor, new System.Collections.Generic.List<(uint, ushort)>(items.ToArray()));
    }

    /// <summary>
    /// ── requests, client ──
    /// None = yourself.
    /// </summary>
    public static void RequestStatus(uint? mobile)
    {
        G.IActionsImports.RequestStatus(mobile);
    }

    public static void RequestSkills(uint? mobile)
    {
        G.IActionsImports.RequestSkills(mobile);
    }

    /// <summary>
    /// `target` picks whose paperdoll / status; none = yours.
    /// </summary>
    public static void OpenWindow(Window w_, uint? target)
    {
        G.IActionsImports.OpenWindow((G.IActionsImports.Window)w_, target);
    }

    public static void CloseWindow(Window w_)
    {
        G.IActionsImports.CloseWindow((G.IActionsImports.Window)w_);
    }

    public static void SetViewRange(byte range)
    {
        G.IActionsImports.SetViewRange(range);
    }

    public static void Logout()
    {
        G.IActionsImports.Logout();
    }
}
