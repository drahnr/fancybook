use mdbook_boilerplate::*;

fn main() -> Result<()> {
    let args = Args::try_parse()?;
    launch(mdbook_fishextract::Fishextract::new(), args, "🧜")
}
