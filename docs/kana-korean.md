# JIS Kana Korean (layout 19)

Status: built in v0.1, minor layout (owner, 2026-11-07). Ranked since 2026-11-07: see Readings. Layout: `docs/layouts/kana-korean.json`;
code: `engine/src/kana_korean.rs`.

Korean typed on the JIS kana layout: each kana key types that kana's Korean approximation
(た 타, し 시, ん final ㄴ). Common syllables are common, and the Japanese approximation of Korean
is close, so a typist who knows kana types Korean without a second layout. With JIS Kana (20)
enabled too, the same keys read as Japanese and Korean at once, and the kana approximation narrows
the Korean readings.

## Keys

The number row, letters and symbol keys follow the JIS kana legends (ぬ 누, ふ 후, あ 아 … ろ 로;
ー types ー). Exceptions and modifiers:

| Key | Kana | Types |
|---|---|---|
| E | い | ㅣ: joins a lone consonant (Z ㅋ then E → 키), else 이 |
| Y | ん | final ㄴ on the previous syllable; ⇧: final ㅇ. Followed by a small vowel, it starts a syllable instead: plain ん on ㄴ (or ㅇ: ん⇧ゃ shows 냐, offers 야), ⇧ん on ㅇ (owner, 2026-11-07) |
| ] | む | 무; ⇧: final ㅁ on the previous syllable |
| K | の | 노; ⇧: **의**, completed (の taken literally as the particle; nothing attaches to it) (owner, 2026-11-07) |
| Z | つ | ㅋ, as a final after an open syllable (카 Z → 캌), else an initial |
| ⇧Z | っ | the next initial goes one step tenser: ㄱ/ㅋ → ㄲ, ㄷ/ㅌ → ㄸ, ㅂ/ㅍ → ㅃ, ㅅ → ㅆ, ㅈ/ㅊ → ㅉ. Followed by a small vowel: ㅋ with that vowel (⇧Z ⇧ぁ → 카, ⇧Z ⇧ゃ → 캬) (owner, 2026-11-07) |
| ¥ | ー | ー; after a short ㅔ / ㅖ, their long form ㅐ / ㅒ instead (세ー → 새, 헤ー → 해) (owner, 2026-11-07) |
| ⇧3–⇧0 | ぁぅぇぉゃゅょ, を | small vowels ㅏ ㅜ ㅔ ㅗ ㅑ ㅠ ㅛ ㅘ: change the previous syllable's vowel (き ⇧ゃ → 캬), joining where Korean has the compound (う ⇧ぇ → 웨) |
| @ | ゛ | voices the previous initial: ㅋ → ㄱ, ㅌ → ㄷ, ㅅ → ㅈ, ㅊ → ㅈ, ㅎ → ㅂ, ㅇ → ㅂ (ゔ) |
| [ | ゜ | ㅎ → ㅍ (ぱ) |

Examples: こんにちは → 콘니치하; ありがとう → 아리가토우; か゛⇧むさ → 감사; ⇧Z か゛ → 까.

Other keys with Shift type the same as without, as in JIS Kana.

## Readings (owner, 2026-11-07)

The preedit shows the **default reading**, the table's (く 쿠). Underneath, every pronunciation of
the kana typed is kept, and each reading that spells a known Korean word (`ko-words.tsv`) is
offered in the menu under the default, most frequent first (Tab / ↓ / かな to it).

| Kana | Readings |
|---|---|
| う-row (う く ぐ す ず つ ぬ ふ ぶ ぷ む る) | vowel ㅜ (default) or ㅡ: く 쿠 / 크, ぐ 구 / 그 |
| う-row as a final | ク / グ ㄱ, ム ㅁ, ル ㄹ, プ / ブ ㅂ on the syllable before, as Japanese spells Korean finals: はんぐく → 한국, か゛むさ → 감사 |
| お-row, and ぉ | ㅗ (default) or ㅓ: そうる → 서울 |
| ょ | ㅛ (default) or ㅕ |
| ん (plain) | final ㄴ (default) or ㅇ; before a small vowel, initial ㄴ (default) or ㅇ. ⇧ん is always ㅇ |
| っ | the next initial tense (default), or a ㅅ final on the syllable before: かっか 카까 / 캇카 |
| ぐ / く after an ㅇ final | ング spells the ㅇ, so the ぐ drops out: あんにょ⇧んぐはせよ shows 안뇽구하세요 and offers 안녕하세요 |

Readings are searched depth-first, the default choice first at each kana, up to 4,096 per
segment; a long segment of ambiguous kana may not reach every combination.

## Open

- **[decide]** ー lengthens ㅔ and ㅖ only; whether other vowels have a long form too.
- **[decide]** Finals: only ㄴ ㅇ ㅁ ㅋ have keys, so 합 or 있 can't be typed. Fine for a minor
  layout, or should more finals get Shift slots?
- **[decide]** Digits and symbols: every kana-space key is taken, so this layout types none; they
  come from muhenkan mode.
