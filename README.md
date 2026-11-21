# mmp-rs

Reader and writer for Warcraft III minimap icon files, `war3map.mmp`:
the icons the game's loading and lobby screens draw on a map's preview
(gold mines, neutral buildings, start locations), each where it is and
in its color. A library, a small CLI and a text form; the
[wc3-rs](../wc3-rs) editor opens `.mmp` files as that text.

Reading and writing are exact, and so is the text form, on every map of
the game and the corpus (208 files, 3 804 icons).

## Format

| type | field |
|---|---|
| u32 | version: 0, the only one |
| u32 | the number of icons |
| | then each icon: |
| i32 | its kind (below) |
| i32 × 2 | x, y: from the preview's top left, 0 to 255 |
| u8 × 4 | color: blue, green, red, alpha; white for icons not a player's |

**Kinds** (`KINDS`): 0 gold mine, 1 neutral building, 2 start location.
Only these are in any map at hand; one description has 3 and 4 as creep
camps.

## Text

```ini
[mmp]
version=0

[icon]
kind=2
x=180
y=32
color=255,3,3,255
```

`color` is red, green, blue, alpha. The kind's name is for an editor to
show; `text::colors` gives each color's place, for a color picker (the
wc3-rs editor has both).

## CLI

```sh
mmp-rs war3map.mmp          # read and write back: a check
mmp-rs --list war3map.mmp   # kind, x, y, color, tab-separated
```

## Tests

```sh
cargo test
MMP_CORPUS=<folder of map folders> cargo test --release corpus -- --ignored
```
