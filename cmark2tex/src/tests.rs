use super::*;

#[test]
fn cmark_to_tex_basic() {
    assert_eq!(
        "\nHello World~\\\\\n",
        cmark_to_tex("Hello World", ".").unwrap()
    );
}

#[test]
fn cmark_to_tex_w_math_inline() {
    assert_eq!(
        r##"
A $\sum_1^12 x^2$ formula~\\
"##,
        cmark_to_tex(r##"A $\sum_1^12 x^2$ formula"##, ".").unwrap()
    );
}

#[test]
fn cmark_to_tex_w_math_block() {
    assert_eq!(
        r##"
$$
\sum_1^12 x^2
$$
formula~\\
"##,
        cmark_to_tex(
            r##"$$
\sum_1^12 x^2
$$
formula"##,
            "."
        )
        .unwrap()
    );
}

#[test]
fn cmark_to_tex_image() {
    assert_eq!(
        "\n\\begin{figure}\n\\centering\n\\includegraphics[width=\\textwidth]{image.png}\n\\caption{}\n\\end{figure}\n~\\\\\n",
        cmark_to_tex("![](image.png)",  "../..").unwrap()
    );
}
