# cmark2tex

[![crates badge][crates-badge]][crates.io]
[![docs badge][docs-badge]][docs]

[crates.io]: https://crates.io/crates/cmark2tex
[crates-badge]: https://img.shields.io/badge/crates.io-v0.1.3-orange.svg?longCache=true

[docs]: https://docs.rs/crate/cmark2tex/0.1.3
[docs-badge]: https://docs.rs/cmark2tex/badge.svg

A small utility to convert markdown files to tex. Forked originally from [md2pdf](https://gitea.tforgione.fr/tforgione/md2pdf/), refocused on `mdbook` conversions.

Used by [mdbook-tectonic](https://github.com/drahnr/mdbook-tectonic) to generate PDF's.

Forked from <https://github.com/lbeckman314/md2tex> forked from <https://gitea.tforgione.fr/tforgione/md2pdf>.

## Usage

### Lib

See [docs.rs][docs]. 

### CLI Tool

```sh
cmark2tex -i input.md -o output.tex
```
