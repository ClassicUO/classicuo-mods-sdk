/* Host imports (wasm import module "cuo") + wrappers — twin of rust/src/{imports,storage}.rs.
 * Out-parameter imports return the needed length (0 = absent) and only fill the buffer
 * when it fits; the wrappers retry once with the exact size. Buffers are scratch. */
#include <string.h>

#include "internal.h"

CUO_IMPORT(resolve_serial) uint64_t cuo__imp_resolve_serial(uint32_t serial);
CUO_IMPORT(gump_size) uint32_t cuo__imp_gump_size(uint32_t id);
CUO_IMPORT(hue_color) uint32_t cuo__imp_hue_color(uint32_t hue);
CUO_IMPORT(measure_text) uint32_t cuo__imp_measure_text(uint32_t font, const char *ptr, uint32_t len);
CUO_IMPORT(resolve_cliloc) uint32_t cuo__imp_resolve_cliloc(uint32_t id, void *out, uint32_t cap);
CUO_IMPORT(net_send) void cuo__imp_net_send(const uint8_t *ptr, uint32_t len);
CUO_IMPORT(entity_parent) uint64_t cuo__imp_entity_parent(uint64_t entity);
CUO_IMPORT(entity_children) uint32_t cuo__imp_entity_children(uint64_t entity, uint64_t *out, uint32_t cap);
CUO_IMPORT(component_get) uint32_t cuo__imp_component_get(uint64_t entity, uint32_t type_id, void *out, uint32_t cap);
CUO_IMPORT(resource_get) uint32_t cuo__imp_resource_get(uint32_t type_id, void *out, uint32_t cap);
CUO_IMPORT(storage_get) uint32_t cuo__imp_storage_get(uint32_t arg, void *out, uint32_t cap);
CUO_IMPORT(storage_set) void cuo__imp_storage_set(const char *ptr, uint32_t len);

typedef uint32_t (*read_fn)(uint32_t a, uint64_t b, void *out, uint32_t cap);

/* NUL-terminated scratch copy of a string-returning import; {NULL,0} when absent. */
static cuo_bytes read_out(read_fn call, uint32_t a, uint64_t b, uint32_t cap)
{
    cuo_bytes none = { NULL, 0 };
    for (;;) {
        char *buf = cuo__scratch_grow(NULL, 0, (size_t)cap + 1);
        uint32_t needed = call(a, b, buf, cap);
        if (needed == 0)
            return none;
        if (needed <= cap) {
            buf[needed] = 0;
            cuo_bytes r = { (const uint8_t *)buf, needed };
            return r;
        }
        cap = needed;
    }
}

static uint32_t call_cliloc(uint32_t a, uint64_t b, void *out, uint32_t cap)
{
    (void)b;
    return cuo__imp_resolve_cliloc(a, out, cap);
}

static uint32_t call_component(uint32_t a, uint64_t b, void *out, uint32_t cap)
{
    return cuo__imp_component_get(b, a, out, cap);
}

static uint32_t call_resource(uint32_t a, uint64_t b, void *out, uint32_t cap)
{
    (void)b;
    return cuo__imp_resource_get(a, out, cap);
}

static uint32_t call_storage(uint32_t a, uint64_t b, void *out, uint32_t cap)
{
    (void)b;
    return cuo__imp_storage_get(a, out, cap);
}

uint64_t cuo_resolve_serial(uint32_t serial)
{
    return cuo__imp_resolve_serial(serial);
}

void cuo_gump_size(uint32_t gump_id, int *w, int *h)
{
    uint32_t packed = cuo__imp_gump_size(gump_id);
    if (w)
        *w = (int)(packed >> 16);
    if (h)
        *h = (int)(packed & 0xFFFF);
}

uint32_t cuo_hue_argb(uint32_t hue)
{
    return cuo__imp_hue_color(hue);
}

int cuo_measure_text(uint32_t font, const char *text)
{
    return (int)cuo__imp_measure_text(font, text, (uint32_t)strlen(text));
}

const char *cuo_cliloc(uint32_t id)
{
    cuo_bytes b = read_out(call_cliloc, id, 0, 256);
    return b.ptr ? (const char *)b.ptr : "";
}

void cuo_net_send(const uint8_t *data, size_t len)
{
    cuo__imp_net_send(data, (uint32_t)len);
}

uint64_t cuo_entity_parent(uint64_t entity)
{
    return cuo__imp_entity_parent(entity);
}

size_t cuo_entity_children(uint64_t entity, const uint64_t **out)
{
    uint32_t cap = 16;
    for (;;) {
        uint64_t *buf = cuo_alloc(cap * sizeof(uint64_t));
        uint32_t n = cuo__imp_entity_children(entity, buf, cap);
        if (n <= cap) {
            if (out)
                *out = buf;
            return n;
        }
        cap = n;
    }
}

cuo_bytes cuo_component_json(uint64_t entity, uint16_t type_id)
{
    return read_out(call_component, type_id, entity, 256);
}

cuo_bytes cuo_resource_json(uint16_t type_id)
{
    return read_out(call_resource, type_id, 0, 256);
}

const char *cuo_storage_get(void)
{
    cuo_bytes b = read_out(call_storage, 0, 0, 512);
    return b.ptr ? (const char *)b.ptr : "";
}

void cuo_storage_set_bytes(cuo_bytes json)
{
    cuo__imp_storage_set((const char *)json.ptr, (uint32_t)json.len);
}

void cuo_storage_set(const char *json)
{
    cuo_storage_set_bytes(cuo_str_bytes(json));
}
