use super::*;
use crate::tests::file;

#[test]
fn text_and_back() {
    let f = file();
    let text = to_text(&f);
    assert!(text.ends_with("[icon]\nkind=2\nx=180\ny=32\ncolor=255,3,3,255\n"), "{text}");
    assert_eq!(from_text(&text).file.unwrap(), f);
    let colors = colors(&bni_rs::parse(&text));
    assert_eq!(colors.len(), 2);
    assert_eq!(colors[1].1, [255, 3, 3, 255]);
}

#[test]
fn errors() {
    let p = from_text("[mmp]\nversion=1\n[icon]\nkind=x\ncolor=1,2\nsize=3\n[other]\n");
    let lines: Vec<usize> = p.errors.iter().map(|e| e.line).collect();
    assert_eq!(lines, [2, 4, 5, 6, 7], "{:?}", p.errors);
    assert!(from_text("[x]\n").errors[0].message.contains("starts with [mmp]"));
}
