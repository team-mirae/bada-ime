// Learn Korean: the characters of a drill's keys as key codes, and back. An uppercase letter is ⇧.

/** "a" → KeyA, "A" → ⇧KeyA, ";" → Semicolon, "-" → Minus. */
export function keyOf(ch) {
  if (ch === ";") return { code: "Semicolon", shift: false };
  if (ch === "-") return { code: "Minus", shift: false };
  return { code: "Key" + ch.toUpperCase(), shift: ch !== ch.toLowerCase() };
}

/** The keys the learn page takes from the keyboard: the letters, ; and -. */
export function isDrillKey(code) {
  return /^Key[A-Z]$/.test(code) || code === "Semicolon" || code === "Minus";
}
