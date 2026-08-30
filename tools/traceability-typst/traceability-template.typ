// Data-in / template-out layout for the traceability PDF.
//
// All rulebook data lives in `data.json` (emitted by `traceability-typst` with
// a 3rd positional arg). This template only handles layout/styling.
// Spike: `json("data.json")` is relative to this file.
//
// NOTE: keep code expressions on consecutive lines. A blank line between two
// code expressions in Typst markup adds paragraph spacing (~0.65em), which
// shifts every following element and changes page breaks.

#set page(paper: "a4", margin: (top: 2cm, bottom: 2cm, left: 2.5cm, right: 2cm))
#set text(font: ("EB Garamond", "Libertinus Serif", "DejaVu Serif"), size: 10pt)
#set par(justify: true, leading: 0.5em)
#set heading(numbering: none)

#show raw.where(block: true): set text(font: ("DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"), size: 7.5pt)
#show raw.where(block: false): set text(font: ("DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"), size: 8.5pt)
#show raw.where(block: true): block.with(fill: luma(248), inset: 0.4em, radius: 2pt)

#show heading.where(level: 1): it => {
  pagebreak()
  v(1em)
  block(stroke: (left: 3pt + luma(80)), inset: (top: 0.2em, bottom: 0.2em, left: 0.6em, right: 0.3em), it)
}

#show heading.where(level: 2): it => {
  v(0.6em)
  it
}

#let data = json("data.json")
#let root = data.root

#let status-tag(status) = {
  let (bg, fg) = if status == "implemented" {
    (green.transparentize(70%), green.darken(30%))
  } else if status == "descriptive" {
    (blue.transparentize(70%), blue.darken(30%))
  } else if status == "implicit" {
    (yellow.transparentize(70%), yellow.darken(40%))
  } else {
    (luma(85), luma(40))
  };
  box(
    fill: bg, inset: (left: 0.4em, right: 0.4em, top: 0.1em, bottom: 0.1em),
    radius: 3pt, text(fill: fg, size: 8pt, weight: "bold", status)
  )
}

#let vscode-link(rel, line) = {
  let abs = root + "/" + rel
  link("vscode://file/" + abs + ":" + str(line))[
    #text(size: 9pt, fill: blue.darken(20%), rel + ":" + str(line))
  ]
}

#let github-link(rel, line) = {
  let url = "https://github.com/barafael/omdurman/blob/HEAD/" + rel + "#L" + str(line)
  link(url)[
    #text(size: 8pt, fill: luma(100), "GH:" + rel + ":" + str(line))
  ]
}

#let progress-bar(done, total) = {
  let filled = "█" * done
  let empty = "░" * (total - done)
  text(font: ("DejaVu Sans Mono", "Liberation Mono"), size: 8pt)[
    #text(fill: green.darken(20%))[#filled]#text(fill: luma(180))[#empty] #done/#total implemented
  ]
}

// Manual prose. `text` segments are inserted literally (no markup parsing, so
// nothing to escape); `ref` segments render as plain text because data-driven
// labels are not possible in Typst; `raw` segments (backtick spans) render as
// inline code, matching the markup path.
#let render-segments(segments) = {
  let out = []
  for s in segments {
    if s.type == "text" {
      out = out + s.text
    } else if s.type == "ref" {
      out = out + s.rref
    } else {
      out = out + raw(block: false, s.text)
    }
  }
  out
}

// Render the structured manual (paragraphs + bullet/ordered lists, possibly
// nested) recursively. The Rust side has already applied smart quotes and
// dash normalization, so the strings render exactly as the markup path did.
//
// A `#parbreak()` before a list/enum that follows a paragraph reproduces the
// blank line between them in the old markup path (that parbreak is what adds
// the ~19pt gap in EB Garamond; without it the list sits ~19pt too high).
// It is emitted only when the source actually had a blank line before the
// block (`b.blank_before`): some lists are attached directly under a
// paragraph in the old markup (e.g. §9.14's "receives:" lists) and would get
// the gap wrongly. The blank lines only exist between *top-level* blocks:
// the old markup always attaches a list/enum inside an item directly to the
// item text, so item content is rendered with `attach: true`, which disables
// the parbreak.
//
// `loose` mirrors Typst's loose-list rule: a blank line between any two items
// (or before nested content) makes the whole list non-tight, spacing every
// item by paragraph spacing (~0.65em extra) instead of the tight leading
// gutter. `#list(tight: false)` reproduces that exactly.
#let render-blocks(blocks, attach: false) = [
  #for i in range(blocks.len()) [
    #let b = blocks.at(i)
    #if b.type == "paragraph" [
      #par[#render-segments(b.segments)]
    ] else if b.type == "list" [
      #if not attach and i > 0 and b.blank_before and blocks.at(i - 1).type == "paragraph" [#parbreak()]
      #list(tight: not b.loose, ..b.items.map(item => render-blocks(item.blocks, attach: true)))
    ] else [
      #if not attach and i > 0 and b.blank_before and blocks.at(i - 1).type == "paragraph" [#parbreak()]
      #enum(numbering: "1.", tight: not b.loose, ..b.items.map(item => render-blocks(item.blocks, attach: true)))
    ]
  ]
]

#let render-snippet(imp) = {
  if imp.ext == "" {
    raw(block: true, imp.snippet)
  } else {
    raw(block: true, lang: imp.ext, imp.snippet)
  }
}

// Title block
#align(center, text(size: 18pt, weight: "bold", "Traceability Matrix"))
#align(center, text(size: 10pt, "REMEMBER GORDON! – Rulebook ⇌ Implementation Mapping"))
#align(center, text(size: 9pt, fill: luma(120), "Generated from `docs/traceability.toml`"))
#v(2em)

// Overview
#heading(level: 1, "Overview") <sect-overview>
#v(0.3em)
#table(
  columns: (1fr, 1fr, 1fr, 1fr),
  stroke: 0.4pt + luma(190),
  [*Implemented*], [*Descriptive*], [*Implicit*], [*Out-of-scope*],
  [#text(fill: green.darken(20%))[#data.status_counts.at("implemented", default: 0)]],
  [#text(fill: blue.darken(20%))[#data.status_counts.at("descriptive", default: 0)]],
  [#text(fill: yellow.darken(30%))[#data.status_counts.at("implicit", default: 0)]],
  [#data.status_counts.at("out-of-scope", default: 0)],
)
#v(0.3em)
#text(size: 9pt)[Total mappings: #data.total_mappings · Total impl sites: #data.total_impl_sites]
#v(1em)

#outline(title: [Table of Contents])
#pagebreak()

// Chapters
#for ch in data.chapters [
  #progress-bar(ch.done, ch.total)
  #heading(level: 1, ch.title)
  #for s in ch.sections [
    #heading(level: 2, s.heading)
    #status-tag(s.status)
    #linebreak()
    #if s.page == none [
      #text(size: 8.5pt, fill: luma(120), style: "italic")[manual page unknown]
    ] else [
      #text(size: 8.5pt, fill: luma(120))[manual page #s.page]
    ]
    #v(0.3em)
    #if s.manual.len() > 0 [
      #if s.collapsed [
        #stack(
          block(height: 5cm, clip: true, stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[#render-blocks(s.manual)]],
          align(right, text(size: 8pt, fill: luma(120), style: "italic")[(see manual for full text)])
        )
      ] else [
        #block(stroke: (left: 3pt + luma(60)), fill: luma(248), inset: 0.5em, radius: 2pt)[#quote(block: true)[#render-blocks(s.manual)]]
      ]
      #v(0.5em)
    ]
    #if s.see_also.len() > 0 [
      #text(size: 8.5pt, fill: luma(120), style: "italic")[See also: #s.see_also.join(", ")]
      #v(0.3em)
    ]
    #if s.impls.len() > 0 [
      #let rows = s.impls.map(imp => (
        [#vscode-link(imp.file, imp.line) \ #github-link(imp.file, imp.line)],
        [#link("https://github.com/barafael/omdurman/blob/HEAD/" + imp.file + "#L" + str(imp.line))[#highlight(fill: yellow.transparentize(70%))[#text(weight: "bold")[#imp.symbol]]]],
        [#if imp.snippet != "" [#render-snippet(imp)]],
      )).flatten()
      #table(
        columns: (1.2fr, 1.8fr, 5fr),
        stroke: 0.4pt + luma(190),
        [*File*], [*Symbol*], [*Code Snippet*],
        ..rows,
      )
      #v(0.5em)
    ]
    #if s.proofs.len() > 0 [
      #let proof-tags = s.proofs.map(t => box(fill: blue.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: blue.darken(30%), weight: "bold")[#t]])
      #text(size: 9pt, fill: luma(80))[Proven by: #proof-tags.join(" ")]
      #v(0.3em)
    ]
    #if s.tests.len() > 0 [
      #let test-tags = s.tests.map(t => box(fill: green.transparentize(85%), inset: (left: 0.3em, right: 0.3em, top: 0.1em, bottom: 0.1em), radius: 2pt)[#text(size: 8pt, fill: green.darken(30%), weight: "bold")[#t]])
      #text(size: 9pt, fill: luma(80))[Covered by tests: #test-tags.join(" ")]
      #v(0.3em)
    ]
  ]
]

// Symbol index
#heading(level: 1, "Appendix: Symbol Index") <sect-symbol-index>
#v(0.5em)
#let index-rows = data.symbol_index.map(entry => (
  [#text(weight: "bold", size: 9pt)[#entry.symbol]],
  [#entry.sections.join(", ")],
)).flatten()
#table(
  columns: (2fr, 5fr),
  stroke: 0.4pt + luma(190),
  [*Symbol*], [*Sections*],
  ..index-rows,
)
