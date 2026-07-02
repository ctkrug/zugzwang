# Design

Design direction for Zugzwang's landing page (`site/index.html`). Zugzwang is a CLI, so there is
no in-browser app; the page is a marketing + reference page for the engine. This file is the
single source of truth for its look, so the page reads as one deliberate brand rather than a stub.

## 1. Aesthetic direction

**Paper-and-ink analysis diagram.** The page looks like a page from a well-set chess book: warm
ivory paper, ink-dark text, a printed board diagram as the hero, and one deep oxblood accent for
the things that matter (links, the CTA, the mark). Engine output is shown in inked "board diagram"
and "transcript" cards, the way a book prints a position next to its analysis. The vibe is
studious and precise, matching an audience that reads source code for fun.

## 2. Tokens

| Token | Value | Use |
|-------|-------|-----|
| `--paper` | `#f4efe4` | page background (warm ivory) |
| `--surface` | `#ece3d2` | recessed panels |
| `--raised` | `#fbf8f1` | raised cards |
| `--ink` | `#1c1a17` | primary text |
| `--muted` | `#6b6456` | secondary text |
| `--accent` | `#8c2f39` | oxblood: links, CTA, mark, "check" |
| `--support` | `#3f6b52` | tournament-board green: secondary highlights |
| `--board-light` | `#efdfbf` | light board squares |
| `--board-dark` | `#b58863` | dark board squares (classic wood) |
| `--line` | `#d8cdb8` | hairline borders |

- **Display font:** Fraunces (Google Fonts), an editorial serif for the wordmark and headings —
  it reads like a book title. Fallback: `Georgia, serif`.
- **UI/body font:** Inter (Google Fonts), for all running text and controls. Fallback:
  `system-ui, sans-serif`.
- **Monospace** (engine output only): `"JetBrains Mono", ui-monospace, SFMono-Regular, monospace`.
- **Type scale:** ~1.25 ratio (16 / 20 / 25 / 31 / 39 / 49 px).
- **Spacing:** 8px scale (4, 8, 16, 24, 32, 48, 64).
- **Radius:** 6px (crisp, not pill-soft). **Shadow:** soft paper lift,
  `0 1px 2px rgba(28,26,23,.06), 0 8px 24px rgba(28,26,23,.06)`.
- **Motion:** UI transitions 140–200ms ease-out; nothing gratuitous.

## 3. Layout intent

- **Header:** wordmark + geometric mark top-left; a single "View on GitHub" link top-right.
- **Hero (fills the first screen):** two columns at ≥900px. Left: eyebrow, H1 headline, subhead,
  two CTAs (primary "View on GitHub", secondary "See it run"). Right: the **hero visual** — the
  starting position rendered as a printed board diagram (CSS grid, Unicode pieces, rank/file
  coordinates). At <900px the columns stack, headline first, board below, both full-width; no
  horizontal scroll at 390px.
- **Show-the-thing:** a transcript card with a real UCI session and a real terminal-play board,
  both copied verbatim from the release binary.
- **Features as benefits:** four items, each naming a concrete capability (perft depth, quiescence,
  Zobrist TT, UCI command set).
- **Reference/FAQ:** 300–600 words of genuinely useful copy for the audience plus a short FAQ, for
  SEO and for anyone deciding whether to read the source.
- **Footer:** MIT note, GitHub link, and the portfolio cross-promotion link.

## 4. Signature detail

The **hero board diagram**: a real chessboard drawn in CSS with Unicode pieces and a-h / 1-8
coordinates, framed like a figure in a chess book, captioned with the exact bytes the engine
prints. It ties the paper theme to the actual product output in one image, no raster assets.

## 5. Accessibility

Text contrast ≥ 4.5:1 (ink on paper is ~13:1; oxblood on paper ~6:1). Every link and button has a
themed hover, `:focus-visible` ring (oxblood), and active state. Respects `prefers-reduced-motion`.
Touch targets ≥ 44px. Semantic landmarks (`header`/`main`/`footer`), one `<h1>`.
</content>
</invoke>
