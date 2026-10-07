/* The cuo:modding host functions (wit/cuo-mod.wit) over the single env.mod_call
 * import: name "<package>/<interface>#<function>", args a JSON array in WIT order,
 * result JSON written into the ABI arena (docs/p1-wire.md). */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "internal.h"

__attribute__((import_module("env"), import_name("mod_call"))) uint64_t
cuo__imp_mod_call(const char *name, uint32_t name_len, const char *args, uint32_t args_len);

cuo_bytes cuo_call(const char *name, const char *args_json)
{
    cuo_bytes none = { NULL, 0 };
    if (!args_json)
        args_json = "[]";
    uint64_t packed = cuo__imp_mod_call(name, (uint32_t)strlen(name), args_json, (uint32_t)strlen(args_json));
    if (packed == 0)
        return none;
    /* The result is in the ABI arena; copy it out before anything grows it. */
    size_t len = (size_t)(packed >> 32);
    char *copy = cuo_strndup((const char *)(uintptr_t)(uint32_t)packed, len);
    cuo_bytes r = { (const uint8_t *)copy, len };
    return r;
}

void cuo_action(const char *fn, const char *args_json)
{
    cuo_call(cuo_fmt("cuo:modding/actions#%s", fn), args_json);
}

/* JSON array of one string. */
static const char *str_arg(const char *s)
{
    cuo_jw w;
    cuo_jw_init(&w);
    cuo_jw_arr(&w);
    cuo_jw_str(&w, s);
    cuo_jw_arr_end(&w);
    return (const char *)cuo_jw_bytes(&w).ptr;
}

/* ── host ────────────────────────────────────────────────────────────────── */

void cuo_log(const char *msg)
{
    cuo_call("cuo:modding/host#log", str_arg(msg));
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
    cuo_jw w;
    cuo_jw_init(&w);
    cuo_jw_arr(&w);
    cuo_jw_uint(&w, font);
    cuo_jw_str(&w, text);
    cuo_jw_arr_end(&w);
    cJSON *r = cuo_json_parse(cuo_call("cuo:modding/host#measure-text", (const char *)cuo_jw_bytes(&w).ptr));
    return r && cJSON_IsNumber(r) ? (uint32_t)r->valuedouble : 0;
}

bool cuo_resolve_serial(uint32_t serial, cuo_entity *out)
{
    cJSON *r = cuo_json_parse(cuo_call("cuo:modding/host#resolve-serial", cuo_fmt("[%u]", serial)));
    /* option<entity>: null or a u64 as a decimal string. */
    if (!r || !cJSON_IsString(r))
        return false;
    if (out)
        *out = strtoull(r->valuestring, NULL, 10);
    return true;
}

static const char *scope_name(cuo_scope scope)
{
    return scope == CUO_SCOPE_CHARACTER ? "character" : "global";
}

const char *cuo_storage_get(cuo_scope scope)
{
    cJSON *r = cuo_json_parse(cuo_call("cuo:modding/host#storage-get", cuo_fmt("[\"%s\"]", scope_name(scope))));
    return r && cJSON_IsString(r) ? r->valuestring : "";
}

void cuo_storage_set(cuo_scope scope, const char *json)
{
    cuo_jw w;
    cuo_jw_init(&w);
    cuo_jw_arr(&w);
    cuo_jw_str(&w, scope_name(scope));
    cuo_jw_str(&w, json);
    cuo_jw_arr_end(&w);
    cuo_call("cuo:modding/host#storage-set", (const char *)cuo_jw_bytes(&w).ptr);
}

/* ── packets ─────────────────────────────────────────────────────────────── */

static const char B64[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/* `list<u8>` crosses as base64. */
static void jw_base64(cuo_jw *w, const uint8_t *p, size_t n)
{
    char *s = cuo_alloc((n + 2) / 3 * 4 + 1);
    size_t o = 0;
    for (size_t i = 0; i < n; i += 3) {
        uint32_t v = (uint32_t)p[i] << 16 | (i + 1 < n ? (uint32_t)p[i + 1] << 8 : 0) | (i + 2 < n ? p[i + 2] : 0);
        s[o++] = B64[v >> 18 & 63];
        s[o++] = B64[v >> 12 & 63];
        s[o++] = i + 1 < n ? B64[v >> 6 & 63] : '=';
        s[o++] = i + 2 < n ? B64[v & 63] : '=';
    }
    cuo_jw_strn(w, s, o);
}

static const char *dir_name(cuo_dir dir)
{
    return dir == CUO_OUTGOING ? "outgoing" : "incoming";
}

void cuo_intercept(cuo_dir dir, const uint8_t *ids, size_t n)
{
    cuo_jw w;
    cuo_jw_init(&w);
    cuo_jw_arr(&w);
    cuo_jw_str(&w, dir_name(dir));
    jw_base64(&w, ids, n);
    cuo_jw_arr_end(&w);
    cuo_call("cuo:modding/packets#intercept", (const char *)cuo_jw_bytes(&w).ptr);
}

static void send_bytes(const char *fn, const uint8_t *data, size_t len)
{
    cuo_jw w;
    cuo_jw_init(&w);
    cuo_jw_arr(&w);
    jw_base64(&w, data, len);
    cuo_jw_arr_end(&w);
    cuo_call(fn, (const char *)cuo_jw_bytes(&w).ptr);
}

void cuo_send_to_server(const uint8_t *data, size_t len)
{
    send_bytes("cuo:modding/packets#send-to-server", data, len);
}

void cuo_send_to_client(const uint8_t *data, size_t len)
{
    send_bytes("cuo:modding/packets#send-to-client", data, len);
}

/* ── assets ──────────────────────────────────────────────────────────────── */

bool cuo_gump_size(uint32_t gump_id, int *w, int *h)
{
    cJSON *r = cuo_json_parse(cuo_call("cuo:modding/assets#image-size", cuo_fmt("[\"gump\",%u]", gump_id)));
    if (!r || !cJSON_IsObject(r))
        return false;
    if (w)
        *w = (int)cuo_json_int(r, "width", 0);
    if (h)
        *h = (int)cuo_json_int(r, "height", 0);
    return true;
}

uint32_t cuo_hue_argb(uint16_t hue)
{
    if (hue == 0)
        return 0xFFFFFFFFu;
    /* The ramp runs dark to light; a white glyph takes the lightest entry. */
    cJSON *r = cuo_json_parse(cuo_call("cuo:modding/assets#hue-ramp", cuo_fmt("[%u]", hue)));
    int n = r && cJSON_IsArray(r) ? cJSON_GetArraySize(r) : 0;
    return n ? (uint32_t)cJSON_GetArrayItem(r, n - 1)->valuedouble : 0xFFFFFFFFu;
}

const char *cuo_cliloc(uint32_t id)
{
    cJSON *r = cuo_json_parse(cuo_call("cuo:modding/assets#cliloc", cuo_fmt("[%u,[]]", id)));
    return r && cJSON_IsString(r) ? r->valuestring : "";
}

/* ── gone in v3 ──────────────────────────────────────────────────────────── */

cuo_bytes cuo_component_json(uint64_t entity, uint16_t type_id)
{
    (void)entity;
    (void)type_id;
    cuo__trap("cuo: component_get is gone (modding surface v3): read components through a query param");
}

cuo_bytes cuo_resource_json(uint16_t type_id)
{
    (void)type_id;
    cuo__trap("cuo: resource_get is gone (modding surface v3): declare cuo_system_res");
}
