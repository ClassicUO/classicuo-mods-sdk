# Build recipe for a C mod. A mod's Makefile is:
#
#   CUO_SDK := ../path/to/classicuo-mods-sdk
#   SRCS    := mod.c
#   include $(CUO_SDK)/c/mod.mk
#
# `make` -> $(OUT) (default build/mod.wasm): a wasm32-wasip2 COMPONENT of the
# cuo:modding/mod world (wit-bindgen C bindings in c/generated, linked by wasi-sdk's
# wasm-component-ld), -Oz, LTO, stripped. Needs wasi-sdk >= 22: WASI_SDK (or
# WASI_SDK_PATH) = its install dir. Extra knobs: CFLAGS_EXTRA, LDFLAGS_EXTRA;
# `make DEBUG=1` keeps names + asserts.

WASI_SDK ?= $(subst \,/,$(or $(WASI_SDK_PATH),/opt/wasi-sdk))
CC       := $(WASI_SDK)/bin/clang
OUT      ?= build/mod.wasm

CUO_C    := $(CUO_SDK)/c
CUO_SDK_SRCS := \
	$(CUO_C)/src/runtime.c $(CUO_C)/src/cmds.c $(CUO_C)/src/host.c \
	$(CUO_C)/src/json.c $(CUO_C)/src/ui.c $(CUO_C)/src/types.c \
	$(CUO_C)/generated/cuo_wit.c \
	$(CUO_C)/third_party/cjson/cJSON.c
CUO_SDK_OBJS := $(CUO_C)/generated/cuo_wit_component_type.o
CUO_SDK_HDRS := $(wildcard $(CUO_C)/include/cuo/*.h $(CUO_C)/src/*.h $(CUO_C)/generated/*.h)

CUO_INCLUDES := -I$(CUO_C)/include -I$(CUO_C)/generated -I$(CUO_C)/third_party/cjson

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

$(OUT): $(SRCS) $(CUO_SDK_SRCS) $(CUO_SDK_OBJS) $(CUO_SDK_HDRS)
	@mkdir -p $(dir $@)
	$(CC) $(CUO_CFLAGS) $(SRCS) $(CUO_SDK_SRCS) $(CUO_SDK_OBJS) -o $@ $(CUO_LDFLAGS)

clean:
	rm -rf $(dir $(OUT))
