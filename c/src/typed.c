/* Components, resources and events through cuo:modding/components: every value crosses
 * as a typed WIT record. The author-facing structs are the generated cuo_X ones (shared
 * with the UI helpers). This file is the generic half; the per-type half — the cuo_X <->
 * wit-bindgen record converters, every cuo_X_comp / _column / _set / _res / _put /
 * _set_resource / _send / _events / _obs_value — is generated into typed_gen.c
 * (tools/mod-typegen, EmitSdkC). */
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
        cuo__trap(cuo_fmt("cuo: '%s' recorded a command but its export in wit/world.wit takes no "
                          "`commands: commands`",
                          c->system));
    c->count++;
    return (cuo_modding_components_borrow_commands_t){ c->handle.__handle };
}

/* Into scratch: a typed insert is queued before the builder chain runs. */
cuo_wit_string_t cuo__wstr_copy(const char *s)
{
    return cuo__wstr(cuo_strdup(s ? s : ""));
}

/* A WIT char <-> the one-character UTF-8 string of the cuo_X struct. */
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

/* An insert: `rec` is the record, already converted (scratch, valid for the call). */
cuo_comp cuo__comp(uint16_t type_id, uint8_t kind, const void *rec)
{
    cuo_comp c = { type_id, kind, rec };
    return c;
}

/* Each builder method queues one insert and hands back a new builder for the same
 * entity; the old one is dropped. */
void cuo__eb_push(cuo__own_eb b, const cuo_comp *comps, size_t n)
{
    for (size_t i = 0; i < n; i++) {
        cuo__own_eb next = cuo__typed_push(cuo_modding_components_borrow_entity_builder(b), &comps[i]);
        cuo_modding_components_entity_builder_drop_own(b);
        b = next;
    }
    cuo_modding_components_entity_builder_drop_own(b);
}
