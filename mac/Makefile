# Builds BadaIME.app (universal arm64 + x86_64) with the Command Line Tools; no Xcode needed.
#   make            build build/BadaIME.app
#   make install    copy it to ~/Library/Input Methods, register it, add a login agent
#   make uninstall  remove it
#   make SENSITIVE=1  also build in the vulgar, hateful and sensitive English words (layout 15)
ENGINE     := ../engine
HEADER     := $(ENGINE)/include/bada.h
BUILD      := build
APP        := $(BUILD)/BadaIME.app
TARGET_DIR := $(ENGINE)/target/mac
INSTALL_TO := $(HOME)/Library/Input Methods
LOGIN_AGENT := $(HOME)/Library/LaunchAgents/me.sjun.inputmethod.Bada.login.plist
MIN_MACOS  := 13.0
SOURCES    := $(wildcard Sources/*.swift)
CARGO      := $(shell command -v cargo || echo $(HOME)/.cargo/bin/cargo)
SIGN_IDENTITY ?= $(or $(shell security find-identity -v -p codesigning 2>/dev/null | sed -n 's/.*"\(Apple Development: [^"]*\)".*/\1/p' | head -1),-)
LSREGISTER := /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
DATA       := $(ENGINE)/data/ko-words.tsv $(ENGINE)/data/en-words.txt $(ENGINE)/data/hanja.tsv $(ENGINE)/data/oecd.tsv $(ENGINE)/data/ja-words.txt $(ENGINE)/data/kanji.bin $(ENGINE)/data/ko-jamo-4gram.bin $(ENGINE)/data/symbols.tsv $(ENGINE)/data/emoji.tsv $(ENGINE)/data/unicode-names.tsv $(ENGINE)/data/LICENSE-mozc.txt \
              $(ENGINE)/data/LICENSE-corpora.txt $(ENGINE)/data/LICENSE-libhangul-hanja.txt $(ENGINE)/data/LICENSE-unicode.txt
# The English → Korean dictionary (layout 15, docs/layouts/15-bada-roman.md). Vulgar, hateful and
# sensitive words (en-ko/sensitive.tsv) are left out; `make SENSITIVE=1` builds them in.
SENSITIVE  ?= 0
EN_KO      := $(ENGINE)/data/en-ko/common.tsv
ifeq ($(SENSITIVE),1)
EN_KO      += $(ENGINE)/data/en-ko/sensitive.tsv
endif
BRANDS     := $(sort $(wildcard $(ENGINE)/data/en-ko/brands/*.tsv))
# Rebuilds the app when SENSITIVE changes.
SENSITIVE_STAMP := $(BUILD)/.sensitive
ICONS      := $(BUILD)/bada.tiff $(BUILD)/muhenkan.tiff $(BUILD)/henkan.tiff
# The app's icon, for Settings → About only (the input menu keeps 바: Resources/icons).
ABOUT_ICON := ../docs/images/icon/bada-icon-512.png

export MACOSX_DEPLOYMENT_TARGET := $(MIN_MACOS)

.PHONY: all install uninstall clean
all: $(APP)
$(shell mkdir -p $(BUILD) && touch $(BUILD)/.metadata_never_index)
$(shell echo $(SENSITIVE) | cmp -s - $(SENSITIVE_STAMP) || echo $(SENSITIVE) > $(SENSITIVE_STAMP))

$(BUILD)/libbada.a: $(wildcard $(ENGINE)/src/*.rs) $(ENGINE)/Cargo.toml
	$(CARGO) build --manifest-path $(ENGINE)/Cargo.toml --release --target-dir $(TARGET_DIR) --target aarch64-apple-darwin
	$(CARGO) build --manifest-path $(ENGINE)/Cargo.toml --release --target-dir $(TARGET_DIR) --target x86_64-apple-darwin
	lipo -create -output $@ $(TARGET_DIR)/aarch64-apple-darwin/release/libbada.a $(TARGET_DIR)/x86_64-apple-darwin/release/libbada.a

$(BUILD)/%/BadaIME: $(SOURCES) $(HEADER) $(BUILD)/libbada.a
	@mkdir -p $(dir $@)
	swiftc -O -swift-version 5 -target $*-apple-macos$(MIN_MACOS) -import-objc-header $(HEADER) \
		-framework Cocoa -framework InputMethodKit -framework Carbon $(SOURCES) $(BUILD)/libbada.a -o $@

$(BUILD)/BadaIME: $(BUILD)/arm64/BadaIME $(BUILD)/x86_64/BadaIME
	lipo -create -output $@ $^

$(BUILD)/%.tiff: Resources/icons/%.png Resources/icons/%@2x.png
	tiffutil -cathidpicheck Resources/icons/$*.png Resources/icons/$*@2x.png -out $@

$(APP): $(BUILD)/BadaIME $(ICONS) $(ABOUT_ICON) Info.plist $(DATA) $(EN_KO) $(BRANDS) $(SENSITIVE_STAMP) $(wildcard Resources/*.lproj/*)
	rm -rf $@
	mkdir -p $@/Contents/MacOS $@/Contents/Resources/data
	cp Info.plist $@/Contents/Info.plist
	cp $(BUILD)/BadaIME $@/Contents/MacOS/BadaIME
	cp $(ICONS) $@/Contents/Resources/
	cp $(ABOUT_ICON) $@/Contents/Resources/bada-icon.png
	cp -R Resources/en.lproj Resources/ko.lproj Resources/ja.lproj $@/Contents/Resources/
	cp $(DATA) $@/Contents/Resources/data/
	{ awk 1 $(EN_KO); echo '#! brands'; awk 1 $(BRANDS); } > $@/Contents/Resources/data/en-ko.tsv
	cp ../LICENSE $@/Contents/Resources/LICENSE.txt
	codesign --force --sign "$(SIGN_IDENTITY)" $@

install: $(APP)
	@killall BadaIME 2>/dev/null || true
	mkdir -p "$(INSTALL_TO)"
	rm -rf "$(INSTALL_TO)/.BadaIME.app.new"
	ditto $(APP) "$(INSTALL_TO)/.BadaIME.app.new"
	if [ -d "$(INSTALL_TO)/BadaIME.app" ]; then rm -rf "$(INSTALL_TO)/.BadaIME.app.old"; mv "$(INSTALL_TO)/BadaIME.app" "$(INSTALL_TO)/.BadaIME.app.old"; fi
	mv "$(INSTALL_TO)/.BadaIME.app.new" "$(INSTALL_TO)/BadaIME.app"
	rm -rf "$(INSTALL_TO)/.BadaIME.app.old"
	$(LSREGISTER) -u "$(CURDIR)/$(APP)" 2>/dev/null || true
	$(LSREGISTER) -f "$(INSTALL_TO)/BadaIME.app"
	swift tools/register.swift "$(INSTALL_TO)/BadaIME.app"
	mkdir -p "$(dir $(LOGIN_AGENT))"
	cp tools/login-agent.plist "$(LOGIN_AGENT)"

uninstall:
	@killall BadaIME 2>/dev/null || true
	rm -rf "$(INSTALL_TO)/BadaIME.app" "$(LOGIN_AGENT)"

clean:
	rm -rf $(BUILD)
