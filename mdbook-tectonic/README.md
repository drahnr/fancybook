# mdbook-tectonic

[![crates badge][crates-badge]][crates.io]
[![docs badge][docs-badge]][docs]
[![source badge][source-badge]][source]

[crates.io]: https://crates.io/crates/mdbook-tectonic
[crates-badge]: https://img.shields.io/badge/crates.io-v0.1.24-orange.svg?longCache=true

[docs]: https://docs.rs/crate/mdbook-tectonic
[docs-badge]: https://docs.rs/mdbook-tectonic/badge.svg

[source]: https://github.com/drahnr/mdbook-tectonic
[source-badge]: https://img.shields.io/badge/source-github-blue

<!-- toc -->

- [Status of Rust Bookshelf](#status-of-rust-bookshelf)
- [Installation](#installation)
- [Uninstallation](#uninstallation)
- [Primary Dependencies](#primary-dependencies)
- [How's it Work?](#hows-it-work)
- [Contributing](#contributing)
  - [I found a problem. Should I create an issue with `mdbook-tectonic` or `md2tex`?](#i-found-a-problem-should-i-create-an-issue-with-mdbook-tectonic-or-md2tex)
- [Are We Stable Yet?](#are-we-stable-yet)
  - [Manual Approach](#manual-approach)
  - [Finally](#finally)

<!-- tocstop -->

An [mdbook](https://github.com/rust-lang-nursery/mdBook) backend for generating LaTeX and PDF documents. Utilizes [`md2tex`](https://github.com/lbeckman314/md2tex) for the markdown to LaTeX transformation, but with the goal of allowing alternative markdown to LaTeX converters. If you have developed your own markdown to LaTeX converter, I'd love to talk with you or share ideas! I'm at [liam@liambeckman.com](mailto:liam@liambeckman).


## Status of Rust Bookshelf

- ✅ compiles successfully
- 🍊 compiles but with warnings/errors
- ❌ compilation fails/not yet attempted

| Compiles? | Generated PDF                          | Generated LaTeX                 | Source                     | Online Version            |
| --------- | ---------------------------------      | -----------------------         | ----------------------     | ---------------------     |
| ❌        | [~~Cargo Book~~][cargo-pdf]            | [~~LaTeX~~][cargo-latex]        | [Source][cargo-src]        | [HTML][cargo-html]        |
| ❌        | [~~Edition Guide~~][edition-pdf]       | [~~LaTeX~~][edition-latex]      | [Source][edition-src]      | [HTML][edition-html]      |
| ❌        | [~~Embedded Rust Book~~][embedded-pdf] | [~~LaTeX~~][embedded-latex]     | [Source][embedded-src]     | [HTML][embedded-html]     |
| 🍊        | [Mdbook User Guide][mdbook-pdf]        | [LaTeX][mdbook-tectonic]           | [Source][mdbook-src]       | [HTML][mdbook-html]       |
| ❌        | [~~Rust Reference~~][reference-pdf]    | [~~LaTeX~~][reference-latex]    | [Source][reference-src]    | [HTML][reference-html]    |
| ❌        | [~~Rust By Example~~][example-pdf]     | [~~LaTeX~~][example-latex]      | [Source][example-src]      | [HTML][example-html]      |
| 🍊        | [Rust Programming Language][rust-pdf]  | [LaTeX][rust-latex]             | [Source][rust-src]         | [HTML][rust-html]         |
| ❌        | [~~Rustc Book~~][rustc-pdf]            | [~~LaTeX~~][rustc-latex]        | [Source][rustc-src]        | [HTML][rustc-html]        |
| ❌        | [~~Rustdoc Book~~][rustdoc-pdf]        | [~~LaTeX~~][rustdoc-latex]      | [Source][rustdoc-src]      | [HTML][rustdoc-html]      |
| ❌        | [~~Rustonomicon~~][rustonomicon-pdf]   | [~~LaTeX~~][rustonomicon-latex] | [Source][rustonomicon-src] | [HTML][rustonomicon-html] |
| ❌        | [~~Unstable Book~~][unstable-pdf]      | [~~LaTeX~~][unstable-latex]     | [Source][unstable-src]     | [HTML][unstable-html]     |

[rust-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases/download/v0.1.24/The.Rust.Programming.Language.pdf
[rust-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases/download/v0.1.24/The.Rust.Programming.Language.tex
[rust-src]: https://github.com/rust-lang/book
[rust-html]: https://doc.rust-lang.org/book/

[mdbook-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases/download/v0.1.24/mdBook.Documentation.pdf
[mdbook-tectonic]: https://github.com/lbeckman314/mdbook-tectonic/releases/download/v0.1.24/mdBook.Documentation.tex
[mdbook-src]: https://github.com/rust-lang-nursery/mdBook/tree/master/book-example
[mdbook-html]: https://rust-lang-nursery.github.io/mdBook/

[example-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[example-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[example-src]: https://github.com/rust-lang/rust-by-example
[example-html]: https://doc.rust-lang.org/stable/rust-by-example/

[edition-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[edition-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[edition-src]: https://github.com/rust-lang-nursery/edition-guide
[edition-html]: https://doc.rust-lang.org/edition-guide/index.html

[rustc-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[rustc-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[rustc-src]: https://github.com/rust-lang/rustc-guide
[rustc-html]: https://doc.rust-lang.org/rustc/index.html

[cargo-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[cargo-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[cargo-src]: https://github.com/rust-lang/cargo/tree/master/src/doc
[cargo-html]: https://doc.rust-lang.org/cargo/index.html

[rustdoc-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[rustdoc-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[rustdoc-src]: https://github.com/rust-lang/rust/tree/master/src/doc/rustdoc
[rustdoc-html]: https://doc.rust-lang.org/rustdoc/index.html

[reference-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[reference-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[reference-src]: https://github.com/rust-lang-nursery/reference
[reference-html]: https://doc.rust-lang.org/reference/index.html

[rustonomicon-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[rustonomicon-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[rustonomicon-src]: https://github.com/rust-lang-nursery/nomicon
[rustonomicon-html]: https://doc.rust-lang.org/nomicon/index.html

[unstable-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[unstable-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[unstable-src]: https://github.com/rust-lang/rust/tree/master/src/doc/unstable-book
[unstable-html]: https://doc.rust-lang.org/unstable-book/index.html

[embedded-pdf]: https://github.com/lbeckman314/mdbook-tectonic/releases
[embedded-latex]: https://github.com/lbeckman314/mdbook-tectonic/releases
[embedded-src]: https://github.com/rust-embedded/book
[embedded-html]: https://rust-embedded.github.io/book/

## Installation

### Requirements

- [Rust](https://www.rust-lang.org/)
- [mdbook](https://github.com/rust-lang-nursery/mdBook)
- [Tectonic + support libraries](https://tectonic-typesetting.github.io/en-US/install.html#the-cargo-install-method)

### Cargo install + Configuration

```sh
cargo install mdbook-tectonic
```

Add the following `toml` configuration to `book.toml`.

```toml
[output.latex]
latex    = true  # default = true
pdf      = true  # default = true
markdown = true  # default = true
```

The next `mdbook build` command will produce LaTeX and PDF files (and the markdown file of your mdbook) in the `book/latex/` directory.

## Uninstallation

To uninstall `mdbook-tectonic`, enter the following in a shell:

```sh
cargo uninstall mdbook-tectonic
```

Then delete the `[output.latex]` configuration in `book.toml`:

```diff
- [output.latex]
- latex    = true
- pdf      = true
- markdown = true
```

## Primary Dependencies

`mdbook-tectonic` is built upon some really wonderful projects, including:

- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark): Parses the markdown source AST.
- [Tectonic](https://tectonic-typesetting.github.io/en-US/): Creates the final PDF file from the transformed LaTeX code.
- [md2tex](https://github.com/lbeckman314/md2tex): Transforms the markdown source to LaTeX. This is a fork of [md2pdf](https://gitea.tforgione.fr/tforgione/md2pdf/), a great utility for converting markdown code to LaTeX and PDF's.  I hope to eventually propose some of the updates back upstream. `md2tex` and `mdbook-tectonic` are developed in tandem, but are meant to be independent programs. Therefore, if one wishes to use an alternative markdown-to-tex conveter, they should be able to plug it in to `mdbook-tectonic` with ease.

## How's it Work?

Broadly speaking, there are three steps, or "transformations", from `mdbook` source to PDF output:

1) **mdbook source to JSON-organized markdown** (`mdbook-tectonic`): retreives the JSON formatted data from `mdbook`. Calls `md2tex` and `tectonic` for LaTeX and PDF generation, respectively.
2) **markdown to LaTeX** (`md2tex`): converts markdown input to LaTeX output.
3) **LaTeX to PDF** (`tectonic`): creates PDF document from LaTeX input.

## Contributing

Pull requests, forks, and plain old copy-pasting are actively encouraged! Also, I am relatively new to Rust (and programming in general) so recommendations or advice in general is always appreciated.

### I found a problem. Should I create an issue with `mdbook-tectonic` or `md2tex`?

Either one. `mdbook-tectonic` can be thought of as a frontend for the LaTeX generation done by `md2tex`. So if there is a LaTeX error, you may favor creating an issue with `md2tex`. Otherwise, creating an issue with `mdbook-tectonic` is a good bet. But any issue is a good issue, so don't worry if it's in the "right" repository or not, I should be able to see it regardless!

## Are We Stable Yet?

Below is a list of features I am currently working on (loosely in a "top-down" direction).

- [x] Add support for equation delimiters "\( x^2 \)" "\[ x^2 \]".
- [x] Allow SVG images (convert to PNG for LaTeX).
    - [x] Configure [resvg](https://github.com/RazrFalcon/resvg) library to convert SVG's to PNG.
    - [x] Save SVG's in `book/latex` directory to keep `src` clean.
- [x] Add CI/CD pipeline ([travis](https://travis-ci.org/))
- [x] Move all LaTeX data to single template file (src/template.tex).
- [ ] Add support for raw HTML tables.
- [ ] Add syntax highlighting via [syntect](https://github.com/trishume/syntect) à la [lumpy-leandoc](https://github.com/ratmice/lumpy-leandoc).
- [ ] Add parallel transformations via [Rayon](https://github.com/rayon-rs/rayon) à la [lumpy-leandoc](https://github.com/ratmice/lumpy-leandoc).
- [ ] Use [lumpy-leandoc](https://github.com/ratmice/lumpy-leandoc)'s method for handling events (replace `current` event with `fold`).
- [ ] Compile *The Rust Book* and *mdbook* documentation without any errors or warnings (e.g. missing Unicode characters). See [Status of Rust Bookshelf](#status-of-rust-bookshelf) for up to date progress.
- [ ] Put "tectonic" dependency in "pdf" feature configuration.
- [ ] Add "table of contents" mdbook toml option.
- [x] Add "markdown" mdbook toml option.
- [ ] Add "number of words" mdbook toml option.
- [ ] Add "examples" directory.
- [ ] Create documentation and move relevent docs to md2tex.
- [ ] Add option for custom LaTeX headers.
- [ ] Add option for alternative markdown-to-latex converter plugin.
- [ ] Add test suites.
- [ ] Cross compile binaries ([trust](https://github.com/japaric/trust))
- [ ] Add option to generate PDF with [mdproof](https://github.com/Geemili/mdproof) to skip LaTeX dependencies.
- [ ] Complete acordance with the [CommonMark spec](https://spec.commonmark.org/).

### Manual Approach

If, however, you don't mind getting your hands dirty with LaTeX, here is my process for when the build step fails:

1) Change the latex configuration in `book.toml` to only output LaTeX and markdown files:

```toml
[output.latex]
latex = true
pdf = false
markdown = true
```

2) First see where `tectonic` is running into errors by manually running it and looking for `! LaTeX Error`:

```sh
tectonic book/latex/MY_BOOK.tex

```

Aha! `! LaTeX Error: Missing \begin{document}.`

In this example, `mdbook-tectonic` failed to output the very important `\begin{document}` line.

3) Fix the grievous goof-up in your favorite editor and rerun `tectonic` (repeat this step until tectonic successfully compiles the PDF):

```sh
ed book/latex/MY_BOOK.tex

tectonic book/latex/MY_BOOK.tex
```

Is it an elegant approach? No. Does it work? Sometimes. Is it a pain? Always.

### Finally

If you're feeling especially adventurous, create an issue or get in touch with me ([liam@liambeckman.com](mailto:liam@liambeckman)) to help prevent the same errors in the future. I'm more than happy to work with you to get your document compiled!

: ^ )
