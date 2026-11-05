# Bundled fonts

| File | Font | Version source | License |
|---|---|---|---|
| `NotoSans-Regular.ttf` | Noto Sans Regular (static, unhinted) | `notofonts/notofonts.github.io`, `fonts/NotoSans/unhinted/ttf/`, fetched 2026-11-05; SHA-256 `f3961a9cde016d41a4879aecda1474d3a36d6bf54fa0e4643de029cc2248b0e8` | SIL Open Font License 1.1, [`NotoSans-OFL.txt`](NotoSans-OFL.txt) (copied from `google/fonts` `ofl/notosans/OFL.txt`) |
| `NotoSansArabic-Regular.ttf` | Noto Sans Arabic Regular (static, unhinted) | `notofonts/notofonts.github.io`, `fonts/NotoSansArabic/unhinted/ttf/`, fetched 2026-11-05; SHA-256 `bd86ca02f087d7f3c3788ba458fb6b73744c7639ed276b8d870dba6def6c40d0` | SIL Open Font License 1.1, [`NotoSansArabic-OFL.txt`](NotoSansArabic-OFL.txt) (copied from `google/fonts` `ofl/notosansarabic/OFL.txt`) |

Copyright 2022 The Noto Project Authors. The fonts are bundled unmodified
and registered in `src/ui/theme.rs` (`fonts()`) as the first fallbacks
after egui's primary font, for proportional and monospace text. Making Noto
Sans the primary face is still open: its taller line height pushes the
welcome screen past 800×500, so that layout must be reworked first.

Bundling a font is **not** evidence of correct Arabic text. egui lays out
each glyph on its own: it does not apply Arabic contextual shaping or the
bidirectional algorithm. Arabic shaping, bidi, IME and screen readers stay
open in R0-04 and UI-10, to be verified on Windows.
