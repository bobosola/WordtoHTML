# wordtohtml

A small command-line tool that converts a Microsoft Word `.docx` document into a
clean, readable HTML file. It is designed for turning blog posts written in Word
into HTML you can paste into a website or CMS.

The tool writes the result next to the input file, using the same base name with
an `.html` extension:

```
SimpleAlbum Blog.docx  ->  SimpleAlbum Blog.html
```

## Features

- **Paragraphs** -> `<p>`
- **Headings** (`Heading1` … `Heading6`, or `Heading 1` …) -> `<h1>` … `<h6>`
- **Bulleted and numbered lists** -> `<ul>` / `<ol>` with `<li>` items,
  including nested list levels
- **Tables** -> `<table>` with `<thead>`/`<tbody>`, `<tr>`, `<td>` and `<th>`
  for the header row
- **Bold** -> `<strong>`, **italic** -> `<em>`, **underline** -> `<u>`,
  **strikethrough** -> `<s>`, **subscript** -> `<sub>`, **superscript** -> `<sup>`
  - Word's built-in `Strong` and `Emphasis` character styles are recognised
  - Adjacent runs with identical formatting are merged into one tag
- **Line breaks** (`<w:br/>`, `<w:cr/>`) -> `<br>`, and tabs -> a space
- **Internal links** (`w:anchor`, e.g. table-of-contents entries) -> `<a href="#bookmark">`
- **Hyperlinks** -> `<a href="...">`
- **Typographic punctuation** cleaned up for the web:
  - `’` / `‘` -> `'`
  - `”` / `“` -> `"`
  - en dash -> `&ndash;`, em dash -> `&mdash;`
  - ellipsis -> `&hellip;`, non-breaking space -> `&nbsp;`
- **UTF-8 output** with `<meta charset="utf-8">` so characters render
  correctly instead of as mojibake such as `â€™`
- Readable formatting: long lines are wrapped near 90 characters (configurable
  with `--wrap`; `--wrap 0` disables wrapping for single-line output) and blocks
  are separated by a single blank line

## Requirements

- Rust (edition 2021). Built and tested with `cargo 1.98`.
- No external tools or Office installation are needed.

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
| `-w, --wrap <columns>` | Wrap the output near this many characters (default `90`). Use `0` for a single line with no newlines. |
| `-h, --help` | Show usage. |

Example:

```sh
./target/release/wordtohtml "/Users/me/Desktop/SimpleAlbum Blog.docx"
# wrote /Users/me/Desktop/SimpleAlbum Blog.html

# wrap at 60 columns
./target/release/wordtohtml --wrap 60 post.docx

# one long line, no newlines (handy for embedding/minifying)
./target/release/wordtohtml --wrap 0 post.docx
```

Run it without a release build:

```sh
cargo run --release -- --wrap 100 "/path/to/document.docx"
```

## Output

The generated file is a complete HTML document:

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

<ul>
<li>Gemini 3.8 Flash</li>
<li>DeepSeek-V4.1-Flash</li>
<li>Kimi-k2.7-code</li>
<li>Jundot-Qwen3.8-27B-oQ8e-mtp (local model)</li>
</ul>
</body>
</html>
```

## How it works

A `.docx` file is a ZIP archive. The converter reads these parts directly and
walks the XML with streaming events — no Office libraries required:

| Part | Purpose |
| --- | --- |
| `word/document.xml` | The document body: paragraphs, runs, lists, tables, links |
| `word/_rels/document.xml.rels` | Maps hyperlink relationship ids to URLs |
| `word/numbering.xml` | Maps list `numId`s to per-level formats (`ul` vs `ol`) |

Conversion happens in a single pass over `document.xml`. Each paragraph's text
is accumulated per run, run formatting is wrapped as
`<strong>`/`<em>`/`<u>`/`<s>`/`<sub>`/`<sup>` when the run ends, and list/table
state is tracked so consecutive items and rows are grouped into a single element.
Empty paragraphs are skipped. The resulting body is line-wrapped to the
configured width (`--wrap`, default 90; `0` collapses it to a single line) and
placed in a minimal HTML document before being written to disk.

## Limitations

- Only the main document body is converted. Headers, footers, footnotes,
  comments, images and embedded objects are ignored.
- Fonts, colors, sizes and alignment are not preserved — the goal is clean
  semantic HTML, not a visual reproduction.
- Nested tables are not supported.
- Multiple paragraphs inside one table cell are joined with a space.
- Bold/italic/underline/strikethrough are read from direct run formatting and
  the built-in `Strong`/`Emphasis` character styles; other named character
  styles are not resolved.
- The input is assumed to be well-formed OOXML produced by Word or a compatible
  editor.

## Project layout

```
src/main.rs   # entire converter
Cargo.toml    # package manifest (deps: quick-xml, zip)
```

## License

Released under the [MIT License](LICENSE).
