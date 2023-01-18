use super::*;

#[test]
fn cmark_to_tex_basic() {
    assert_eq!(
        cmark_to_tex("Hello World", ".").unwrap(),
        "\nHello World~\\\\\n",
    );
}

#[test]
fn cmark_to_tex_w_math_inline() {
    assert_eq!(
        cmark_to_tex(r##"A $\sum_1^12 x^2$ formula"##, ".").unwrap(),
        r##"
A $\sum_1^12 x^2$ formula~\\
"##,
    );
}

#[test]
fn cmark_to_tex_w_math_block() {
    assert_eq!(
        
        cmark_to_tex(
            r##"$$
\sum_1^12 x^2
$$
formula"##,
            "."
        )
        .unwrap(),
        r##"
$$
\sum_1^12 x^2
$$
formula~\\
"##,
    );
}

#[test]
fn cmark_to_tex_image() {
    assert_eq!(
        cmark_to_tex("![](image.png)",  "../..").unwrap(),
        "\n\\begin{figure}\n\\centering\n\\includegraphics[width=\\textwidth]{image.png}\n\\caption{}\n\\end{figure}\n~\\\\\n",
    );
}
