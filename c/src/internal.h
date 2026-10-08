/* Shared between the SDK's translation units. Not part of the mod-facing API. */
#ifndef CUO_INTERNAL_H
#define CUO_INTERNAL_H

#include "cuo/cuo.h"

#include "mod_abi_reader.h"
#include "mod_abi_builder.h"

_Noreturn void cuo__trap(const char *msg);

/* call-scoped scratch */
void cuo__scratch_reset(void);
void *cuo__scratch_grow(void *p, size_t old_n, size_t new_n);

/* string -> u64 map (keys are copied) */
typedef struct cuo__map {
    char **keys;
    uint64_t *vals;
    size_t cap, count, used; /* used = live + tombstones */
} cuo__map;

bool cuo__map_get(const cuo__map *m, const char *key, uint64_t *out);
void cuo__map_put(cuo__map *m, const char *key, uint64_t val);
bool cuo__map_remove(cuo__map *m, const char *key, uint64_t *out);

/* The one builder shared by the setup reply and every command buffer. */
flatcc_builder_t *cuo__builder(void);

/* Copy the finished buffer into the ABI arena; returns the packed len<<32 | ptr. */
uint64_t cuo__pack_builder(flatcc_builder_t *B);

/* Command buffer plumbing for the dispatcher. */
void cuo__cmds_begin(cuo_cmds *c);
/* 0 when empty and CUO_PASS. `replacement` is read only for CUO_REPLACE. */
uint64_t cuo__cmds_finish(cuo_cmds *c, cuo_verdict verdict, cuo_bytes replacement);
cuo_cmds *cuo__cmds_instance(void);

struct cuo_input {
    ModAbi_SystemInput_table_t t;
};

struct cuo_obs {
    ModAbi_ObserverInput_table_t t;
    uint64_t entity;
};

/* Called by the dispatcher once setup returned, before building the reply. */
void cuo__publish_hotkeys(cuo_builder *m);

#endif
