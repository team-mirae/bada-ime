# Troubleshooting

## Caps Lock (spec §6.7)

bada ime reads Caps Lock presses at the HID level and sets the system's Caps Lock state, which macOS
sends to each keyboard's LED. The log (`~/Library/Logs/BadaIME.log`, on with
`defaults write me.sjun.inputmethod.Bada DiagnosticLog -bool true`) shows every press (`Caps Lock: key down …`), what macOS reported, what bada ime set, and each
keyboard's LED 0.3 s and 1 s after the key is released.

- **Caps Lock does nothing, or is macOS's own:** bada ime needs Accessibility (input menu → Allow
  英数 and かな Everywhere…) and Input Monitoring. The log should say `language key tap: on` and
  `HID keys: on`. Also turn off System Settings → Keyboard → Input Sources → "Use Caps Lock to
  switch to and from ABC".
- **The LED blinks but doesn't stay lit, on one keyboard only** (2026-11-11): a REALFORCE X1P JP TKL
  (Topre, vendor 2131, product 802), set to be ignored by Karabiner-Elements, blinked on ⇧+Caps Lock
  and stayed dark, while a Keychron C2 Pro on the same Mac lit and stayed lit. The REALFORCE wasn't
  among the keyboards bada ime's HID listener opened (the log's LED list). **Fix:** let
  Karabiner-Elements manage it (Devices: not ignored); then it works. Its own lighting setting
  (REALFORCE Connect Online → Illumination → CapsLock Key) must be on, and an animated Normal
  lighting can hide the Caps Lock key's light.
- **Through Karabiner-Elements:** Karabiner seizes the keyboards it modifies and types through its
  own virtual keyboard; bada ime sees the presses there, and Karabiner passes the Caps Lock LED on
  to the physical keyboard.
