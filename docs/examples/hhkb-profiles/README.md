# Example: HHKB JIS with three henkan profiles

The owner's own setup (2026-11-08): an HHKB Professional HYBRID Type-S, JIS (PD-KB820BS), with
bada ime 0.1.0. Open [setup.html](setup.html) in a browser for the whole of it, keyboard included.

| Keys | Sends | Profile | Candidates |
|---|---|---|---|
| かな | かな (LANG1) | `mac-kana` | QWERTY + Korean (bada) ×2 |
| Fn + かな | 変換 (HID 0x8A) | `henkan` | QWERTY + Japanese (Kana) ×2 |
| Fn + 英数 | カタカナ (HID 0x88) | `katakana` | Symbols + Unicode |
| 英数 | 英数 (LANG2) | muhenkan | QWERTY |

- **The keyboard** ([HHKB_Setting_20261108.hks](HHKB_Setting_20261108.hks), HHKB Keymap Tool
  2.0.1): the Normal layer is the HHKB JIS default; the Fn layer puts 変換 on かな and カタカナ on
  英数 (and Caps Lock on 半角/全角). macOS gives those two keys no key events of their own, so bada
  ime reads them at the HID level (spec §6.6): Settings → Profiles → Record Key, then press them.
- **bada ime** ([bada-settings.json](bada-settings.json), Settings → Backup → Export): profiles
  on, each profile with its own F keys (変換's F6–F10 are the Kana actions: hiragana, katakana,
  half-width katakana, full- and half-width Latin; カタカナ's not intercepted), script forms on.
- To use it: import the settings file in bada ime (Settings → Backup → Import), load the keymap in
  HHKB Keymap Tool, and record the two profile keys again on your Mac.

Render your own: `python3 tools/setup-page/build.py KEYMAP.hks bada-settings.json setup.html`.
