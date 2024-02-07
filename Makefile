LINUX_BUILD_DIR=target/x86_64-unknown-linux-gnu/release
WIN_BUILD_DIR=target/x86_64-pc-windows-gnu/release
WIN32_BUILD_DIR=target/i686-pc-windows-gnu/release
MAC_BUILD_DIR=target/x86_64-apple-darwin/release

.SILENT: all clean linux win win32
.PHONY: all clean linux win win32

all: linux win win32 mac

linux:
	cargo build --release --target x86_64-unknown-linux-gnu
	strip $(LINUX_BUILD_DIR)/burrito
	zip -R $(LINUX_BUILD_DIR)/burrito.zip data/* data/sounds/*
	cd $(LINUX_BUILD_DIR) && zip -u burrito.zip burrito

win:
	cargo build --release --target x86_64-pc-windows-gnu
	strip $(WIN_BUILD_DIR)/burrito.exe
	zip -R $(WIN_BUILD_DIR)/burrito.zip data/* data/sounds/* install.bat
	cd $(WIN_BUILD_DIR) && zip -u burrito.zip burrito.exe

win32:
	cargo build --release --target i686-pc-windows-gnu
	strip $(WIN32_BUILD_DIR)/burrito.exe
	zip -R $(WIN32_BUILD_DIR)/burrito.zip data/* data/sounds/* install.bat
	cd $(WIN32_BUILD_DIR) && zip -u burrito.zip burrito.exe

mac:
	cargo build --release --target x86_64-apple-darwin
	strip $(MAC_BUILD_DIR)/burrito
	zip -R $(MAC_BUILD_DIR)/burrito.zip data/* data/sounds/*
	cd $(MAC_BUILD_DIR) && zip -u burrito.zip burrito

clean:
	cargo clean
