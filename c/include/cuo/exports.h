/* exports.h — SDK-internal: what the generated trampolines (build/gen/cuo_exports.c,
 * written by gen-exports.awk from the mod's world) hand the dispatcher. Not mod API. */
#ifndef CUO_EXPORTS_H
#define CUO_EXPORTS_H

#include <stddef.h>
#include <stdint.h>

#include "cuo_wit.h"

/* An export parameter's kind, from its C type. */
enum {
    CUO__K_UNKNOWN = 0,
    CUO__K_COMMANDS,
    CUO__K_QUERY,
    CUO__K_RES,
    CUO__K_EVENTS,
    CUO__K_TRIGGER, /* observer: trigger-data, first */
    CUO__K_DIR,     /* on-packet observer: packet-direction, first */
    CUO__K_PACKET,  /* on-packet observer: list<u8>, second */
    CUO__K_VERDICT, /* on-packet observer: the -> verdict out-pointer, last */
};

#define CUO__KIND(T)                                                                    \
    _Generic((T){ 0 },                                                                  \
        tinyecs_modding_ecs_own_commands_t: CUO__K_COMMANDS,                            \
        tinyecs_modding_ecs_own_query_t: CUO__K_QUERY,                                  \
        tinyecs_modding_ecs_own_res_t: CUO__K_RES,                                      \
        tinyecs_modding_ecs_own_events_t: CUO__K_EVENTS,                                \
        tinyecs_modding_ecs_trigger_data_t *: CUO__K_TRIGGER,                           \
        tinyecs_modding_ecs_packet_direction_t: CUO__K_DIR,                             \
        cuo_wit_list_u8_t *: CUO__K_PACKET,                                             \
        tinyecs_modding_ecs_verdict_t *: CUO__K_VERDICT,                                \
        default: CUO__K_UNKNOWN)

typedef struct cuo__arg {
    int32_t handle; /* commands / query / res / events */
    void *ptr;      /* trigger / packet / verdict */
    uint8_t dir;
} cuo__arg;

static inline cuo__arg cuo__arg_h(int32_t h) { return (cuo__arg){ .handle = h }; }
static inline cuo__arg cuo__arg_commands(tinyecs_modding_ecs_own_commands_t x) { return cuo__arg_h(x.__handle); }
static inline cuo__arg cuo__arg_query(tinyecs_modding_ecs_own_query_t x) { return cuo__arg_h(x.__handle); }
static inline cuo__arg cuo__arg_res(tinyecs_modding_ecs_own_res_t x) { return cuo__arg_h(x.__handle); }
static inline cuo__arg cuo__arg_events(tinyecs_modding_ecs_own_events_t x) { return cuo__arg_h(x.__handle); }
static inline cuo__arg cuo__arg_ptr(void *p) { return (cuo__arg){ .ptr = p }; }
static inline cuo__arg cuo__arg_dir(tinyecs_modding_ecs_packet_direction_t d) { return (cuo__arg){ .dir = d }; }

#define CUO__ARG(x)                                                                     \
    _Generic((x),                                                                       \
        tinyecs_modding_ecs_own_commands_t: cuo__arg_commands,                          \
        tinyecs_modding_ecs_own_query_t: cuo__arg_query,                                \
        tinyecs_modding_ecs_own_res_t: cuo__arg_res,                                    \
        tinyecs_modding_ecs_own_events_t: cuo__arg_events,                              \
        tinyecs_modding_ecs_packet_direction_t: cuo__arg_dir,                           \
        default: cuo__arg_ptr)(x)

/* One export of the mod's world: its name (kebab-case, as the system is named) and
 * its parameter kinds in order. */
typedef struct cuo__export {
    const char *name;
    const uint8_t *kinds;
    size_t n;
} cuo__export;

extern const cuo__export cuo__exports[];
extern const size_t cuo__nexports;

/* Runs the system / observer declared under x->name with the export's arguments. */
void cuo__dispatch(const cuo__export *x, const cuo__arg *args);

#endif
