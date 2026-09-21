use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::exit;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

mod symbols;

const USAGE: &str = concat!(
    "usage: wordtohtml [OPTIONS] <path-to-docx>\n",
    "\n",
    "Converts a Word .docx file to HTML next to the input, e.g. doc.docx -> doc.html.\n",
    "\n",
    "options:\n",
    "  -w, --wrap <columns>  Wrap output near this many characters (default 90).\n",
    "                        Use 0 for a single line; newlines inside code\n",
    "                        blocks are kept either way.\n",
    "  -h, --help            Show this help.",
);

struct Options {
    input: String,
    wrap: usize,
}

fn main() {
    let opts = match parse_args() {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("error: {e}\n\n{USAGE}");
            exit(2);
        }
    };

    let input = Path::new(&opts.input);
    let output = input.with_extension("html");

    let body = match docx_to_html(&opts.input) {
        Ok(body) => body,
        Err(e) => {
            eprintln!("error: {e}");
            exit(1);
        }
    };

    let html = build_document(&body, input, opts.wrap);

    if let Err(e) = std::fs::write(&output, html) {
        eprintln!("error: could not write {}: {e}", output.display());
        exit(1);
    }

    println!("wrote {}", output.display());
}

fn parse_args() -> Result<Options, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut wrap = 90usize;
    let mut input: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-h" | "--help" => {
                println!("{USAGE}");
                exit(0);
            }
            "-w" | "--wrap" => {
                i += 1;
                let value = args.get(i).ok_or("--wrap requires a value")?;
                wrap = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid wrap width: {value}"))?;
            }
            _ if arg.starts_with('-') && arg != "-" => {
                return Err(format!("unknown option: {arg}"));
            }
            _ => {
                if input.is_some() {
                    return Err(format!("unexpected extra argument: {arg}"));
                }
                input = Some(arg.to_string());
            }
        }
        i += 1;
    }

    let input = input.ok_or("missing <path-to-docx>")?;
    Ok(Options { input, wrap })
}

// Build the final HTML document. `width` is the wrap column; 0 means a single
// line with no newlines at all, except inside code blocks where the newlines
// are part of the content.
fn build_document(body: &str, input: &Path, width: usize) -> String {
    if width == 0 {
        wrap_document(&strip_newlines(body), input)
    } else {
        wrap_document(&wrap(body, width), input)
    }
}

// Wrap the converted fragment in a minimal HTML document. The charset
// declaration is essential so Word's typographic characters (e.g. ’) are not
// misinterpreted as Latin-1 and rendered as mojibake such as â€™.
fn wrap_document(body: &str, input: &Path) -> String {
    let title = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{}</title>\n</head>\n<body>\n{}\n</body>\n</html>\n",
        escape(&title),
        body.trim_end()
    )
}

fn docx_to_html(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;

    let mut document = String::new();
    zip.by_name("word/document.xml")?
        .read_to_string(&mut document)?;

    let rels = match zip.by_name("word/_rels/document.xml.rels") {
        Ok(mut f) => {
            let mut xml = String::new();
            f.read_to_string(&mut xml)?;
            parse_rels(&xml)
        }
        Err(_) => HashMap::new(),
    };

    // numId -> per-level list tag ("ul" or "ol")
    let numbering = match zip.by_name("word/numbering.xml") {
        Ok(mut f) => {
            let mut xml = String::new();
            f.read_to_string(&mut xml)?;
            parse_numbering(&xml)
        }
        Err(_) => HashMap::new(),
    };

    Ok(parse_document(&document, &rels, numbering))
}

// Collect a single attribute value from an element, if present.
fn attr_value(e: &BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == key {
            return Some(
                attr.unescape_value()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&attr.value).to_string()),
            );
        }
    }
    None
}

// Word's numbering.xml maps numId -> abstractNumId, and each abstract numbering
// declares a number format per indent level. Turn that into numId -> list tags.
fn parse_numbering(xml: &str) -> HashMap<String, Vec<&'static str>> {
    // abstractNumId -> (ilvl -> tag)
    let mut abstracts: HashMap<String, HashMap<u8, &'static str>> = HashMap::new();
    // numId -> abstractNumId
    let mut nums: HashMap<String, String> = HashMap::new();

    let mut reader = Reader::from_str(xml);
    let mut cur_abstract: Option<String> = None;
    let mut cur_num: Option<String> = None;
    let mut cur_lvl: u8 = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"w:abstractNum" => {
                    cur_abstract = attr_value(&e, b"w:abstractNumId");
                    if let Some(id) = &cur_abstract {
                        abstracts.entry(id.clone()).or_default();
                    }
                }
                b"w:lvl" => {
                    cur_lvl = attr_value(&e, b"w:ilvl")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                }
                b"w:numFmt" => {
                    if let (Some(aid), Some(val)) = (&cur_abstract, attr_value(&e, b"w:val")) {
                        let tag = match val.as_str() {
                            "bullet" | "none" => "ul",
                            _ => "ol",
                        };
                        abstracts
                            .entry(aid.clone())
                            .or_default()
                            .insert(cur_lvl, tag);
                    }
                }
                b"w:num" => cur_num = attr_value(&e, b"w:numId"),
                b"w:abstractNumId" => {
                    if let (Some(nid), Some(aid)) = (&cur_num, attr_value(&e, b"w:val")) {
                        nums.insert(nid.clone(), aid);
                    }
                }
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"w:abstractNum" => cur_abstract = None,
                b"w:num" => cur_num = None,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    let mut map = HashMap::new();
    for (num_id, abstract_id) in nums {
        let tags = abstracts
            .get(&abstract_id)
            .map(|levels| {
                let max = levels.keys().copied().max().unwrap_or(0);
                (0..=max)
                    .map(|i| *levels.get(&i).unwrap_or(&"ul"))
                    .collect()
            })
            .unwrap_or_default();
        map.insert(num_id, tags);
    }
    map
}

// word/_rels/document.xml.rels: Relationship Id -> Target
fn parse_rels(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut reader = Reader::from_str(xml);
    let mut id = String::new();
    let mut target = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"Relationship" {
                    id.clear();
                    target.clear();
                    for attr in e.attributes().flatten() {
                        let value = attr
                            .unescape_value()
                            .map(|v| v.to_string())
                            .unwrap_or_else(|_| String::from_utf8_lossy(&attr.value).to_string());
                        match attr.key.as_ref() {
                            b"Id" => id = value,
                            b"Target" => target = value,
                            _ => {}
                        }
                    }
                    if !id.is_empty() {
                        map.insert(id.clone(), target.clone());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    map
}

// An open list level: its indent level, HTML tag ("ul"/"ol"), and the Word
// numbering id that opened it, so a change of id starts a fresh list.
struct OpenList {
    level: u8,
    tag: &'static str,
    num_id: Option<String>,
}

#[derive(Default)]
struct State {
    html: String,
    paragraph: String,
    heading: u8,
    href: Option<String>,
    link_text: String,
    in_text: bool,

    // Current run text and its formatting, flushed on </w:r>.
    run: String,
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    subscript: bool,
    superscript: bool,
    mono: bool,

    // Paragraph-level code state. `pre` is set by a preformatted/code paragraph
    // style; `mono_only` records whether every text run so far used a monospace
    // font or code character style, so a fully monospace paragraph can become a
    // <pre><code> block instead of prose with inline <code> tags.
    pre: bool,
    mono_only: bool,
    runs: usize,

    // List state. num_id/num_level are set from the current paragraph's numPr
    // and consumed when that paragraph closes.
    numbering: HashMap<String, Vec<&'static str>>,
    num_id: Option<String>,
    num_level: u8,
    list_stack: Vec<OpenList>,

    // Table state. Contents are buffered so rows can be classified as header
    // or body before anything is flushed to the output.
    in_table: bool,
    table: String,
    row: String,
    cell: String,
    row_header: bool,
    table_header: bool,
    first_row_seen: bool,
    tbody_open: bool,
}

fn parse_document(
    xml: &str,
    rels: &HashMap<String, String>,
    numbering: HashMap<String, Vec<&'static str>>,
) -> String {
    let mut reader = Reader::from_str(xml);
    let mut state = State {
        numbering,
        ..Default::default()
    };

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => open(&e, &mut state, rels),
            Ok(Event::Empty(e)) => {
                open(&e, &mut state, rels);
                close(e.name().as_ref(), &mut state);
            }
            Ok(Event::Text(e)) => {
                if state.in_text {
                    // Code content keeps its tabs and newlines; everywhere else
                    // whitespace collapses to single spaces.
                    let text =
                        escape_text(&e.unescape().unwrap_or_default(), state.pre || state.mono);
                    state.run.push_str(&text);
                }
            }
            Ok(Event::End(e)) => close(e.name().as_ref(), &mut state),
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    flush_lists(&mut state);
    merge_adjacent(&state.html)
}

// Word frequently splits a formatted phrase across many runs, which would emit
// runs of adjacent tags such as <em>very</em><em> </em><em>long</em>. Since the
// formatting is identical, stitch them into one tag.
fn merge_adjacent(html: &str) -> String {
    const TAGS: [&str; 7] = ["em", "strong", "u", "s", "sub", "sup", "code"];
    let mut out = html.to_string();
    loop {
        let mut changed = false;
        for tag in TAGS {
            let boundary = format!("</{tag}><{tag}>");
            if out.contains(&boundary) {
                out = out.replace(&boundary, "");
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    out
}

// Join the markup onto one line for --wrap 0, but keep the newlines inside a
// <pre> block: those are the code's own line breaks, not formatting.
fn strip_newlines(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut in_pre = false;
    let mut tag = String::new();

    for c in html.chars() {
        if c == '<' {
            in_tag = true;
            tag.clear();
        }
        if in_tag {
            tag.push(c);
        }
        if c == '>' {
            in_tag = false;
            if tag.starts_with("<pre") {
                in_pre = true;
            } else if tag.starts_with("</pre") {
                in_pre = false;
            }
        }
        if c == '\n' && !in_pre {
            continue;
        }
        out.push(c);
    }
    out
}

// Wrap lines at spaces near 90 characters so the HTML can be read in an
// editor without horizontal scrolling. Newlines are whitespace in HTML.
fn wrap(html: &str, limit: usize) -> String {
    if limit == 0 {
        return html.to_string();
    }

    let mut out = String::new();
    let mut line = String::new();
    let mut len = 0;
    let mut last_space: Option<usize> = None;
    let mut in_tag = false;
    // Code blocks are emitted verbatim: no re-wrapping and no stripping of the
    // leading spaces that carry indentation.
    let mut in_pre = false;
    let mut tag = String::new();

    for c in html.chars() {
        if c == '\n' {
            out.push_str(&line);
            out.push('\n');
            line.clear();
            len = 0;
            last_space = None;
            in_tag = false;
            continue;
        }

        if c == '<' {
            in_tag = true;
            tag.clear();
        }

        if c == ' ' && len == 0 && !in_tag && !in_pre {
            continue; // no leading space on a wrapped line
        }

        line.push(c);
        len += 1;

        if in_tag {
            tag.push(c);
        }

        if c == '>' {
            in_tag = false;
            if tag.starts_with("<pre") {
                in_pre = true;
            } else if tag.starts_with("</pre") {
                in_pre = false;
            }
        }
        if c == ' ' && !in_tag {
            last_space = Some(line.len() - 1);
        }

        // Break at the last space once the line goes past the limit. Tags are
        // never split, so a very long tag or URL can still exceed the limit,
        // and <pre> content is left exactly as it was written.
        if len > limit && !in_tag && !in_pre {
            if let Some(i) = last_space {
                out.push_str(&line[..i]);
                out.push('\n');
                line = line[i + 1..].to_string();
                len = line.chars().count();
                last_space = None;
            }
        }
    }

    out.push_str(&line);
    out
}

fn open(e: &BytesStart, state: &mut State, rels: &HashMap<String, String>) {
    match e.name().as_ref() {
        b"w:p" => {
            state.paragraph.clear();
            state.heading = 0;
            state.pre = false;
            state.mono_only = true;
            state.runs = 0;
            state.num_id = None;
            state.num_level = 0;
        }
        b"w:ilvl" => {
            state.num_level = attr_value(e, b"w:val")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
        b"w:numId" => state.num_id = attr_value(e, b"w:val"),
        b"w:tbl" => {
            flush_lists(state);
            state.table.clear();
            state.table.push_str("<table>\n");
            state.row.clear();
            state.cell.clear();
            state.in_table = true;
            state.table_header = false;
            state.first_row_seen = false;
            state.tbody_open = false;
            state.row_header = false;
        }
        b"w:tblLook" => {
            if matches!(
                attr_value(e, b"w:firstRow").as_deref(),
                Some("1") | Some("true")
            ) {
                state.table_header = true;
            }
        }
        b"w:tr" => {
            state.row.clear();
            state.row_header = state.table_header && !state.first_row_seen;
        }
        b"w:tblHeader" => state.row_header = true,
        b"w:tc" => state.cell.clear(),
        b"w:pStyle" => {
            if let Some(style) = attr_value(e, b"w:val") {
                state.heading = heading_level(&style);
                state.pre = is_pre_style(&style);
            }
        }
        b"w:hyperlink" => {
            state.link_text.clear();
            state.href = None;
            let mut rel_href = None;
            let mut anchor_href = None;
            for attr in e.attributes().flatten() {
                match attr.key.as_ref() {
                    b"r:id" | b"id" => {
                        let id = String::from_utf8_lossy(&attr.value).to_string();
                        rel_href = rels.get(&id).cloned();
                    }
                    // Internal links (TOC entries, cross-references) carry a
                    // bookmark name instead of a relationship id.
                    b"w:anchor" | b"anchor" => {
                        let anchor = String::from_utf8_lossy(&attr.value).to_string();
                        if !anchor.is_empty() {
                            anchor_href = Some(format!("#{anchor}"));
                        }
                    }
                    _ => {}
                }
            }
            state.href = rel_href.or(anchor_href);
        }
        b"w:r" => {
            state.run.clear();
            state.bold = false;
            state.italic = false;
            state.underline = false;
            state.strike = false;
            state.subscript = false;
            state.superscript = false;
            state.mono = false;
        }
        // A monospace family on any script makes the run code.
        b"w:rFonts" => {
            for key in [b"w:ascii".as_slice(), b"w:hAnsi", b"w:cs", b"w:eastAsia"] {
                if attr_value(e, key).is_some_and(|name| is_mono_font(&name)) {
                    state.mono = true;
                    break;
                }
            }
        }
        b"w:b" | b"w:bCs" => state.bold = run_flag(e),
        b"w:i" | b"w:iCs" => state.italic = run_flag(e),
        b"w:u" => state.underline = run_flag(e),
        b"w:strike" | b"w:dstrike" => state.strike = run_flag(e),
        b"w:vertAlign" => match attr_value(e, b"w:val").as_deref() {
            Some("subscript") => {
                state.subscript = true;
                state.superscript = false;
            }
            Some("superscript") => {
                state.superscript = true;
                state.subscript = false;
            }
            _ => {
                state.subscript = false;
                state.superscript = false;
            }
        },
        // Word's built-in character styles for bold/italic text.
        b"w:rStyle" => {
            if let Some(style) = attr_value(e, b"w:val") {
                match style.to_lowercase().as_str() {
                    "strong" => state.bold = true,
                    "emphasis" => state.italic = true,
                    _ => {}
                }
                if is_mono_style(&style) {
                    state.mono = true;
                }
            }
        }
        // Inside code a break stays a newline and a tab keeps its width; both
        // survive verbatim in the emitted <pre>.
        b"w:br" | b"w:cr" => {
            if state.pre || state.mono {
                state.run.push('\n');
            } else {
                state.run.push_str("<br>");
            }
        }
        b"w:tab" => {
            if state.pre || state.mono {
                state.run.push('\t');
            } else {
                state.run.push(' ');
            }
        }
        // A symbol-font glyph such as a Wingdings smiley. Word spells it as a
        // font index rather than a character, so translate it before the run is
        // flushed; otherwise the whole element is lost.
        b"w:sym" => {
            let font = attr_value(e, b"w:font").unwrap_or_default();
            let code =
                attr_value(e, b"w:char").and_then(|v| u32::from_str_radix(v.trim(), 16).ok());
            if let Some(c) = code.and_then(|c| symbols::to_char(&font, c)) {
                // Goes through escape() like any other text: the Symbol table
                // maps some indices to &, < and >.
                state.run.push_str(&escape(&c.to_string()));
            }
        }
        b"w:t" => state.in_text = true,
        _ => {}
    }
}

fn close(name: &[u8], state: &mut State) {
    match name {
        b"w:t" => state.in_text = false,
        b"w:r" => {
            let text = std::mem::take(&mut state.run);
            if !text.is_empty() {
                let blank = text.trim().is_empty();
                // Nest formatting in a fixed order so identical formatting in
                // adjacent runs can be merged later by merge_adjacent().
                let mut out = text;
                if state.subscript {
                    out = format!("<sub>{out}</sub>");
                }
                if state.superscript {
                    out = format!("<sup>{out}</sup>");
                }
                if state.italic {
                    out = format!("<em>{out}</em>");
                }
                if state.bold {
                    out = format!("<strong>{out}</strong>");
                }
                if state.underline {
                    out = format!("<u>{out}</u>");
                }
                if state.strike {
                    out = format!("<s>{out}</s>");
                }
                // A preformatted paragraph wraps the block as a whole, so only
                // inline code gets its own <code> tag here.
                if state.mono && !state.pre {
                    out = format!("<code>{out}</code>");
                }
                if state.href.is_some() {
                    state.link_text.push_str(&out);
                } else {
                    state.paragraph.push_str(&out);
                }

                state.runs += 1;
                // Whitespace-only runs (Word often splits those out) must not
                // stop a code paragraph from being recognised.
                if !state.mono && !blank {
                    state.mono_only = false;
                }
            }
        }
        b"w:hyperlink" => {
            let text = std::mem::take(&mut state.link_text);
            match state.href.take() {
                Some(href) => {
                    state.paragraph.push_str(&format!(
                        "<a href=\"{}\">{}</a>",
                        escape_attr(&href),
                        text
                    ));
                }
                None => state.paragraph.push_str(&text),
            }
        }
        b"w:tc" => {
            let tag = if state.row_header { "th" } else { "td" };
            let cell = std::mem::take(&mut state.cell);
            state.row.push_str(&format!("<{tag}>{cell}</{tag}>\n"));
        }
        b"w:tr" => {
            let row = std::mem::take(&mut state.row);
            let row = format!("<tr>\n{row}</tr>\n");
            if state.row_header && !state.first_row_seen {
                state.table.push_str("<thead>\n");
                state.table.push_str(&row);
                state.table.push_str("</thead>\n");
            } else {
                if !state.tbody_open {
                    state.table.push_str("<tbody>\n");
                    state.tbody_open = true;
                }
                state.table.push_str(&row);
            }
            state.first_row_seen = true;
        }
        b"w:tbl" => {
            if state.tbody_open {
                state.table.push_str("</tbody>\n");
            }
            state.table.push_str("</table>\n\n");
            let table = std::mem::take(&mut state.table);
            state.html.push_str(&table);
            state.in_table = false;
            state.tbody_open = false;
            state.row_header = false;
        }
        b"w:p" => {
            let content = std::mem::take(&mut state.paragraph);
            if state.in_table {
                if !content.is_empty() {
                    if !state.cell.is_empty() {
                        state.cell.push(' ');
                    }
                    state.cell.push_str(&content);
                }
            } else if state.pre
                || (state.mono_only
                    && state.runs > 0
                    && state.heading == 0
                    && state.num_id.is_none())
            {
                // A code paragraph: keep its text verbatim. Runs in a
                // preformatted paragraph were never wrapped in <code>, but a
                // paragraph that is entirely monospace was, so unwrap those
                // individual tags before wrapping the block as a whole.
                flush_lists(state);
                let content = if state.pre {
                    content
                } else {
                    content.replace("<code>", "").replace("</code>", "")
                };
                let content = if content.trim().is_empty() {
                    "\n".to_string()
                } else {
                    content
                };
                state
                    .html
                    .push_str(&format!("<pre><code>{content}</code></pre>\n\n"));
            } else if state.num_id.is_some() && !content.is_empty() {
                let level = state.num_level;
                let tag = list_tag(&state.numbering, state.num_id.as_deref(), level);
                emit_list_item(state, level, tag, content);
            } else {
                flush_lists(state);
                if !content.is_empty() {
                    let tag = if state.heading >= 1 {
                        format!("h{}", state.heading.min(6))
                    } else {
                        "p".to_string()
                    };
                    // A single trailing blank line separates each block. Empty
                    // paragraphs in the document are ignored so they cannot
                    // double up that spacing.
                    state
                        .html
                        .push_str(&format!("<{tag}>{content}</{tag}>\n\n"));
                }
            }
            state.num_id = None;
            state.num_level = 0;
        }
        _ => {}
    }
}

// Tag for a list level, falling back to a bullet list when unspecified.
fn list_tag(
    numbering: &HashMap<String, Vec<&'static str>>,
    num_id: Option<&str>,
    level: u8,
) -> &'static str {
    num_id
        .and_then(|id| numbering.get(id))
        .and_then(|tags| tags.get(level as usize).copied())
        .unwrap_or("ul")
}

// Close every open list, including the still-open last item.
fn flush_lists(state: &mut State) {
    if state.list_stack.is_empty() {
        return;
    }
    while let Some(list) = state.list_stack.pop() {
        state.html.push_str("</li>\n");
        state.html.push_str(&format!("</{}>\n", list.tag));
    }
    state.html.push('\n');
}

// Append one list item, opening/closing lists so consecutive items at the same
// level share a single <ul>/<ol> and deeper items nest inside their parent.
fn emit_list_item(state: &mut State, level: u8, tag: &'static str, content: String) {
    let num_id = state.num_id.clone();

    // Close any lists deeper than this item's level.
    while state.list_stack.last().is_some_and(|l| l.level > level) {
        let list = state.list_stack.pop().unwrap();
        state.html.push_str("</li>\n");
        state.html.push_str(&format!("</{}>\n", list.tag));
    }

    // A list at this level is already open: close the previous item, then
    // restart the list if its type or numbering instance changed.
    let restart = match state.list_stack.last() {
        Some(list) if list.level == level => {
            state.html.push_str("</li>\n");
            list.tag != tag || list.num_id != num_id
        }
        _ => false,
    };
    if restart {
        let list = state.list_stack.pop().unwrap();
        state.html.push_str(&format!("</{}>\n", list.tag));
    }

    // Open any missing list levels up to this item's level.
    while state.list_stack.len() <= level as usize {
        let idx = state.list_stack.len() as u8;
        let lvl_tag = list_tag(&state.numbering, num_id.as_deref(), idx);
        state.html.push_str(&format!("<{lvl_tag}>\n"));
        state.list_stack.push(OpenList {
            level: idx,
            tag: lvl_tag,
            num_id: num_id.clone(),
        });
    }

    state.html.push_str(&format!("<li>{content}"));
}

// A boolean toggle such as <w:b/> or <w:i/>. A missing or true value means
// enabled; "0"/"false"/"off" means explicitly disabled.
fn run_flag(e: &BytesStart) -> bool {
    match attr_value(e, b"w:val").as_deref() {
        None => true,
        Some(v) => !matches!(v, "0" | "false" | "off" | "none"),
    }
}

// "Heading1", "heading 2", "Heading-3" -> 1, 2, 3
fn heading_level(style: &str) -> u8 {
    let style = style.trim().to_lowercase();
    // Split the trailing digits without slicing across a UTF-8 boundary, so a
    // non-ASCII style name (e.g. "titreé") cannot panic.
    let name = style.trim_end_matches(|c: char| c.is_ascii_digit());
    let digit = &style[name.len()..];
    let name = name.trim_end_matches([' ', '-', '_']);
    match (name, digit.parse::<u8>()) {
        ("heading" | "h", Ok(level)) => level.min(6),
        _ => 0,
    }
}

// Escape a value for use inside a double-quoted attribute. Only the double
// quote differs from text escaping.
fn escape_attr(s: &str) -> String {
    escape(s).replace('"', "&quot;")
}

// Strip separators and case from a Word style id so "HTML Preformatted",
// "htmlpreformatted" and "Source-Code" can all be matched the same way.
fn normalize_style(style: &str) -> String {
    style
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

// Paragraph styles that mean "preformatted text" in Word and common exporters.
fn is_pre_style(style: &str) -> bool {
    matches!(
        normalize_style(style).as_str(),
        "pre"
            | "preformatted"
            | "htmlpreformatted"
            | "htmlpre"
            | "code"
            | "codeblock"
            | "sourcecode"
            | "macro"
            | "macrotext"
            | "plaintext"
    )
}

// Character styles that mark inline or block code.
fn is_mono_style(style: &str) -> bool {
    is_pre_style(style)
        || matches!(
            normalize_style(style).as_str(),
            "htmlcode" | "verbatim" | "verbatimchar" | "codephrase" | "inlinecode"
        )
}

// Heuristic: does a Word font name denote a monospace family? Word stores the
// family as free text ("Consolas", "Courier New", "JetBrains Mono"), so names
// are matched by substring plus a few exact names that lack a giveaway word.
fn is_mono_font(name: &str) -> bool {
    let name = name.trim().to_lowercase();
    const HINTS: [&str; 12] = [
        "mono",
        "courier",
        "consolas",
        "menlo",
        "monaco",
        "console",
        "typewriter",
        "fixed",
        "terminal",
        "inconsolata",
        "cascadia",
        "andale",
    ];
    if HINTS.iter().any(|hint| name.contains(hint)) {
        return true;
    }
    matches!(
        name.as_str(),
        "fira code" | "source code pro" | "anonymous pro" | "iosevka" | "input" | "hack" | "m+ 1m"
    )
}

fn escape(s: &str) -> String {
    escape_text(s, false)
}

// Escape text for HTML content. `preserve_ws` keeps tabs and newlines for code
// (they survive as written inside <pre>); otherwise whitespace is collapsed so
// the generated HTML's only newlines are the structural ones.
fn escape_text(s: &str, preserve_ws: bool) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            // Keep quotes as plain ASCII in text content so the source reads
            // naturally rather than using entities.
            '"' => out.push('"'),
            // Word stores typographic quotes and apostrophes: ’/‘ -> ' and
            // “/” -> ".
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{201C}' | '\u{201D}' => out.push('"'),
            '\u{2013}' => out.push_str("&ndash;"),
            '\u{2014}' => out.push_str("&mdash;"),
            '\u{2026}' => out.push_str("&hellip;"),
            '\u{00A0}' => out.push_str("&nbsp;"),
            '\n' | '\r' | '\t' => {
                if preserve_ws {
                    out.push(c);
                } else {
                    out.push(' ');
                }
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_fonts_are_detected() {
        for mono in [
            "Consolas",
            "Courier New",
            "JetBrains Mono",
            "Fira Code",
            "Source Code Pro",
            "Hack",
        ] {
            assert!(is_mono_font(mono), "{mono} should be monospace");
        }
        for proportional in ["Calibri", "Cambria", "Encode Sans", "Input Sans"] {
            assert!(
                !is_mono_font(proportional),
                "{proportional} is not monospace"
            );
        }
    }

    #[test]
    fn code_styles_are_detected() {
        for style in [
            "HTMLPreformatted",
            "HTML Preformatted",
            "Source Code",
            "Code",
        ] {
            assert!(is_pre_style(style), "{style} should be a pre style");
        }
        for style in ["HTMLCode", "VerbatimChar", "Code"] {
            assert!(is_mono_style(style), "{style} should mark inline code");
        }
        assert!(!is_pre_style("Heading1"));
        assert!(!is_mono_style("Strong"));
        assert!(!is_mono_style("Emphasis"));
    }

    #[test]
    fn wrap_leaves_pre_content_alone() {
        let html = "<pre><code>    indented\n  line\n</code></pre>\n\n<p>plain words here</p>";
        let wrapped = wrap(html, 20);
        assert!(
            wrapped.contains("<pre><code>    indented\n  line\n</code></pre>"),
            "pre content was altered: {wrapped}"
        );
    }

    #[test]
    fn wrap_still_breaks_prose() {
        let wrapped = wrap("<p>one two three four five</p>", 10);
        assert!(wrapped.contains('\n'), "long prose should wrap: {wrapped}");
    }

    #[test]
    fn wrap_zero_keeps_code_newlines() {
        let html = "<pre><code>a\nb</code></pre>\n\n<p>x</p>";
        assert_eq!(strip_newlines(html), "<pre><code>a\nb</code></pre><p>x</p>");
    }

    #[test]
    fn escape_keeps_whitespace_only_for_code() {
        assert_eq!(escape_text("a\tb\nc", true), "a\tb\nc");
        assert_eq!(escape_text("a\tb\nc", false), "a b c");
        assert_eq!(escape_text("a < b", true), "a &lt; b");
    }
}
