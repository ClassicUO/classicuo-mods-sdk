# Build recipe for a C mod. A mod is a directory with its sources, its world in
# wit/world.wit, and a Makefile:
#
#   CUO_SDK := ../path/to/classicuo-mods-sdk
#   SRCS    := mod.c
#   include $(CUO_SDK)/c/mod.mk
#
# `make` -> $(OUT) (default build/mod.wasm): a wasm32-wasip2 COMPONENT of the mod's
# world, -Oz, LTO, stripped. The steps:
#   1. build/wit = wit/world.wit + the contract it includes (../wit, c/wit) as deps/
#   2. wit-bindgen c -> build/gen/cuo_wit.{c,h} + the component-type object
#   3. gen-exports.awk -> build/gen/cuo_exports.c (one trampoline per system export)
#   4. wasi-sdk clang links SRCS + the SDK + the bindings (wasm-component-ld)
# Needs wasi-sdk >= 22 (WASI_SDK or WASI_SDK_PATH = its install dir) and
# wit-bindgen-cli 0.57 (`cargo install wit-bindgen-cli --version ^0.57`; WIT_BINDGEN to
# override). Extra knobs: CFLAGS_EXTRA, LDFLAGS_EXTRA; `make DEBUG=1` keeps names +
# asserts.

WASI_SDK    ?= $(subst \,/,$(or $(WASI_SDK_PATH),/opt/wasi-sdk))
CC          := $(WASI_SDK)/bin/clang
WIT_BINDGEN ?= wit-bindgen
AWK         ?= awk
OUT         ?= build/mod.wasm
WORLD_WIT   ?= wit/world.wit

CUO_C    := $(CUO_SDK)/c
CUO_WIT  := $(CUO_SDK)/wit
GEN      := $(dir $(OUT))gen
WITDIR   := $(dir $(OUT))wit

CUO_WIT_SRCS := $(CUO_WIT)/cuo-mod.wit $(CUO_WIT)/deps/tinyecs-mod/tinyecs-mod.wit $(CUO_C)/wit/cuo-c-sdk.wit
CUO_SDK_SRCS := \
	$(CUO_C)/src/runtime.c $(CUO_C)/src/cmds.c $(CUO_C)/src/host.c \
	$(CUO_C)/src/json.c $(CUO_C)/src/ui.c $(CUO_C)/src/types.c \
	$(CUO_C)/third_party/cjson/cJSON.c
CUO_SDK_HDRS := $(wildcard $(CUO_C)/include/cuo/*.h $(CUO_C)/src/*.h)
GEN_SRCS     := $(GEN)/cuo_wit.c $(GEN)/cuo_exports.c
GEN_OBJS     := $(GEN)/cuo_wit_component_type.o

CUO_INCLUDES := -I$(CUO_C)/include -I$(GEN) -I$(CUO_C)/third_party/cjson

ifdef DEBUG
CUO_OPT := -O1 -g
CUO_STRIP :=
else
CUO_OPT := -Oz -flto -DNDEBUG
CUO_STRIP := -Wl,--strip-all
endif

CUO_CFLAGS := --target=wasm32-wasip2 -mexec-model=reactor -std=c11 $(CUO_OPT) \
	-Wall -Wno-unused-function $(CUO_INCLUDES) $(CFLAGS_EXTRA)
CUO_LDFLAGS := -Wl,--gc-sections $(CUO_STRIP) $(LDFLAGS_EXTRA)

.PHONY: all clean
all: $(OUT)

# The world's own package is the root of build/wit, so wit-bindgen picks its only
# world; --rename-world keeps the C names (cuo_wit_*, exports_cuo_wit_*) the same for
# every mod.
$(GEN)/cuo_wit.h: $(WORLD_WIT) $(CUO_WIT_SRCS)
	@rm -rf $(WITDIR) $(GEN)
	@mkdir -p $(WITDIR)/deps/cuo-mod $(WITDIR)/deps/tinyecs-mod $(WITDIR)/deps/cuo-c-sdk
	cp $(WORLD_WIT) $(WITDIR)/world.wit
	cp $(CUO_WIT)/cuo-mod.wit $(WITDIR)/deps/cuo-mod/
	cp $(CUO_WIT)/deps/tinyecs-mod/tinyecs-mod.wit $(WITDIR)/deps/tinyecs-mod/
	cp $(CUO_C)/wit/cuo-c-sdk.wit $(WITDIR)/deps/cuo-c-sdk/
	$(WIT_BINDGEN) c $(WITDIR) --rename-world cuo_wit --out-dir $(GEN)

$(GEN)/cuo_wit.c $(GEN_OBJS): $(GEN)/cuo_wit.h

$(GEN)/cuo_exports.c: $(GEN)/cuo_wit.h $(CUO_C)/gen-exports.awk
	$(AWK) -f $(CUO_C)/gen-exports.awk $< > $@

$(OUT): $(SRCS) $(CUO_SDK_SRCS) $(CUO_SDK_HDRS) $(GEN_SRCS) $(GEN_OBJS)
	@mkdir -p $(dir $@)
	$(CC) $(CUO_CFLAGS) $(SRCS) $(CUO_SDK_SRCS) $(GEN_SRCS) $(GEN_OBJS) -o $@ $(CUO_LDFLAGS)

clean:
	rm -rf $(dir $(OUT))
