# Code Review: wordtohtml (`src/main.rs`)

This review covers confirmed bugs, edge cases, potential panics, and structural/performance improvements identified in the codebase.

---

## 1. Definite Bug: Potential Panic on Non-ASCII Style Names

In `heading_level`:

```rust
fn heading_level(style: &str) -> u8 {
    let style = style.trim().to_lowercase();
    let (name, digit) = style.split_at(style.len().saturating_sub(1));
    let name = name.trim_end_matches([' ', '-', '_']);
    match (name, digit.parse::<u8>()) {
        ("heading" | "h", Ok(level)) => level.min(6),
        _ => 0,
    }
}
```

- **Bug**: `style.len()` returns the length in **bytes**, not Unicode scalar characters.
- If a document contains a paragraph style ending in a multi-byte character (e.g. localized styles in French, Russian, or custom user styles like `"titreé"` or `"заголовок"`), `style.len().saturating_sub(1)` splits across the byte sequence of a multi-byte UTF-8 character.
- In Rust, calling `str::split_at` on a non-character boundary immediately triggers a **thread panic and process crash**.
- **Fix**: Safely split using `char_indices()`, inspect `style.chars().last()`, or parse the numeric suffix by splitting on whitespace/delimiters.

---

## 2. Behavioral & Data Loss Issues

### a. Soft Line Breaks (`<w:br/>`) and Tabs (`<w:tab/>`) are Ignored
In Word, users frequently insert soft line breaks with <kbd>Shift</kbd>+<kbd>Enter</kbd> (`<w:br/>`) or align text using tabs (`<w:tab/>`).
- Currently, neither `open()` nor `close()` handles `w:br` or `w:tab`.
- Because they contain no `Event::Text`, the text before and after is **concatenated without a space or break**:
  ```
  "First line"<w:br/>"Second line"  -->  "First lineSecond line"
  ```
- **Fix**: Map `<w:br/>` to `<br>` (or a space) and `<w:tab/>` to a space or `&emsp;`.

### b. Internal Bookmark / TOC Links are Lost
For hyperlinks, the code only inspects `r:id`:
```rust
if key == b"r:id" || key == b"id" {
    let id = String::from_utf8_lossy(&attr.value).to_string();
    state.href = rels.get(&id).cloned();
}
```
- In Word, internal document links (such as Table of Contents links or cross-references) use `w:anchor="BookmarkName"` without an `r:id`.
- For these links, `state.href` remains `None`, so the hyperlink tag is completely discarded and only the raw text is emitted.
- **Fix**: Check for `w:anchor` / `anchor` and emit `<a href="#BookmarkName">`.

### c. Hardcoded XML Namespace Prefixes (`w:` and `r:`)
- The parser matches exact byte literals: `b"w:p"`, `b"w:r"`, `b"w:t"`, `b"w:val"`, `b"r:id"`.
- While Microsoft Word uses `w:` and `r:`, the OpenXML specification allows any prefix mapping (e.g., `<word:p xmlns:word="...">` or default namespaces `<p xmlns="...">`). Documents exported by LibreOffice, Google Docs, Pandoc, or Python-docx can occasionally use different prefixes.
- **Fix**: Match on the local tag name (after any `:`).

---

## 3. Table & List Limitations

### a. Nested Tables Cause Premature Exit
- Table state is tracked with a flat `in_table: bool`.
- If a document contains a table inside a table cell, closing the inner `</w:tbl>` sets `state.in_table = false`.
- Any remaining cells/rows of the outer parent table will then be parsed as regular top-level paragraphs, corrupting the document output.

### b. Missing Table Cell Spans (`colspan` / `rowspan`)
- Merged table cells in Word use `<w:gridSpan w:val="N"/>` (horizontal) and `<w:vMerge/>` (vertical).
- Because these are ignored, merged cells are emitted as standard 1×1 `<td>` cells, shifting column alignments in the resulting HTML.

### c. Adjacent Distinct Lists Merge
- If two distinct lists of the same type (e.g., two separate bullet lists created with different `numId`s) appear without a standard paragraph between them, `emit_list_item()` sees `open_tag == tag` and continues the existing `<ul>` without closing and reopening it.

---

## 4. Performance & Memory Inefficiencies

### a. Full In-Memory String Loading
In `docx_to_html`:
```rust
let mut document = String::new();
zip.by_name("word/document.xml")?.read_to_string(&mut document)?;
```
- A large Word document can have a 50MB–100MB `document.xml`.
- Reading the entire file into a `String` requires buffering the whole decompressed XML before parsing begins.
- `quick_xml::Reader` supports reading directly from a streaming reader (`Reader::from_reader(zip.by_name(...)?)`), which requires near-zero extra memory.

### b. Quadratic String Allocation in `merge_adjacent`
```rust
fn merge_adjacent(html: &str) -> String {
    let mut out = html.to_string();
    loop {
        let next = out
            .replace("</em><em>", "")
            .replace("</strong><strong>", "");
        if next == out {
            break;
        }
        out = next;
    }
    out
}
```
- Each call to `.replace()` re-allocates and copies the entire document string.
- If a document has 100 formatting runs, this loop can allocate and copy the entire HTML string hundreds of times.

---

## 5. Formatting & Output Considerations

### a. Hard Line-Wrapping at 90 Characters (`wrap()`)
- In HTML, newlines are rendered as single spaces.
- However, if the generated HTML is pasted into a blog engine or Markdown renderer that treats newlines as hard breaks (like GitHub-flavored Markdown or Ghost), the 90-character wrap introduces unintended line breaks into the rendered post.
- If the HTML is meant for web publishing, preserving paragraphs as single unwrapped lines (leaving word wrap to the editor/CSS) is often safer.

### b. Other Common Word Formatting Ignored
The following formatting is commonly used in Word blog drafts but currently dropped:
- `<w:u>` (Underline) -> `<u>`
- `<w:strike>` (Strikethrough) -> `<s>` or `<del>`
- `<w:vertAlign w:val="subscript|superscript"/>` -> `<sub>` / `<sup>`
- Monospace / Code font runs (e.g. Courier, Consolas) -> `<code>`

---

## Summary Checklist

| Issue | Severity | Impact |
|---|---|---|
| `style.split_at(...)` UTF-8 panic | **High** | Crashes on non-ASCII style names |
| Missing `<w:br/>` & `<w:tab/>` | **Medium** | Missing spaces/breaks between words |
| Dropped `w:anchor` links | **Low-Medium** | Internal bookmarks/TOC links lose `<a>` tag |
| Hardcoded `w:`/`r:` prefixes | **Low-Medium** | Incompatibility with non-MS Word generators |
| Nested tables corruption | **Low** | Malformed HTML if nested tables exist |
| Multiple string clones in `merge_adjacent` | **Low** | Slower processing on long documents |
