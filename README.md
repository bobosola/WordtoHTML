# wordtohtml

This is a small cross-platform command-line tool that converts a Microsoft Word `.docx` document into clean minimal HTML. It is designed for turning blog posts written in Word into HTML that you can paste directly into a website or CMS.

By default it copies the converted document text to the clipboard. You can also optionally pass `-o`  (`--output`) to write a complete minimal HTML file named as per the Word document and in the same location.

## Examples

Every option has a short and a long form. In the examples below, `wordtohtml` is the release binary at`./target/release/wordtohtml` (see [Build](#build)).

```sh
# copy the body content to the clipboard (the default)
wordtohtml mypost.docx
# copied to clipboard in approx. 90 columns

# write a complete minimal HTML document next to the input: mypost.docx -> mypost.html
wordtohtml -o mypost.docx (or --output)
# wrote mypost.html in approx. 90 columns

# wrap the output near 60 columns instead of the default 90
wordtohtml -w 60 mypost.docx (or --wrap)
# wrote mypost.html in approx. 60 columns

# one long line with no newlines to clipboard (handy for embedding or minifying)
wordtohtml -w 0 mypost.docx
# copied to clipboard with no newlines

# one long line with no newlines to HTML file
wordtohtml -o -w 0 mypost.docx
# wrote mypost.html with no newlines

# show the help text
wordtohtml -h (or --help)
```

Run it straight from source without building the binary first:

```sh
cargo run --release -- --wrap 100 "/path/to/document.docx"
```

## Features

- **Paragraphs** -> `<p>`
- **Headings** (`Heading1` … `Heading6`, or `Heading 1` …) -> `<h1>` … `<h6>`
- **Bulleted and numbered lists** -> `<ul>` / `<ol>` with `<li>` items, including nested list levels
- **Tables** -> `<table>` with `<thead>`/`<tbody>`, `<tr>`, `<td>` and `<th>` for the header row
- **Bold** -> `<strong>`, **italic** -> `<em>`, **underline** -> `<u>`,
  **strikethrough** -> `<s>`, **subscript** -> `<sub>`, **superscript** -> `<sup>`
  - Word's built-in `Strong` and `Emphasis` character styles are recognised
  - Adjacent runs with identical formatting are merged into one tag
- **Line breaks** (`<w:br/>`, `<w:cr/>`) -> `<br>`, and tabs -> a space
  (a break or tab inside code is kept as a newline/tab)
- **Monospace text** -> `<code>`; a paragraph that is entirely monospace, or uses a preformatted/code paragraph style (`HTML Preformatted`, `Source Code`), becomes a `<pre><code>` block. Runs are recognised by font (`Consolas`, `Courier New`, `JetBrains Mono`, …) and by Word's `HTML Code` / `Code` / `VerbatimChar` character styles. Spaces, tabs and line breaks inside a code block are preserved verbatim, and no CSS is emitted, so the page styles `pre`/`code` itself.
- **Internal links** (`w:anchor`, e.g. table-of-contents entries) -> `<a href="#bookmark">`
- **Hyperlinks** -> `<a href="...">`
- **Symbol-font glyphs** (`w:sym`, e.g. a Wingdings smiley) -> the matching Unicode character (`🙂`), so they render instead of vanishing. Word stores these as an index into the named font rather than a character, so `Symbol`, `Wingdings` `Wingdings 2`, `Wingdings 3` and `Webdings` each have a glyph table (`w:char="F04A"` is Wingdings 0x4A, a smiley). Faces become colour emoji (`U+1F642`) rather than the legacy monochrome code points
- **Typographic punctuation** cleaned up for the web:
  - `’` / `‘` -> `'`
  - `”` / `“` -> `"`
  - en dash -> `&ndash;`, em dash -> `&mdash;`
  - ellipsis -> `&hellip;`, non-breaking space -> `&nbsp;`
- **UTF-8 output** so Word's typographic characters render correctly instead of as mojibake such as `â€™` (file output also gets a `<meta charset="utf-8">`)
- **Readable formatting**: long lines are wrapped near 90 characters (configurable with `--wrap`; `--wrap 0` disables wrapping for single-line output) and blocks are separated by a single blank line
- **Clipboard output** by default, printing `copied to clipboard` so it is clear the fragment is ready to paste

## Requirements

- Rust (edition 2024, so Rust 1.85 or newer). Built and tested with `cargo 1.98`.
- No external tools or Office installation are needed.
- Runs on macOS, Windows and Linux. On Linux the clipboard uses X11/XWayland or the Wayland data-control protocol.

## Build

```sh
cargo build --release
```

The binary is produced at `target/release/wordtohtml`.

## Usage

```sh
wordtohtml [OPTIONS] <path-to-docx>
```

Options:

| Option | Description |
| --- | --- |
| `-o, --output` | Write the complete HTML document next to the input (`doc.docx` -> `doc.html`) instead of copying the fragment to the clipboard. |
| `-w, --wrap <columns>` | Wrap the output near this many characters (default `90`). Use `0` for a single line with no newlines. |
| `-h, --help` | Show usage. |

See [Examples](#examples) for short and long forms of each option.

## Output

Without `-o` (`--output`), the clipboard receives only the converted fragment — the markup for the document body:

```html
<h1>Kimi vs DeepSeek vs Gemini vs local Qwen vs Me</h1>

<p>This is unapologetically a post for nerds like me…</p>

<ul>
<li>Gemini 3.8 Flash</li>
<li>DeepSeek-V4.1-Flash</li>
<li>Kimi-k2.7-code</li>
<li>Jundot-Qwen3.8-27B-oQ8e-mtp (local model)</li>
</ul>
```

With `-o` (`--output`), that same fragment is wrapped in a minimal document and written next to the input, with the extension changed to `.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>SimpleAlbum Blog</title>
</head>
<body>
<h1>Kimi vs DeepSeek vs Gemini vs local Qwen vs Me</h1>

<p>This is unapologetically a post for nerds like me…</p>
</body>
</html>
```

## How it works

A `.docx` file is a ZIP archive. The converter reads these parts directly and walks the XML with streaming events — no Office libraries required:

| Part | Purpose |
| --- | --- |
| `word/document.xml` | The document body: paragraphs, runs, lists, tables, links |
| `word/_rels/document.xml.rels` | Maps hyperlink relationship ids to URLs |
| `word/numbering.xml` | Maps list `numId`s to per-level formats (`ul` vs `ol`) |

Conversion happens in a single pass over `document.xml`. Each paragraph's text is accumulated per run, run formatting is wrapped as `<strong>`/`<em>`/`<u>`/`<s>`/`<sub>`/`<sup>` when the run ends, and list/table state is tracked so consecutive items and rows are grouped into a single element. Empty paragraphs are skipped. The resulting body is line-wrapped to the configured width (`--wrap`, default 90; `0` collapses it to a single line) and then either copied to the system clipboard (via `arboard`) or, with `--output`, placed in a minimal HTML document and written to disk. On Linux, where the clipboard is owned by the setting process, the text is handed to a detached background copy of the program that keeps serving paste requests until the clipboard is overwritten.

## Limitations

- Only the main document body is converted. Headers, footers, footnotes, comments, images and embedded objects are ignored.
- Fonts, colors, sizes and alignment are not preserved — the goal is clean semantic HTML, not a visual reproduction.
- Nested tables are not supported.
- Multiple paragraphs inside one table cell are joined with a space.
- Bold/italic/underline/strikethrough are read from direct run formatting and the built-in `Strong`/`Emphasis` character styles; other named character styles are not resolved.
- The input is assumed to be well-formed OOXML produced by Word or a compatible editor.
- On Linux the clipboard is owned by the process that set it, so the tool re-execs a small background copy of itself to keep serving paste requests after it exits; that helper stops once you copy something else. macOS and Windows keep the clipboard content after the process exits, so no helper is used there.

## Project layout

```
src/main.rs      # converter: parse, transform, wrap, clipboard/file output
src/symbols.rs   # `w:sym` font tables (Symbol, Wingdings, Webdings) -> Unicode
Cargo.toml       # package manifest (deps: arboard, quick-xml, zip)
```

## License

Released under the [MIT License](LICENSE).
