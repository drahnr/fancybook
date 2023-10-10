# mdbook-fishextract plugin

This plugin adds functionality to `mdbook` for extracting `mermaid`. 

## Install

Compile this crate and add the `mdbook-fishextract` to your search path. 

```toml
[preprocessor.fishextract]
renderer = ["tectonic","markdown","latex"]
assets = "src/generated-artifacts"

```

## Prerequisites

* `npm install -g @mermaid-js/mermaid-cli` which installs a binary `mmdc` which needs to be in your `$PATH`.

## Syntax

For block equation rendering use the following syntax

```md
```mermaid
..
```

and replaces them by an image link

```md
![mermaid graph {chapter}](src/assets/fishextract_{chapter|replace('.' -> '_')}__{hash:10}.pdf "Your title")
```
