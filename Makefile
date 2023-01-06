CARGO=cargo
CARGO_FLAGS=

ifeq ($(APP_ENVIRONMENT),prod)
	TARGET=target/release/nec_telegram_bot
	CARGO_FLAGS+=--release
else
	TARGET=target/debug/nec_telegram_bot
endif

.DEFAULT_GOAL := build

build: $(TARGET)
.PHONY: build

$(TARGET):
	$(CARGO) build $(CARGO_FLAGS)
