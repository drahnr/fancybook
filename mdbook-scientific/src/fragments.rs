use fs_err as fs;
use itertools::Itertools;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{io::Write, str, usize};

use sha2::{Digest, Sha256};

use crate::errors::*;
use crate::types::*;

/// Convert input string to 24 character hash
pub fn hash(input: impl AsRef<str>) -> String {
    let mut sh = Sha256::new();
    sh.update(input.as_ref().as_bytes());
    let mut out = format!("{:x}", sh.finalize());
    out.truncate(24);
    out
}

/// Generate SVG file from latex file with given zoom
///
/// `base` is used as based and added with extensions for intermediate files
pub fn generate_svg_from_latex(base: &Path, zoom: f32) -> Result<PathBuf> {
    let dest_path = base.parent().expect("Parent path must exist. qed");
    let file: &Path = base.file_name().unwrap().as_ref();

    // use latex to generate a dvi
    let dvi_path = base.with_extension("dvi");
    if !dvi_path.exists() {
        let latex_path = find_binary("latex")?;

        let cmd = Command::new(latex_path)
            .current_dir(dest_path)
            //.arg("--jobname").arg(&dvi_path)
            .arg(&file.with_extension("tex"))
            .output()?;

        if !cmd.status.success() {
            let buf = String::from_utf8_lossy(&cmd.stdout);

            // latex prints error to the stdout, if this is empty, then something is fundamentally
            // wrong with the latex binary (for example shared library error). In this case just
            // exit the program
            if buf.is_empty() {
                let buf = String::from_utf8_lossy(&cmd.stderr);
                panic!("latex exited with `{}`", buf);
            }

            let err = buf
                .split('\n')
                .filter(|x| {
                    (x.starts_with("! ") || x.starts_with("l.")) && !x.contains("Emergency stop")
                })
                .fold(("", "", usize::MAX), |mut err, elm| {
                    if let Some(striped) = elm.strip_prefix("! ") {
                        err.0 = striped;
                    } else if let Some(striped) = elm.strip_prefix("l.") {
                        let mut elms = striped.splitn(2, ' ').map(|x| x.trim());
                        if let Some(Ok(val)) = elms.next().map(|x| x.parse::<usize>()) {
                            err.2 = val;
                        }
                        if let Some(val) = elms.next() {
                            err.1 = val;
                        }
                    }

                    err
                });

            return Err(ScientificError::InvalidMath(
                err.0.to_string(),
                err.1.to_string(),
                err.2,
            ));
        }
    }

    // convert the dvi to a svg file with the woff font format
    let svg_path = base.with_extension("svg");
    if !svg_path.exists() && dvi_path.exists() {
        let dvisvgm_path = find_binary("dvisvgm")?;

        let cmd = Command::new(dvisvgm_path)
            .current_dir(dest_path)
            .arg("-b")
            .arg("1")
            .arg("--font-format=woff")
            .arg(&format!("--zoom={}", zoom))
            .arg(&dvi_path)
            .output()?;

        let buf = String::from_utf8_lossy(&cmd.stderr);
        if !cmd.status.success() || buf.contains("error:") {
            return Err(ScientificError::InvalidDvisvgm(buf.to_string()));
        }
    }

    Ok(svg_path)
}

/// Generate latex file from gnuplot
///
/// This function generates a latex file with gnuplot `epslatex` backend and then source it into
/// the generate latex function
fn generate_latex_from_gnuplot<'a>(
    dest_dir: &Path,
    content: &Content<'a>,
    filename: &Path,
) -> Result<()> {
    let content = content.trimmed().as_str();
    let gnuplot_path = find_binary("gnuplot")?;

    let cmd = Command::new(gnuplot_path)
        .stdin(Stdio::piped())
        .current_dir(dest_dir)
        .arg("-p")
        .spawn()?;

    let mut stdin = cmd.stdin.expect("Stdin of gnuplot spawn must exist. qed");

    stdin.write_all(format!("set output '{}'\n", filename.display()).as_bytes())?;
    stdin.write_all("set terminal epslatex color standalone\n".as_bytes())?;
    stdin.write_all(content.as_bytes())?;

    Ok(())
}

/// Parse an equation with the given zoom
pub fn generate_replacement_file_from_template<'a>(
    fragment_path: &Path,
    asset_path: &Path,
    content: &Content<'a>,
    zoom: f32,
    chapter_number: &str,
    _chapter_name: &str,
) -> Result<Replacement<'a>> {
    let content_hash = hash(content.as_str());
    let name = format!(
        "scientific_{}__{}",
        &chapter_number,
        &content_hash.as_str()[..10]
    )
    .replace('.', "_");
    let name = PathBuf::from(name).with_extension("svg");

    let fragment_file = fragment_path.join(&name);
    let svg_asset_file = asset_path.join(&name);

    if content.byte_range.len() == 2 {
        log::error!(
            "Found $$ but got interpreted as two consecutive $ signs! {:?}",
            &content.byte_range
        )
    }
    log::info!(
        r#"Found equation from {}:{}..{}:{}:
    {}"#,
        content.start.lineno,
        content.start.column,
        content.end.lineno,
        content.end.column,
        content.s
    );
    log::debug!(
        "Using temporary helper file {}",
        fragment_file.with_extension("tex").display()
    );

    let tex = content.trimmed().as_str();

    // create a new tex file containing the equation
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(fragment_file.with_extension("tex"))?;

    let fragment = include_str!("fragment.tex")
        .split("$$")
        .enumerate()
        .map(|(idx, s)| match idx {
            0 | 2 => s,
            1 => tex,
            _ => unreachable!("fragment.tex must have exactly 2 instances of `$$`"),
        })
        .join("$$");

    let bytes = fragment.as_bytes();
    file.write_all(fragment.as_bytes())?;

    let svg_fragment_file = generate_svg_from_latex(&fragment_file, zoom)?;
    log::debug!(
        "Wrote fragment with {} bytes to {}",
        bytes.len(),
        fragment_file.with_extension("tex").display()
    );

    let svg = fs::read_to_string(&svg_fragment_file)?;
    log::debug!(
        "Generated svg with {} bytes to {}",
        svg.len(),
        svg_fragment_file.display()
    );

    Ok(Replacement {
        content: content.clone(),
        intermediate: None,
        svg_fragment_file,
        svg_asset_file,
    })
}

/// Parse a latex content and convert it to a SVG file
pub fn parse_latex<'a>(
    fragment_path: &Path,
    asset_path: &Path,
    content: &Content<'a>,
) -> Result<Replacement<'a>> {
    let tex = content.trimmed().as_str();
    let name = hash(tex);
    let name = PathBuf::from(name).with_extension("svg");
    let svg_fragment_file = fragment_path.join(&name);
    let svg_asset_file = asset_path.join(&name);

    // create a new tex file containing the equation
    if !svg_fragment_file.with_extension("tex").exists() {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(svg_fragment_file.with_extension("tex"))?;

        file.write_all(tex.as_bytes())?;
    }

    let svg_fragment_file = generate_svg_from_latex(&svg_fragment_file, 1.0)?;

    Ok(Replacement {
        content: content.clone(),
        intermediate: None,
        svg_fragment_file,
        svg_asset_file,
    })
}

/// Parse a gnuplot file and generate a SVG file
pub fn parse_gnuplot<'a>(
    fragment_path: &Path,
    asset_path: &Path,
    content: &Content<'a>,
) -> Result<Replacement<'a>> {
    let name = hash(content.as_str());
    let name = PathBuf::from(name).with_extension("svg");

    let path = fragment_path.join(&name);
    if !path.with_extension("tex").exists() {
        //let name_plot = format!("{}_plot", name);
        generate_latex_from_gnuplot(fragment_path, content, path.with_extension("tex").as_path())?;
    }

    let svg_asset_path = asset_path.join(&name);

    let svg_fragment_path = generate_svg_from_latex(&path, 1.0)?;

    let intermediate = fs::read_to_string(path.with_extension("tex"))?;

    Ok(Replacement {
        content: content.to_owned(),
        intermediate: Some(intermediate),
        svg_fragment_file: svg_fragment_path,
        svg_asset_file: svg_asset_path,
    })
}

/// Parse gnuplot without using the latex backend
pub fn parse_gnuplot_only<'a>(
    fragment_path: &Path,
    asset_path: &Path,
    content: &Content<'a>,
) -> Result<Replacement<'a>> {
    let gnuplot_input = content.trimmed().as_str();
    let name = hash(gnuplot_input);
    let name = PathBuf::from(name).with_extension("svg");
    let svg_fragment_path = fragment_path.join(&name);
    let svg_asset_path = asset_path.join(&name);

    if !svg_fragment_path.with_extension("svg").exists() {
        let gnuplot_path = find_binary("gnuplot")?;
        let cmd = Command::new(gnuplot_path)
            .stdin(Stdio::piped())
            .current_dir(fragment_path)
            .arg("-p")
            .spawn()?;

        let mut stdin = cmd.stdin.unwrap();
        stdin.write_all(format!("set output '{}'\n", name.display()).as_bytes())?;
        stdin.write_all("set terminal svg\n".as_bytes())?;
        stdin.write_all("set encoding utf8\n".as_bytes())?;
        stdin.write_all(gnuplot_input.as_bytes())?;
    }

    Ok(Replacement {
        content: content.clone(),
        intermediate: None,
        svg_fragment_file: svg_fragment_path,
        svg_asset_file: svg_asset_path,
    })
}

/// Generate html from BibTeX file using `bib2xhtml`
pub fn bib_to_html(source: &str, bib2xhtml: &str) -> Result<String> {
    let source = fs::canonicalize(source)?;
    let bib2xhtml = Path::new(bib2xhtml);

    //./bib2xhtml.pl -s alpha -u -U ~/Documents/Bachelor_thesis/literature.bib
    let cmd = Command::new(bib2xhtml.join("./bib2xhtml.pl"))
        .current_dir(bib2xhtml)
        .args(["-s", "alpha", "-u", "-U"])
        .arg(source)
        .output()
        .expect("Could not spawn bib2xhtml");

    let buf = String::from_utf8_lossy(&cmd.stdout);

    let err_str = String::from_utf8_lossy(&cmd.stderr);
    if err_str.contains("error messages)") {
        Err(ScientificError::InvalidBibliography(err_str.to_string()))
    } else {
        let buf = buf
            .split('\n')
            .skip_while(|x| *x != "<dl class=\"bib2xhtml\">")
            .take_while(|x| *x != "</dl>")
            .map(|x| x.replace("<a name=\"", "<a id=\""))
            .collect();

        Ok(buf)
    }
}
