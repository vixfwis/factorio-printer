### Factorio image (blue)printing tool
```
Usage: factorio-printer [OPTIONS] [FILE]

Arguments:
  [FILE]  Input image file. '-': stdin

Options:
  -o <FILE>                    Output image in PNG format. '-': stdout, '!': disable [default: output.png]
  -b <FILE>                    Output blueprint. '-': stdout, '!': disable [default: blueprint.txt]
  -s, --scale <scale>          Scaling factor [default: 1.0]
  -p, --preset <preset>        Built-in tilesets [default: colorcoding] [possible values: base, colorcoding]
  -t, --tileset <FILE>         Alternative tileset
      --export-tileset <FILE>  Export current tileset in CSV format
      --alpha <VALUE>          Pixels with alpha channel less that <VALUE> are skipped [default: 128]
      --split <SIDE>           Split blueprint into squares of <SIDE>^2 size. 0 means no splitting [default: 0]
  -h, --help                   Print help
```

##### Tileset
By default, there are 2 presets: base game and Color Coding mod. 
Those 2 are in the source code. 
It's possible to use custom tileset - the format is CSV with headers. 
For exact format - use `--export-tileset` option

##### Split
Blueprint will be split into squares and exported as a book. 
Each blueprint in the book will have X,Y coords in its name and icons
By default splitting is disabled

### Build
* [Get Rust toolchain](https://www.rust-lang.org/tools/install)
* `cargo build --release`
* Binary will be in `target/release/factorio-printer`