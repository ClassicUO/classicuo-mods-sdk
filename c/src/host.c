/* Typed wrappers over the cuo:modding imports (wit/cuo-mod.wit). Results the canonical
 * ABI malloc'd are copied into scratch and freed right away. */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "internal.h"

static const char *take_string(cuo_wit_string_t *s)
{
    const char *r = cuo_strndup((const char *)s->ptr, s->len);
    cuo_wit_string_free(s);
    return r;
}

/* ── host ────────────────────────────────────────────────────────────────── */

void cuo_log(const char *msg)
{
    cuo_wit_string_t s = cuo__wstr(msg);
    cuo_modding_host_log(&s);
}

void cuo_logf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(NULL, 0, fmt, ap);
    va_end(ap);
    char *d = cuo_alloc((size_t)n + 1);
    va_start(ap, fmt);
    vsnprintf(d, (size_t)n + 1, fmt, ap);
    va_end(ap);
    cuo_log(d);
}

uint32_t cuo_measure_text(uint16_t font, const char *text)
{
    cuo_wit_string_t s = cuo__wstr(text);
    return cuo_modding_host_measure_text(font, &s);
}

bool cuo_resolve_serial(uint32_t serial, cuo_entity *out)
{
    cuo_modding_host_entity_t e;
    if (!cuo_modding_host_resolve_serial(serial, &e))
        return false;
    if (out)
        *out = e;
    return true;
}

bool cuo_game_viewport(float *x, float *y, float *w, float *h)
{
    cuo_modding_host_screen_rect_t r;
    if (!cuo_modding_host_game_viewport(&r))
        return false;
    if (x) *x = r.x;
    if (y) *y = r.y;
    if (w) *w = r.width;
    if (h) *h = r.height;
    return true;
}

static bool point_out(bool ok, const cuo_modding_host_screen_point_t *p, float *sx, float *sy)
{
    if (!ok)
        return false;
    if (sx) *sx = p->x;
    if (sy) *sy = p->y;
    return true;
}

bool cuo_world_to_screen(uint16_t x, uint16_t y, int8_t z, float *sx, float *sy)
{
    cuo_modding_host_screen_point_t p;
    return point_out(cuo_modding_host_world_to_screen(x, y, z, &p), &p, sx, sy);
}

bool cuo_entity_screen_position(cuo_entity e, float *sx, float *sy)
{
    cuo_modding_host_screen_point_t p;
    return point_out(cuo_modding_host_entity_screen_position(e, &p), &p, sx, sy);
}

const char *cuo_storage_get(cuo_scope scope)
{
    cuo_wit_string_t s;
    cuo_modding_host_storage_get((cuo_modding_host_scope_t)scope, &s);
    return take_string(&s);
}

void cuo_storage_set(cuo_scope scope, const char *blob)
{
    cuo_wit_string_t s = cuo__wstr(blob);
    cuo_modding_host_storage_set((cuo_modding_host_scope_t)scope, &s);
}

/* ── host: pages ─────────────────────────────────────────────────────────── */

_Static_assert(CUO_PAGE_TEXT == CUO_MODDING_HOST_PAGE_CHANGE_TEXT && CUO_PAGE_HTML == CUO_MODDING_HOST_PAGE_CHANGE_HTML
                   && CUO_PAGE_STYLE == CUO_MODDING_HOST_PAGE_CHANGE_STYLE && CUO_PAGE_ATTR == CUO_MODDING_HOST_PAGE_CHANGE_ATTR,
               "cuo_page_change_kind follows the WIT's page-change cases");

static bool entity_out(bool ok, cuo_modding_host_entity_t e, cuo_entity *out)
{
    if (ok && out)
        *out = e;
    return ok;
}

/* The import only reads the change: its strings stay the caller's. */
static cuo_modding_host_page_change_t lower_change(cuo_page_change c)
{
    cuo_modding_host_page_change_t w;
    w.tag = (uint8_t)c.kind;
    if (c.kind == CUO_PAGE_STYLE || c.kind == CUO_PAGE_ATTR) {
        w.val.style.f0 = cuo__wstr(c.a);
        w.val.style.f1 = cuo__wstr(c.b);
    } else {
        w.val.text = cuo__wstr(c.a);
    }
    return w;
}

bool cuo_page_open(const char *name, const char *html, const char *css, cuo_entity *page)
{
    cuo_wit_string_t n = cuo__wstr(name), h = cuo__wstr(html), c = cuo__wstr(css);
    cuo_modding_host_entity_t e = 0;
    return entity_out(cuo_modding_host_page_open(&n, &h, &c, &e), e, page);
}

bool cuo_page_spawn(const char *name, const char *html, const char *css, int32_t x, int32_t y, cuo_entity *page)
{
    cuo_wit_string_t n = cuo__wstr(name), h = cuo__wstr(html), c = cuo__wstr(css);
    cuo_modding_host_entity_t e = 0;
    return entity_out(cuo_modding_host_page_spawn(&n, &h, &c, x, y, &e), e, page);
}

void cuo_page_edit(cuo_entity page, const char *id, cuo_page_change change)
{
    cuo_wit_string_t i = cuo__wstr(id);
    cuo_modding_host_page_change_t w = lower_change(change);
    cuo_modding_host_page_edit(page, &i, &w);
}

const char *cuo_page_value(cuo_entity page, const char *id)
{
    cuo_wit_string_t i = cuo__wstr(id), s;
    return cuo_modding_host_page_value(page, &i, &s) ? take_string(&s) : NULL;
}

size_t cuo_page_clicks(cuo_entity page, const char *const **args)
{
    cuo_wit_list_string_t l;
    cuo_modding_host_page_clicks(page, &l);
    size_t n = l.len;
    if (args) {
        const char **out = cuo_alloc(n * sizeof *out);
        for (size_t k = 0; k < n; k++)
            out[k] = cuo_strndup((const char *)l.ptr[k].ptr, l.ptr[k].len);
        *args = out;
    }
    cuo_wit_list_string_free(&l);
    return n;
}

bool cuo_page_is_open(cuo_entity page)
{
    return cuo_modding_host_page_is_open(page);
}

void cuo_page_close(cuo_entity page)
{
    cuo_modding_host_page_close(page);
}

bool cuo_page_element(cuo_entity page, const char *id, cuo_entity *out)
{
    cuo_wit_string_t i = cuo__wstr(id);
    cuo_modding_host_entity_t e = 0;
    return entity_out(cuo_modding_host_page_element(page, &i, &e), e, out);
}

bool cuo_page_clone(cuo_entity page, const char *container, const char *template_id, cuo_entity *out)
{
    cuo_wit_string_t c = cuo__wstr(container), t = cuo__wstr(template_id);
    cuo_modding_host_entity_t e = 0;
    return entity_out(cuo_modding_host_page_clone(page, &c, &t, &e), e, out);
}

void cuo_page_clear(cuo_entity page, const char *container)
{
    cuo_wit_string_t c = cuo__wstr(container);
    cuo_modding_host_page_clear(page, &c);
}

bool cuo_page_part(cuo_entity page, cuo_entity element, const char *part, cuo_entity *out)
{
    cuo_wit_string_t p = cuo__wstr(part);
    cuo_modding_host_entity_t e = 0;
    return entity_out(cuo_modding_host_page_part(page, element, &p, &e), e, out);
}

void cuo_page_edit_element(cuo_entity page, cuo_entity element, cuo_page_change change)
{
    cuo_modding_host_page_change_t w = lower_change(change);
    cuo_modding_host_page_edit_element(page, element, &w);
}

/* ── packets ─────────────────────────────────────────────────────────────── */

void cuo_send_to_server(const uint8_t *data, size_t len)
{
    cuo_wit_list_u8_t l = { (uint8_t *)data, len };
    cuo_modding_packets_send_to_server(&l);
}

void cuo_send_to_client(const uint8_t *data, size_t len)
{
    cuo_wit_list_u8_t l = { (uint8_t *)data, len };
    cuo_modding_packets_send_to_client(&l);
}

/* ── assets ──────────────────────────────────────────────────────────────── */

bool cuo_gump_size(uint32_t gump_id, int *w, int *h)
{
    cuo_modding_assets_size_t s;
    if (!cuo_modding_assets_image_size(CUO_MODDING_ASSETS_IMAGE_KIND_GUMP, gump_id, &s))
        return false;
    if (w)
        *w = (int)s.width;
    if (h)
        *h = (int)s.height;
    return true;
}

uint32_t cuo_hue_argb(uint16_t hue)
{
    if (hue == 0)
        return 0xFFFFFFFFu;
    /* The ramp runs dark to light; a white glyph takes the lightest entry. */
    cuo_wit_list_u32_t ramp;
    if (!cuo_modding_assets_hue_ramp(hue, &ramp))
        return 0xFFFFFFFFu;
    uint32_t argb = ramp.len ? ramp.ptr[ramp.len - 1] : 0xFFFFFFFFu;
    cuo_wit_list_u32_free(&ramp);
    return argb;
}

const char *cuo_cliloc(uint32_t id)
{
    cuo_wit_list_string_t args = { NULL, 0 };
    cuo_wit_string_t s;
    cuo_modding_assets_cliloc(id, &args, &s);
    return take_string(&s);
}

/* ── assets: the map ─────────────────────────────────────────────────────── */

static void *scratch_copy(const void *src, size_t bytes)
{
    void *d = cuo_alloc(bytes);
    if (bytes)
        memcpy(d, src, bytes);
    return d;
}

bool cuo_map_load(uint8_t map, uint32_t *w, uint32_t *h)
{
    cuo_modding_assets_size_t s;
    if (!cuo_modding_assets_map_load(map, &s))
        return false;
    if (w) *w = s.width;
    if (h) *h = s.height;
    return true;
}

bool cuo_map_land_area(uint8_t map, cuo_map_area area, cuo_land_area *out)
{
    cuo_modding_assets_map_area_t a = { area.x, area.y, area.width, area.height };
    cuo_modding_assets_land_area_t r;
    if (!cuo_modding_assets_map_land_area(map, &a, &r))
        return false;
    if (out) {
        out->area = (cuo_map_area){ r.area.x, r.area.y, r.area.width, r.area.height };
        out->graphics = scratch_copy(r.graphics.ptr, r.graphics.len * sizeof(uint16_t));
        out->z = scratch_copy(r.z.ptr, r.z.len);
    }
    cuo_modding_assets_land_area_free(&r);
    return true;
}

size_t cuo_map_statics_area(uint8_t map, cuo_map_area area, const cuo_placed_static **out)
{
    cuo_modding_assets_map_area_t a = { area.x, area.y, area.width, area.height };
    cuo_modding_assets_list_placed_static_t l;
    cuo_modding_assets_map_statics_area(map, &a, &l);
    size_t n = l.len;
    if (out) {
        cuo_placed_static *d = cuo_alloc(n * sizeof *d);
        for (size_t k = 0; k < n; k++)
            d[k] = (cuo_placed_static){ l.ptr[k].x, l.ptr[k].y, l.ptr[k].z, l.ptr[k].graphic, l.ptr[k].hue };
        *out = d;
    }
    cuo_modding_assets_list_placed_static_free(&l);
    return n;
}

bool cuo_land_radar_color(uint16_t id, uint32_t *argb)
{
    uint32_t c;
    if (!cuo_modding_assets_land_radar_color(id, &c))
        return false;
    if (argb) *argb = c;
    return true;
}

bool cuo_static_radar_color(uint16_t graphic, uint16_t hue, uint32_t *argb)
{
    uint32_t c;
    if (!cuo_modding_assets_static_radar_color(graphic, hue, &c))
        return false;
    if (argb) *argb = c;
    return true;
}

uint32_t cuo_map_revision(void)
{
    return cuo_modding_assets_map_revision();
}
