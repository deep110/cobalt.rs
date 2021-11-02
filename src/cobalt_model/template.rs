use std::fmt;
use std::path;

use super::files;
use crate::error::*;
use crate::syntax_highlight;
use crate::custom_blocks;
use liquid;

type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

fn load_partials_from_path(root: path::PathBuf) -> Result<Partials> {
    let mut source = Partials::empty();

    debug!("Loading snippets from {:?}", root);
    let template_files = files::FilesBuilder::new(root)?
        .ignore_hidden(false)?
        .build()?;
    for file_path in template_files.files() {
        let rel_path = file_path
            .strip_prefix(template_files.root())
            .expect("file was found under the root")
            .to_str()
            .expect("only UTF-8 characters supported in paths")
            .to_owned();
        trace!("Loading snippet {:?}", rel_path);
        let content = files::read_file(file_path)?;
        source.add(rel_path, content);
    }
    Ok(source)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LiquidBuilder {
    pub includes_path: path::PathBuf,
    pub theme: kstring::KString,
}

impl LiquidBuilder {
    pub fn build(self) -> Result<Liquid> {
        let highlight = Self::highlight(self.theme)?;
        let parser = liquid::ParserBuilder::with_stdlib()
            .filter(liquid_lib::extra::DateInTz)
            .filter(liquid_lib::shopify::Pluralize)
            // Intentionally staying with `stdlib::IncludeTag` rather than `jekyll::IncludeTag`
            .filter(liquid_lib::jekyll::Slugify)
            .filter(liquid_lib::jekyll::Pop)
            .filter(liquid_lib::jekyll::Push)
            .filter(liquid_lib::jekyll::Shift)
            .filter(liquid_lib::jekyll::Unshift)
            .filter(liquid_lib::jekyll::ArrayToSentenceString)
            .partials(load_partials_from_path(self.includes_path)?)
            .block(highlight)
            .block(custom_blocks::EquationBlock::new())
            .build()?;
        Ok(Liquid { parser })
    }

    fn highlight(theme: kstring::KString) -> Result<Box<dyn liquid_core::ParseBlock>> {
        let result: Result<()> = match syntax_highlight::has_syntax_theme(&theme) {
            Ok(true) => Ok(()),
            Ok(false) => Err(failure::format_err!(
                "Syntax theme '{}' is unsupported",
                theme
            )),
            Err(err) => {
                warn!("Syntax theme named '{}' ignored. Reason: {}", theme, err);
                Ok(())
            }
        };
        result?;
        let block = syntax_highlight::CodeBlockParser::new(theme);
        Ok(Box::new(block))
    }
}

pub struct Liquid {
    parser: liquid::Parser,
}

impl Liquid {
    pub fn parse(&self, template: &str) -> Result<liquid::Template> {
        let template = self.parser.parse(template)?;
        Ok(template)
    }
}

impl fmt::Debug for Liquid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Liquid{{}}")
    }
}
