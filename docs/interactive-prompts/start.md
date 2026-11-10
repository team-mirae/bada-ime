# Install bada ime

Follow [rules.md](rules.md) throughout. Facts are in [docs/spec.md](../spec.md) (§4.1 setup, §7 build)
and [mac/Makefile](../../mac/Makefile).

## 0. The clone

You were pointed here from a clone of team-mirae/bada-ime: look at it without asking (README, `mac/`,
`engine/`). If it isn't one, say so and stop.

## 1. Questions, one at a time

1. > Which keyboard will you type on? bada ime is for **JIS keyboards** only (日本語配列); on others
   > the symbol keys type the wrong symbols.
   >
   > 1. A Mac with a JIS keyboard
   > 2. An external JIS keyboard (HHKB JIS, Realforce JIS, …)
   > 3. Not JIS
   For 3: say bada ime won't fit and stop, unless they want to try anyway.
2. > Which do you want to type in henkan mode? (Several can be on at once; they're ranked as you
   > type.) Korean (bada, 2-set, JIS Korean, JIS Moonshot, Roman, Kana), Japanese (Kana, Romaji),
   > English (QWERTY).
   Note the answer for step 4.

## 2. Tools

Building needs **Rust** (`cargo`) and the **Command Line Tools** (`swiftc`). Check for each (a read,
section 1 of the rules); for a missing one, ask to install it on its own (rustup from rustup.rs;
`xcode-select --install`).

## 3. Build and install

Each is a question of its own:

1. `make` in `mac/`: builds `mac/build/BadaIME.app`. Only writes inside the clone.
2. `make install` in `mac/`: copies it to `~/Library/Input Methods`, registers it, enables both
   modes and adds a login agent (`~/Library/LaunchAgents/me.sjun.inputmethod.Bada.login.plist`).
   Undo: `make uninstall`.

## 4. Input sources and settings

Walk them through these in System Settings → Keyboard → Input Sources (they click; you explain), one
change per step:

1. Check **bada (muhenkan)** (English) and **bada (henkan)** (Japanese) are listed; add any missing.
2. Add **ABC – Extended**, then remove **ABC**, so 英数 lands in bada (muhenkan) (spec §4.1).
3. Remove other Japanese input methods, so かな lands in bada (henkan).
4. In the input menu, **Allow 英数 and かな Everywhere…** grants Accessibility, so かな can move through
   suggestions while already in henkan mode.
5. In **bada ime Settings…**, turn on the henkan candidates from step 1.2.

## 5. Try it

Ask them to type a sentence in each language they chose, and say what came out. Fix what's off.

## 6. The dictionary

Then go to [dictionary.md](dictionary.md).

## 7. Wrap up

List what was installed and changed, and how to undo each: `make uninstall`, the input sources, the
Accessibility permission (System Settings → Privacy & Security → Accessibility), the login agent.
