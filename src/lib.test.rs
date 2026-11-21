use super::*;

pub(crate) fn file() -> MinimapIcons {
    MinimapIcons {
        version: 0,
        icons: vec![
            Icon { kind: 0, x: 46, y: 235, red: 255, green: 255, blue: 255, alpha: 255 },
            Icon { kind: 2, x: 180, y: 32, red: 255, green: 3, blue: 3, alpha: 255 },
        ],
    }
}

#[test]
fn reads_and_writes_back() {
    let f = file();
    let bytes = f.to_bytes().unwrap();
    assert_eq!(bytes.len(), 8 + 2 * 16);
    // Blue, green, red, alpha, as stored.
    assert_eq!(&bytes[36..40], &[3, 3, 255, 255]);
    assert_eq!(parse(&bytes).unwrap(), f);
    assert_eq!(kind_name(2), Some("start location"));
}

#[test]
fn bad_files() {
    let err = |b: &[u8]| parse(b).unwrap_err().to_string();
    assert!(err(&[1, 0, 0, 0, 0, 0, 0, 0]).contains("unknown mmp version 1"));
    assert!(err(&[0, 0, 0, 0, 9, 0, 0, 0]).contains("doesn't fit"));
}

/// `MMP_CORPUS` is a folder of `<map>/war3map.mmp`.
#[test]
#[ignore]
fn corpus() {
    let dir = std::env::var("MMP_CORPUS").expect("MMP_CORPUS");
    let (mut n, mut icons, mut bad) = (0, 0, Vec::new());
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path().join("war3map.mmp");
        let Ok(bytes) = std::fs::read(&path) else { continue };
        n += 1;
        match parse(&bytes) {
            Ok(f) if f.to_bytes().unwrap() != bytes => bad.push(format!("{}: differs", path.display())),
            Ok(f) => {
                icons += f.icons.len();
                let back = text::from_text(&text::to_text(&f));
                if back.file.map(|f| f.to_bytes().unwrap()) != Some(bytes.clone()) {
                    bad.push(format!("{}: differs through text: {:?}", path.display(), back.errors));
                }
            }
            Err(e) => bad.push(format!("{}: {e}", path.display())),
        }
    }
    println!("{n} files, {icons} icons");
    assert!(bad.is_empty(), "{} of {n}:\n{}", bad.len(), bad.join("\n"));
}
