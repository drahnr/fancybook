use super::*;

pub fn format_figure<'a>(
    replacement: &Replacement<'a>,
    refer: &str,
    head_num: &str,
    figures_counter: usize,
    title: &str,
    renderer: SupportedRenderer,
) -> String {
    use SupportedRenderer::*;
    match renderer {
        Html | Markdown => {
            format!(
                r#"<figure id="{refer}" class="figure">
                    <object data="{file}" type="image/svg+xml"/></object>
                    <figcaption>Figure {head_num}{figures_counter} {title}</figcaption>
                </figure>"#,
                refer = refer,
                head_num = head_num,
                figures_counter = figures_counter,
                title = title,
                file = replacement.svg_asset_file.display()
            )
        }
        Latex | Tectonic => {
            format!(
                r#"$${}
$$"#,
                replacement.inner_str_or_intermediate()
            )
        }
    }
}

pub fn format_equation_block<'a>(
    replacement: &Replacement<'a>,
    refer: &str,
    head_num: &str,
    equations_counter: usize,
    renderer: SupportedRenderer,
) -> String {
    use SupportedRenderer::*;
    match renderer {
        Html | Markdown => {
            format!(
                r#"<div id="{refer}" class="equation">
                    <div class="equation_inner">
                        <object data="{file}" type="image/svg+xml"></object>
                    </div><span>({head_num}{equations_counter})</span>
                </div>"#,
                refer = refer,
                head_num = head_num,
                equations_counter = equations_counter,
                file = replacement.svg_asset_file.display()
            )
        }
        Latex | Tectonic => {
            format!(
                r#"$${}
$$"#,
                replacement.inner_str_or_intermediate()
            )
        }
    }
}

pub fn format_equation_inline<'a>(
    replacement: &Replacement<'a>,
    renderer: SupportedRenderer,
) -> String {
    use SupportedRenderer::*;
    match renderer {
        Html | Markdown => {
            format!(
                r#"<object class="equation_inline" data="{file}" type="image/svg+xml"></object>"#,
                file = replacement.svg_asset_file.display()
            )
        }
        Latex | Tectonic => {
            format!(r#"${}$"#, replacement.inner_str_or_intermediate())
        }
    }
}

pub fn format_bib_reference<'a>(refere: &str, title: &str, renderer: SupportedRenderer) -> String {
    use SupportedRenderer::*;
    match renderer {
        Html => {
            format!(r#"<a class="bib_ref" href='bibliography.html#{refere}'>{title}</a>"#)
        }
        Latex | Tectonic | Markdown => {
            format!("$ref:bib:{refere}$")
        }
    }
}

pub fn format_fig_reference<'a>(refere: &str, title: &str, renderer: SupportedRenderer) -> String {
    use SupportedRenderer::*;
    match renderer {
        Html => {
            format!(r#"<a class="fig_ref" href='#{refere}'>{title}</a>"#)
        }
        Latex | Tectonic | Markdown => {
            format!("$ref:fig:{refere}$")
        }
    }
}

pub fn format_equ_reference<'a>(refere: &str, title: &str, renderer: SupportedRenderer) -> String {
    use SupportedRenderer::*;
    match renderer {
        Html => {
            format!(r#"<a class="equ_ref" href='#{refere}'>Eq. ({title})</a>"#)
        }
        Latex | Tectonic | Markdown => {
            format!("$ref:equ:{refere}$")
        }
    }
}
