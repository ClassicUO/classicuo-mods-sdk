/* JSON: a streaming writer into scratch, and cJSON parsing into call-scoped trees.
 * The host side is System.Text.Json (no naming policy, enums as numbers). */
#include <math.h>
#include <stdio.h>
#include <string.h>

#include "internal.h"

static void put(cuo_jw *w, const char *s, size_t n)
{
    if (w->len + n + 1 > w->cap) {
        size_t cap = w->cap ? w->cap * 2 : 256;
        while (cap < w->len + n + 1)
            cap *= 2;
        w->buf = cuo__scratch_grow(w->buf, w->len, cap);
        w->cap = cap;
    }
    memcpy(w->buf + w->len, s, n);
    w->len += n;
    w->buf[w->len] = 0;
}

static void putc_(cuo_jw *w, char c)
{
    put(w, &c, 1);
}

/* Separator bookkeeping: a value right after a key never takes a comma. */
static void pre_value(cuo_jw *w)
{
    if (w->after_key) {
        w->after_key = false;
        return;
    }
    uint64_t bit = 1ull << (w->depth & 63);
    if (w->has_item & bit)
        putc_(w, ',');
    w->has_item |= bit;
}

void cuo_jw_init(cuo_jw *w)
{
    memset(w, 0, sizeof *w);
}

static void open_(cuo_jw *w, char c)
{
    pre_value(w);
    putc_(w, c);
    w->depth++;
    w->has_item &= ~(1ull << (w->depth & 63));
}

static void close_(cuo_jw *w, char c)
{
    w->depth--;
    putc_(w, c);
}

void cuo_jw_obj(cuo_jw *w) { open_(w, '{'); }
void cuo_jw_obj_end(cuo_jw *w) { close_(w, '}'); }
void cuo_jw_arr(cuo_jw *w) { open_(w, '['); }
void cuo_jw_arr_end(cuo_jw *w) { close_(w, ']'); }

static void str_body(cuo_jw *w, const char *s, size_t n)
{
    static const char hex[] = "0123456789abcdef";
    putc_(w, '"');
    size_t run = 0;
    for (size_t i = 0; i < n; i++) {
        unsigned char ch = (unsigned char)s[i];
        if (ch >= 0x20 && ch != '"' && ch != '\\')
            continue;
        put(w, s + run, i - run);
        run = i + 1;
        char esc[6] = { '\\', 0 };
        switch (ch) {
        case '"': esc[1] = '"'; put(w, esc, 2); break;
        case '\\': esc[1] = '\\'; put(w, esc, 2); break;
        case '\n': esc[1] = 'n'; put(w, esc, 2); break;
        case '\r': esc[1] = 'r'; put(w, esc, 2); break;
        case '\t': esc[1] = 't'; put(w, esc, 2); break;
        default:
            esc[1] = 'u'; esc[2] = '0'; esc[3] = '0';
            esc[4] = hex[ch >> 4]; esc[5] = hex[ch & 15];
            put(w, esc, 6);
        }
    }
    put(w, s + run, n - run);
    putc_(w, '"');
}

void cuo_jw_key(cuo_jw *w, const char *key)
{
    pre_value(w);
    str_body(w, key, strlen(key));
    putc_(w, ':');
    w->after_key = true;
}

void cuo_jw_strn(cuo_jw *w, const char *s, size_t n)
{
    pre_value(w);
    str_body(w, s ? s : "", s ? n : 0);
}

void cuo_jw_str(cuo_jw *w, const char *s)
{
    cuo_jw_strn(w, s, s ? strlen(s) : 0);
}

void cuo_jw_int(cuo_jw *w, int64_t v)
{
    char b[24];
    pre_value(w);
    put(w, b, (size_t)snprintf(b, sizeof b, "%lld", (long long)v));
}

void cuo_jw_uint(cuo_jw *w, uint64_t v)
{
    char b[24];
    pre_value(w);
    put(w, b, (size_t)snprintf(b, sizeof b, "%llu", (unsigned long long)v));
}

void cuo_jw_num(cuo_jw *w, double v)
{
    /* JSON has no NaN/Infinity and STJ rejects them. */
    if (!isfinite(v))
        v = 0;
    if (fabs(v) < 1e15 && v == (double)(int64_t)v) {
        cuo_jw_int(w, (int64_t)v);
        return;
    }
    char b[32];
    pre_value(w);
    put(w, b, (size_t)snprintf(b, sizeof b, "%.17g", v));
}

void cuo_jw_bool(cuo_jw *w, bool v)
{
    pre_value(w);
    put(w, v ? "true" : "false", v ? 4 : 5);
}

void cuo_jw_null(cuo_jw *w)
{
    pre_value(w);
    put(w, "null", 4);
}

void cuo_jw_raw(cuo_jw *w, const char *json)
{
    pre_value(w);
    put(w, json, strlen(json));
}

cuo_bytes cuo_jw_bytes(cuo_jw *w)
{
    if (!w->buf)
        put(w, "", 0);
    cuo_bytes b = { (const uint8_t *)w->buf, w->len };
    return b;
}

/* ── parsing ─────────────────────────────────────────────────────────────── */

static void *scratch_malloc(size_t n)
{
    return cuo__scratch_grow(NULL, 0, n);
}

static void scratch_free(void *p)
{
    (void)p;
}

cJSON *cuo_json_parse(cuo_bytes json)
{
    if (!json.ptr || !json.len)
        return NULL;
    cJSON_Hooks hooks = { scratch_malloc, scratch_free };
    cJSON_InitHooks(&hooks);
    cJSON *root = cJSON_ParseWithLength((const char *)json.ptr, json.len);
    cJSON_InitHooks(NULL);
    return root;
}

int64_t cuo_json_int(const cJSON *obj, const char *key, int64_t dflt)
{
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    return cJSON_IsNumber(v) ? (int64_t)v->valuedouble : dflt;
}

double cuo_json_num(const cJSON *obj, const char *key, double dflt)
{
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    return cJSON_IsNumber(v) ? v->valuedouble : dflt;
}

bool cuo_json_bool(const cJSON *obj, const char *key, bool dflt)
{
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    return cJSON_IsBool(v) ? cJSON_IsTrue(v) : dflt;
}

const char *cuo_json_str(const cJSON *obj, const char *key, const char *dflt)
{
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    return cJSON_IsString(v) ? v->valuestring : dflt;
}
