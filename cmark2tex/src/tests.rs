use super::*;

#[test]
fn cmark_to_tex_basic() {
    assert_eq!(
        cmark_to_tex("Hello World", ".", &[PathBuf::from(".")]).unwrap(),
        "\nHello World~\\\\\n",
    );
}

#[test]
fn cmark_to_tex_w_math_inline() {
    assert_eq!(
        cmark_to_tex(r##"A $\sum_1^12 x^2$ formula"##, ".", &[PathBuf::from(".")]).unwrap(),
        r##"
A $\sum_1^12 x^2$ formula~\\
"##,
    );
}

#[test]
fn cmark_to_tex_w_math_block_anon() {
    assert_eq!(
        cmark_to_tex(
            r##"$$
\sum_1^12 x^2
$$
formula"##,
            ".",
            &[PathBuf::from(".")]
        )
        .unwrap(),
        r##"

\begin{align}%
\sum_1^12 x^2
\end{align}
formula~\\
"##,
    );
}

#[test]
fn cmark_to_tex_w_math_block_reference() {
    assert_eq!(
        cmark_to_tex(
            r##"$$equ,xyz,Some title
\sum_1^12 x^2
$$
formula $ref:equ:xyz$"##,
            ".",
            &[PathBuf::from(".")]
        )
        .unwrap(),
        r##"

\begin{align}%
\label{xyz}%
\sum_1^12 x^2
\end{align}
formula \eqref{xyz}~\\
"##,
    );
}

#[test]
fn cmark_to_tex_image() {
    assert_eq!(
        cmark_to_tex(r##"![FIXME](a.png "Hello World!")"##, ".", &[PathBuf::from("../mdbook-tectonic/examples/sample-book/src/chapter-1")]).unwrap(),
        r###"
\begin{figure}%
\centering%
\includegraphics[width=\textwidth]{../mdbook-tectonic/examples/sample-book/src/chapter-1/a.png}%
\caption{Hello World!}%
\end{figure}%
FIXME~\\
"###,
    );
}

#[test]
fn cmark_to_tex_large() {
    const MD: &str = r####"```markdown
$codeagain!$
$$
not_math
$$
```

$$equ,oink,foo
y = \sum somath
$$

Now ref that one block equ $ref:equ:oink$.
"####;

    const TEX: &str = r####"\begin{lstlisting}[language=markdown]
$codeagain!$
$$
not_math
$$

\end{lstlisting}


\begin{align}%
\label{oink}%
y = \sum somath
\end{align}~\\

Now ref that one block equ \eqref{oink}.~\\
"####;
    assert_eq!(cmark_to_tex(MD, "/tmp", &[PathBuf::from("foo")]).unwrap(), TEX)
}
