# fancybook

A collection of mdbook plugins to easy the creation of decent technical documents.

* `mdbook-tectonic` a renderer for latex input
* `mdbook-boilerplate` reduces the size of `main.rs` files of preprocessors
* `mdbook-scientific` collects equations, and figures from source, when nested in `$` or `$$`, renders them with `tectonic`
* `mdbook-fishextract` extracts `mermaid` graphs, renders them and injects images back into the markdown/cmark

## Acknowledgements

`mdbook-scientific` and `mdbook-tectonic` (formerly `mdbook-latex`) and `cmark2tex` (formerly `md2tex`) are all forked and heavily refactored/improved versions
with little chance and/or activity of upstreaming.

## Bugs

If you find bugs, please provide a reproducible testcase as a PR, otherwise I am inclined to close. It's a fun project after all that is used for my $work.