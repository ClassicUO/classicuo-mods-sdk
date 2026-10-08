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

const char *cuo_storage_get(cuo_scope scope)
{
    cuo_wit_string_t s;
    cuo_modding_host_storage_get((cuo_modding_host_scope_t)scope, &s);
    return take_string(&s);
}

void cuo_storage_set(cuo_scope scope, const char *json)
{
    cuo_wit_string_t s = cuo__wstr(json);
    cuo_modding_host_storage_set((cuo_modding_host_scope_t)scope, &s);
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

/* ── gone ────────────────────────────────────────────────────────────────── */

cuo_bytes cuo_component_json(uint64_t entity, uint16_t type_id)
{
    (void)entity;
    (void)type_id;
    cuo__trap("cuo: component_get is gone: read components through a query param");
}

cuo_bytes cuo_resource_json(uint16_t type_id)
{
    (void)type_id;
    cuo__trap("cuo: resource_get is gone: declare cuo_system_res");
}
