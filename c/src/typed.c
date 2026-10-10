/* Typed components: the curated types through cuo:modding/components (no JSON). The
 * author-facing structs stay the generated cuo_X ones (shared with the JSON path and the
 * UI helpers). This file is the generic half; the per-type half — which paths are typed,
 * the cuo_X <-> wit-bindgen record converters, every cuo_X_typed / _column / _set /
 * _res / _send / _events / _obs_typed — is generated into typed_gen.c (tools/mod-typegen,
 * EmitSdkC). */
#include <string.h>

#include "internal.h"

cuo_modding_components_borrow_query_t cuo__typed_query(cuo_query q)
{
    if (q.handle < 0)
        cuo__trap("cuo: a typed column / set on an absent query");
    return (cuo_modding_components_borrow_query_t){ q.handle };
}

cuo_modding_components_borrow_commands_t cuo__typed_cmds(cuo_cmds *c)
{
    if (c->handle.__handle < 0)
        cuo__trap(cuo_fmt("cuo: '%s' sent a typed event but its export takes no `commands`", c->system));
    return (cuo_modding_components_borrow_commands_t){ c->handle.__handle };
}

/* Into scratch: a typed insert is queued before the builder chain runs. */
cuo_wit_string_t cuo__wstr_copy(const char *s)
{
    return cuo__wstr(cuo_strdup(s ? s : ""));
}

/* A WIT char <-> the one-character string the JSON path carries (UTF-8). */
uint32_t cuo__char_of(const char *s)
{
    const unsigned char *p = (const unsigned char *)(s ? s : "");
    if (p[0] < 0x80)
        return p[0];
    if ((p[0] & 0xE0) == 0xC0 && p[1])
        return (uint32_t)(p[0] & 0x1F) << 6 | (p[1] & 0x3F);
    if ((p[0] & 0xF0) == 0xE0 && p[1] && p[2])
        return (uint32_t)(p[0] & 0x0F) << 12 | (uint32_t)(p[1] & 0x3F) << 6 | (p[2] & 0x3F);
    if ((p[0] & 0xF8) == 0xF0 && p[1] && p[2] && p[3])
        return (uint32_t)(p[0] & 0x07) << 18 | (uint32_t)(p[1] & 0x3F) << 12 | (uint32_t)(p[2] & 0x3F) << 6 |
               (p[3] & 0x3F);
    return 0;
}

const char *cuo__char_str(uint32_t ch)
{
    char *s = cuo_alloc(5);
    if (ch < 0x80) {
        s[0] = (char)ch;
    } else if (ch < 0x800) {
        s[0] = (char)(0xC0 | ch >> 6);
        s[1] = (char)(0x80 | (ch & 0x3F));
    } else if (ch < 0x10000) {
        s[0] = (char)(0xE0 | ch >> 12);
        s[1] = (char)(0x80 | (ch >> 6 & 0x3F));
        s[2] = (char)(0x80 | (ch & 0x3F));
    } else {
        s[0] = (char)(0xF0 | ch >> 18);
        s[1] = (char)(0x80 | (ch >> 12 & 0x3F));
        s[2] = (char)(0x80 | (ch >> 6 & 0x3F));
        s[3] = (char)(0x80 | (ch & 0x3F));
    }
    return s;
}

/* A typed insert: `w` is the record, already converted (scratch, valid for the call). */
cuo_comp cuo__typed_comp(uint16_t type_id, uint8_t kind, const void *w, size_t size)
{
    cuo_comp c = { type_id, { w, size }, kind };
    return c;
}

/* ── the builder chain behind cuo_spawn / cuo_insert ─────────────────────── */

int32_t cuo__eb_spawn(tinyecs_modding_ecs_borrow_commands_t c, cuo_entity *e)
{
    cuo__own_eb b = cuo_modding_components_spawn((cuo_modding_components_borrow_commands_t){ c.__handle });
    *e = cuo_modding_components_method_entity_builder_id(cuo_modding_components_borrow_entity_builder(b));
    return b.__handle;
}

int32_t cuo__eb_of(tinyecs_modding_ecs_borrow_commands_t c, cuo_entity e)
{
    return cuo_modding_components_entity_of((cuo_modding_components_borrow_commands_t){ c.__handle }, e).__handle;
}

/* Each builder method queues one insert and hands back a new builder for the same
 * entity; the old one is dropped. */
void cuo__eb_push(int32_t builder, const cuo_comp *comps, size_t n)
{
    cuo__own_eb b = { builder };
    for (size_t i = 0; i < n; i++)
        if (comps[i].typed) {
            cuo__own_eb next = cuo__typed_push(cuo_modding_components_borrow_entity_builder(b), &comps[i]);
            cuo_modding_components_entity_builder_drop_own(b);
            b = next;
        }
    cuo_modding_components_entity_builder_drop_own(b);
}
