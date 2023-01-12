use mdbook_boilerplate::*;

fn main() -> Result<()> {
    let args = Args::try_parse()?;
    launch(mdbook_scientific::Scientific::new(), args)
}
