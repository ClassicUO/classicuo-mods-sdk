/* UI payload constructors, chat and hotkeys — the hand-written layer over the generated
 * types (twin of rust/src/ui.rs + dotnet/Types.Ui.cs, ModContext.ChatApi, ModBuilder.Hotkey). */
#include <stdlib.h>
#include <string.h>

#include "internal.h"

cuo_Val cuo_val_auto(void)
{
    cuo_Val v = { CUO_VAL_TYPE_AUTO, 0, true };
    return v;
}

cuo_Val cuo_val_px(float x)
{
    cuo_Val v = { CUO_VAL_TYPE_PX, x, false };
    return v;
}

cuo_Val cuo_val_percent(float x)
{
    cuo_Val v = { CUO_VAL_TYPE_PERCENT, x, false };
    return v;
}

cuo_Val cuo_val_grow(void)
{
    cuo_Val v = { CUO_VAL_TYPE_GROW, 0, false };
    return v;
}

cuo_UiRect cuo_rect_splat(cuo_Val v)
{
    cuo_UiRect r = { v, v, v, v };
    return r;
}

cuo_UiRect cuo_rect_zero(void)
{
    return cuo_rect_splat(cuo_val_px(0));
}

cuo_BorderRadius cuo_radius_all(float r)
{
    cuo_BorderRadius b = { r, r, r, r };
    return b;
}

cuo_Node cuo_node_base(void)
{
    cuo_Node n;
    memset(&n, 0, sizeof n);
    n.width = n.height = cuo_val_auto();
    n.min_width = n.min_height = n.max_width = n.max_height = cuo_val_auto();
    n.left = n.top = n.right = n.bottom = cuo_val_auto();
    n.padding = n.border = cuo_rect_zero();
    n.gap = cuo_val_px(0);
    return n;
}

cuo_Node cuo_node_abs(float left, float top, float width, float height)
{
    cuo_Node n = cuo_node_base();
    n.position_type = CUO_POSITION_TYPE_ABSOLUTE;
    n.left = cuo_val_px(left);
    n.top = cuo_val_px(top);
    n.width = cuo_val_px(width);
    n.height = cuo_val_px(height);
    return n;
}

cuo_Color cuo_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a)
{
    /* IsVisible is A > 0 on the host (a getter STJ never reads back); filled only so
     * the payload is self-consistent. */
    cuo_Color c = { r, g, b, a, a > 0 };
    return c;
}

cuo_Color cuo_rgb(uint8_t r, uint8_t g, uint8_t b)
{
    return cuo_rgba(r, g, b, 255);
}

cuo_Color cuo_hue_color(uint32_t hue)
{
    uint32_t argb = cuo_hue_argb(hue);
    return cuo_rgba((uint8_t)(argb >> 16), (uint8_t)(argb >> 8), (uint8_t)argb, (uint8_t)(argb >> 24));
}

/* ── chat ────────────────────────────────────────────────────────────────── */

/* Font 3 unicode is what server speech arrives as; the zero value (ascii font 0) is
 * the big gothic fonts.mul face nothing else on screen uses. */
#define SPEECH_FONT 3

void cuo_chat_system(cuo_cmds *c, const char *text, uint16_t hue)
{
    cuo_ModChatMessage m;
    memset(&m, 0, sizeof m);
    m.text = text;
    m.name = "";
    m.hue = hue;
    m.font = SPEECH_FONT;
    m.is_unicode = true;
    m.kind = 1;
    cuo_ModChatMessage_emit(c, 0, &m);
}

void cuo_chat_overhead(cuo_cmds *c, const char *text, uint16_t hue, uint32_t serial, const char *name)
{
    cuo_ModChatMessage m;
    memset(&m, 0, sizeof m);
    m.text = text;
    m.name = name ? name : "";
    m.hue = hue;
    m.serial = serial;
    m.font = SPEECH_FONT;
    m.is_unicode = true;
    m.kind = 0;
    cuo_ModChatMessage_emit(c, 0, &m);
}

/* ── hotkeys ─────────────────────────────────────────────────────────────── */

static cuo_ModHotkeyBinding *bindings;
static size_t nbindings, capbindings;

static void add_binding(cuo_builder *m, const char *name, int32_t key, int32_t mouse, unsigned flags)
{
    (void)m;
    if (nbindings == capbindings) {
        capbindings = capbindings ? capbindings * 2 : 8;
        bindings = realloc(bindings, capbindings * sizeof *bindings);
    }
    size_t len = strlen(name) + 1;
    cuo_ModHotkeyBinding b = {
        .name = memcpy(malloc(len), name, len),
        .key = key,
        .mouse = mouse,
        .ctrl = (flags & CUO_HK_CTRL) != 0,
        .shift = (flags & CUO_HK_SHIFT) != 0,
        .alt = (flags & CUO_HK_ALT) != 0,
        .consume = (flags & CUO_HK_CONSUME) != 0,
    };
    bindings[nbindings++] = b;
}

void cuo_hotkey(cuo_builder *m, const char *name, uint32_t key, unsigned flags)
{
    add_binding(m, name, (int32_t)key, 0, flags);
}

void cuo_hotkey_mouse(cuo_builder *m, const char *name, int32_t mouse_button, unsigned flags)
{
    add_binding(m, name, 0, mouse_button, flags);
}

static void publish(const cuo_input *in, cuo_cmds *c, void *user)
{
    (void)in;
    (void)user;
    /* ConsumeKeys stays false: consume is per binding. */
    cuo_ModHotkeyBindingsDto dto = { { bindings, nbindings }, false };
    cuo_resource_set(c, cuo_ModHotkeyBindingsDto_comp(&dto));
}

void cuo__publish_hotkeys(cuo_builder *m)
{
    if (nbindings)
        cuo_add_system(m, "hotkeys", CUO_STAGE_STARTUP, publish, NULL);
}
