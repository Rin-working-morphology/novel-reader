---
target: src/views/index.vue
total_score: 27
p0_count: 0
p1_count: 2
timestamp: 2026-05-29T05-57-22Z
slug: src-views-index-vue
---
# src/views/index.vue Design Critique

Target: D:\Projects\novel-reader\src\views\index.vue

## Design Health Score

Total: 27/40, Acceptable.

Key read: the app has a quiet reading-product foundation, but the chapter-boundary navigation currently feels like a separate visual system inserted over the page.

| # | Heuristic | Score | Key Issue |
|---|---|---:|---|
| 1 | Visibility of System Status | 2 | Loading and empty states exist, but chapter boundary affordances appear only near scroll edges and do not explain state consistently. |
| 2 | Match System / Real World | 3 | Reading model is familiar, with file list and chapter selector. The overlay navigation feels more video-player-like than reader-like. |
| 3 | User Control and Freedom | 3 | Users can jump chapters through header and outline, but inline previous/next is click-only and lacks explicit keyboard/focus treatment. |
| 4 | Consistency and Standards | 2 | Header select, outline tree, and boundary overlays use different visual vocabularies for the same chapter-navigation job. |
| 5 | Error Prevention | 3 | Previous/next guards prevent invalid chapter indexes; no destructive interaction here. |
| 6 | Recognition Rather Than Recall | 3 | Chapter select and outline are visible; overlay discovery depends on scrolling to a boundary. |
| 7 | Flexibility and Efficiency | 2 | Ctrl+T exists for transparent mode, but chapter movement lacks visible shortcut or standard keyboard path in this file. |
| 8 | Aesthetic and Minimalist Design | 2 | Reading surface is calm, but glassy gradient overlays add chrome exactly where the text should be quietest. |
| 9 | Error Recovery | 3 | Tauri calls are caught in places; browser run shows noisy console errors outside Tauri, but desktop context likely avoids that. |
| 10 | Help and Documentation | 4 | For this narrow reader flow, extra help is not required; empty state provides the needed first action. |

## Anti-Patterns Verdict

LLM assessment: not broadly AI-looking, but the chapter boundary controls carry several Codex/UI-generator tells: decorative glass, large pill radius, gradient fades, `transition: all`, and an inconsistent control pattern. The main product direction says "chrome disappears"; this implementation makes the chapter edge visually louder than the chapter text.

Deterministic scan: `detect.mjs --json` returned `[]` for `src/views/index.vue` and `src/components/BaseReadingArea.vue`. It did not catch the overlay problem because the issue is compositional and product-register-specific, not a named static anti-pattern in the scanner.

Visual overlays: not available. Browser injection was not used because the Browser Playwright evaluate surface is read-only in this runtime. Browser inspection reached the app shell, but not a selected-book reading boundary.

## Overall Impression

The shell is directionally right for a local reader: quiet palette, restrained layout, familiar Naive UI structure. The biggest opportunity is to treat previous/next chapter as part of the reading page rhythm, not as a floating glass control pasted over it.

## What's Working

1. `index.vue` keeps responsibilities clean: header, file sidebar, unified reading surface, and outline are composed clearly.
2. The reading body has generous vertical padding and 1.8 line height, which supports long-form reading.
3. Multiple chapter navigation routes exist: header select, right outline, and boundary navigation.

## Priority Issues

### [P1] Previous/next overlays fight the reading surface

Why it matters: at chapter boundaries, the user's attention should move smoothly from prose to navigation. The current `chapter-overlay` uses absolute placement, 120px gradient fades, blurred glass, and a 24px pill, which feels like a media overlay instead of reader UI.

Fix: replace overlays with quiet in-flow chapter boundary blocks at the top and bottom of the scroll content. Use the same surface, 1px border or tonal hover, 6-8px radius, and a compact two-line structure: action label plus adjacent chapter title. Keep arrows, but use them as small leading/trailing icons.

Suggested command: `$impeccable layout src/components/BaseReadingArea.vue`

### [P1] Same task, three different chapter navigation vocabularies

Why it matters: header select, outline tree, and overlay pills all mean "move chapter," but they look and behave unrelated. This is exactly the consistency gap the user is feeling as visual fracture.

Fix: define one chapter-navigation component vocabulary: compact action row for boundary moves, select for random access, tree for full outline. They should share sizing, text color, hover color, focus ring, icon size, and active-state treatment.

Suggested command: `$impeccable polish src/views/index.vue`

### [P2] Top and bottom boundary controls have asymmetric information order

Why it matters: top overlay reads icon, action, title; bottom reads title, action, icon. The asymmetry makes the controls feel custom-built twice instead of intentionally mirrored.

Fix: use mirrored structure by direction: previous is arrow + label stack, next is label stack + arrow, with label stack order unchanged: action first, title second.

Suggested command: `$impeccable clarify src/components/BaseReadingArea.vue`

### [P2] Glass and gradient treatments conflict with transparent-mode doctrine

Why it matters: the design system says transparent mode should remove chrome before removing legibility. The current frosted/glass vocabulary in the reading area and boundary controls creates decorative chrome that may remain visually noisy.

Fix: reserve blur only for actual transparent readability protection. In normal reading mode, use flat tonal surfaces. In transparent mode, make boundary controls near-invisible until hover/focus or collapse them into a small edge affordance.

Suggested command: `$impeccable quieter src/components/BaseReadingArea.vue`

### [P3] Header uses gradient text for a plain book title

Why it matters: the gradient is neutralized to one color by theme overrides, but the component choice still carries decorative intent for a utility label.

Fix: replace `n-gradient-text` with plain text styling matching the title token.

Suggested command: `$impeccable typeset src/components/HeaderBar.vue`

## Persona Red Flags

Alex, power reader: will want quick previous/next and random chapter jump. The header select helps, but boundary navigation has no visible keyboard affordance and is only discoverable at scroll edges.

Sam, accessibility-dependent reader: boundary controls are clickable `div`s, not buttons. They need semantic buttons, focus style, keyboard activation, and accessible labels such as "上一章：chapter title".

Casey, distracted mobile or small-window reader: the 120px absolute overlays consume a large share of the reading viewport at the exact moment the user is scanning chapter end text. Long chapter titles are capped at 200px and may become ambiguous.

## Minor Observations

- The empty state is quiet and appropriate, but the visible right collapsed trigger creates a stray control at the far edge when no outline content exists.
- The loading copy `小祥●▛▙...` has personality, but it may read as broken glyphs to first-time users.
- `transition: all 0.3s ease` should be narrowed to color, background-color, opacity, or transform.

## Questions to Consider

- Should chapter boundary navigation feel like a page footnote, a toolbar action, or a floating edge hint?
- If the reader chrome disappeared for 95% of the session, what is the minimum previous/next affordance users still need?
- Do header chapter select and right outline both need to exist at full strength, or should one become secondary?
