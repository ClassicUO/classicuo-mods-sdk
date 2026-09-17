namespace CuoModSdk.Types;

// Ergonomic constructors for the generated cuo UI payload shapes — the C# twin of
// cuo-mod-sdk/src/ui.rs. There is exactly ONE Node / Val / UiRect / Color in this SDK:
// the generated struct in Types.cs, which mirrors what the host writes through
// System.Text.Json. These `partial` halves only add constructors, so a mod can never
// build a Node the host doesn't recognise.

public partial struct Val
{
    public static Val Auto() => new() { Type = ValType.Auto, IsAuto = true };
    public static Val Px(float v) => new() { Type = ValType.Px, Value = v };
    public static Val Percent(float v) => new() { Type = ValType.Percent, Value = v };
    /// <summary>Fill what the parent has left (Clay Grow); MinWidth/MinHeight Px is the floor.</summary>
    public static Val Grow() => new() { Type = ValType.Grow };
}

public partial struct UiRect
{
    public static UiRect Splat(Val v) => new() { Left = v, Right = v, Top = v, Bottom = v };
    public static UiRect Zero() => Splat(Val.Px(0f));
}

public partial struct BorderRadius
{
    public static BorderRadius All(float r) => new() { TopLeft = r, TopRight = r, BottomLeft = r, BottomRight = r };
}

public partial struct Node
{
    /// <summary>The default the mods start from: Flex / Relative / Visible / Row / Start / Start, every Val Auto, padding + border zero (Px 0), gap Px 0.</summary>
    public static Node Base() => new()
    {
        Width = Val.Auto(),
        Height = Val.Auto(),
        MinWidth = Val.Auto(),
        MinHeight = Val.Auto(),
        MaxWidth = Val.Auto(),
        MaxHeight = Val.Auto(),
        Left = Val.Auto(),
        Top = Val.Auto(),
        Right = Val.Auto(),
        Bottom = Val.Auto(),
        Padding = UiRect.Zero(),
        Border = UiRect.Zero(),
        Gap = Val.Px(0f),
    };

    /// <summary>Absolutely positioned, fixed-size node — the shape of every UO gump element.</summary>
    public static Node Abs(float left, float top, float w, float h)
    {
        var n = Base();
        n.PositionType = PositionType.Absolute;
        n.Left = Val.Px(left);
        n.Top = Val.Px(top);
        n.Width = Val.Px(w);
        n.Height = Val.Px(h);
        return n;
    }
}

public partial struct Color
{
    /// <summary>Channels are 0..=255 (the host's Clay convention), not 0..=1.</summary>
    public static Color Rgba(byte r, byte g, byte b, byte a) => new()
    {
        R = r,
        G = g,
        B = b,
        A = a,
        // IsVisible is `A > 0` on the host (a getter-only property STJ never writes
        // back); filled only so the payload is self-consistent.
        IsVisible = a > 0,
    };

    public static Color Rgb(byte r, byte g, byte b) => Rgba(r, g, b, 255);
}
