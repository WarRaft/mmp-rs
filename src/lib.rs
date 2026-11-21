//! Reader and writer for Warcraft III minimap icon files, `war3map.mmp`:
//! the icons the game's loading and lobby screens draw on a map's
//! preview (gold mines, neutral buildings, start locations), each where
//! it is and in its color.
//!
//! ```no_run
//! let bytes = std::fs::read("war3map.mmp").unwrap();
//! let file = mmp_rs::parse(&bytes).unwrap();
//! for icon in &file.icons {
//!     println!("{:?} at {}, {}", mmp_rs::kind_name(icon.kind), icon.x, icon.y);
//! }
//! assert_eq!(file.to_bytes().unwrap(), bytes);
//! ```
//!
//! Reading and writing are exact. For people, a file has a text form
//! ([`text`]).

pub mod text;

use parser_rs::bin::{Reader, Writer};
pub use parser_rs::{BinError, BinResult};

/// The format version this crate reads and writes: the only one there is.
pub const VERSION: u32 = 0;

/// What the icons are, by [`Icon::kind`], as the game draws them. Only 0
/// to 2 are in any map at hand; one description has 3 and 4 as creep
/// camps.
pub const KINDS: [(i32, &str); 5] = [
    (0, "gold mine"),
    (1, "neutral building"),
    (2, "start location"),
    (3, "creep camp"),
    (4, "creep camp (other)"),
];

/// The name of icon kind `kind` ([`KINDS`]).
pub fn kind_name(kind: i32) -> Option<&'static str> {
    KINDS.iter().find(|(k, _)| *k == kind).map(|(_, n)| *n)
}

/// One icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icon {
    /// See [`KINDS`].
    pub kind: i32,
    /// From the preview's top left, 0 to 255 across and down.
    pub x: i32,
    pub y: i32,
    /// The file stores blue, green, red, alpha; white (all 255) for
    /// icons not a player's.
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

/// A whole `war3map.mmp`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapIcons {
    /// Always [`VERSION`].
    pub version: u32,
    pub icons: Vec<Icon>,
}

impl MinimapIcons {
    /// The file's bytes.
    pub fn to_bytes(&self) -> BinResult<Vec<u8>> {
        if self.version != VERSION {
            return Err(BinError::new(0, format!("unknown mmp version {}", self.version)));
        }
        let mut w = Writer::new();
        w.u32(self.version);
        w.count(self.icons.len())?;
        for i in &self.icons {
            w.i32(i.kind);
            w.i32(i.x);
            w.i32(i.y);
            w.bytes(&[i.blue, i.green, i.red, i.alpha]);
        }
        Ok(w.into_bytes())
    }
}

/// Reads a `war3map.mmp`. Every byte must be read.
pub fn parse(bytes: &[u8]) -> BinResult<MinimapIcons> {
    let mut r = Reader::new(bytes);
    let version = r.u32()?;
    if version != VERSION {
        return Err(BinError::new(0, format!("unknown mmp version {version} (0 is the only one)")));
    }
    let icons = r.list(16, |r| {
        let (kind, x, y) = (r.i32()?, r.i32()?, r.i32()?);
        let [blue, green, red, alpha] = r.array()?;
        Ok(Icon { kind, x, y, red, green, blue, alpha })
    })?;
    r.finish()?;
    Ok(MinimapIcons { version, icons })
}

#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
