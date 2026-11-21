//! The text form of a minimap icon file: BNI, a `[mmp]` section with the
//! version, then an `[icon]` section per icon, in file order.
//!
//! ```ini
//! [mmp]
//! version=0
//!
//! [icon]
//! kind=2
//! x=180
//! y=32
//! color=255,3,3,255
//! ```
//!
//! `color` is red, green, blue, alpha (the file stores blue first). The
//! kind's name is for an editor to show; [`colors`] gives the colors'
//! places for an editor's color picker.

use bni_rs::Document;
use bni_rs::ast::Item;
use parser_rs::{ParseError, Span};

use crate::{Icon, MinimapIcons, VERSION};

/// The first section's name.
pub const HEADER: &str = "mmp";

/// An icon's section.
pub const ICON: &str = "icon";

pub fn to_text(file: &MinimapIcons) -> String {
    let mut out = format!("[{HEADER}]\nversion={}\n", file.version);
    for i in &file.icons {
        out += &format!(
            "\n[{ICON}]\nkind={}\nx={}\ny={}\ncolor={},{},{},{}\n",
            i.kind, i.x, i.y, i.red, i.green, i.blue, i.alpha
        );
    }
    out
}

/// Is `doc` a minimap icon file's text form: is its first section
/// `[mmp]`?
pub fn is_minimap_text(doc: &Document) -> bool {
    doc.sections.first().is_some_and(|s| s.name.text == HEADER)
}

/// What [`from_text`] read: the file when the text has no errors, and
/// every error, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub file: Option<MinimapIcons>,
    pub errors: Vec<ParseError>,
}

pub fn from_text(text: &str) -> Parsed {
    from_bni(&bni_rs::parse(text))
}

/// [`from_text`] of a document already parsed. Its BNI errors come first.
pub fn from_bni(doc: &Document) -> Parsed {
    let mut errors = doc.errors.clone();
    let Some((head, icons)) = doc.sections.split_first().filter(|(h, _)| h.name.text == HEADER) else {
        let span = doc.sections.first().map_or(Span::new(0, 0, 0, 1), |s| s.name.span);
        errors.push(error(span, format!("the file starts with [{HEADER}]")));
        return Parsed { file: None, errors };
    };
    let mut err = |item: &Item, m: String| errors.push(error(item_span(item), m));
    for item in &doc.preamble {
        err(item, format!("a field outside a section: the file starts with [{HEADER}]"));
    }
    for item in &head.items {
        match item.key.as_ref().map(|k| k.text.trim()) {
            Some("version") if item.value_text().trim() == VERSION.to_string() => {}
            Some("version") => err(item, format!("the version is {VERSION}, the only one there is")),
            _ => err(item, format!("[{HEADER}] has only `version`")),
        }
    }
    let mut file = MinimapIcons { version: VERSION, icons: Vec::new() };
    let mut bad_sections = Vec::new();
    for section in icons {
        if section.name.text != ICON {
            bad_sections.push(error(section.name.span, format!("after [{HEADER}], every section is [{ICON}]")));
            continue;
        }
        let mut icon = Icon { kind: 0, x: 0, y: 0, red: 255, green: 255, blue: 255, alpha: 255 };
        let mut seen: Vec<&str> = Vec::new();
        for item in &section.items {
            let Some(key) = item.key.as_ref().map(|k| k.text.trim()) else {
                err(item, "a field without a name".into());
                continue;
            };
            if seen.contains(&key) {
                err(item, format!("`{key}` is set twice"));
                continue;
            }
            seen.push(key);
            let value = item.value_text();
            let value = value.trim();
            let int = |v: &str| v.parse::<i32>().map_err(|_| format!("`{key}` is a whole number"));
            let result = match key {
                "kind" => int(value).map(|n| icon.kind = n),
                "x" => int(value).map(|n| icon.x = n),
                "y" => int(value).map(|n| icon.y = n),
                "color" => color(item).map(|[r, g, b, a]| (icon.red, icon.green, icon.blue, icon.alpha) = (r, g, b, a)),
                _ => Err(format!("no field `{key}`: an icon has kind, x, y, color")),
            };
            if let Err(m) = result {
                err(item, m);
            }
        }
        file.icons.push(icon);
    }
    errors.extend(bad_sections);
    errors.sort_by_key(|e| (e.line, e.col));
    let file = errors.is_empty().then_some(file);
    Parsed { file, errors }
}

fn color(item: &Item) -> Result<[u8; 4], String> {
    let values: Vec<u8> = item
        .fields
        .iter()
        .map(|f| f.text().trim().parse::<u8>())
        .collect::<Result<_, _>>()
        .map_err(|_| "`color` is red, green, blue, alpha: numbers from 0 to 255".to_string())?;
    values.try_into().map_err(|_| "`color` has four values: red, green, blue, alpha".to_string())
}

/// Every `color` that reads as one: the span of its value, and red,
/// green, blue, alpha.
pub fn colors(doc: &Document) -> Vec<(Span, [u8; 4])> {
    if !is_minimap_text(doc) {
        return Vec::new();
    }
    doc.sections[1..]
        .iter()
        .flat_map(|s| &s.items)
        .filter(|i| i.key.as_ref().map(|k| k.text.trim()) == Some("color"))
        .filter_map(|item| {
            let rgba = color(item).ok()?;
            let (first, last) = (item.fields.first()?.span, item.fields.last()?.span);
            Some((Span::new(first.start_line, first.start_col, last.end_line, last.end_col), rgba))
        })
        .collect()
}

fn error(span: Span, message: impl Into<String>) -> ParseError {
    let end_col = if span.end_line == span.start_line { span.end_col.max(span.start_col + 1) } else { span.start_col + 1 };
    ParseError { message: message.into(), line: span.start_line, col: span.start_col, end_col }
}

fn item_span(item: &Item) -> Span {
    let start = item.key.as_ref().map_or(item.eq, |k| k.span);
    let end = item.fields.last().map_or(item.eq, |f| f.span);
    Span::new(start.start_line, start.start_col, end.end_line, end.end_col)
}

#[cfg(test)]
#[path = "text.test.rs"]
mod tests;
