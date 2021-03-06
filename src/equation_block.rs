use std::io::Write;

use katex;
use liquid_core::error::ResultLiquidReplaceExt;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::{BlockReflection, ParseBlock, TagBlock, TagTokenIter};

#[derive(Clone, Debug)]
struct Equation {
    content: String,
}

impl Renderable for Equation {
    fn render_to(&self, writer: &mut dyn Write, _runtime: &mut Runtime<'_>) -> Result<()> {
        write!(writer, "{}", self.content).replace("Equation Failed to render")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct EquationBlock {
    katex_opts: katex::Opts,
}

impl EquationBlock {
    pub fn new() -> Self {
        let katex_opts = katex::Opts::builder()
            .display_mode(true)
            .output_type(katex::OutputType::Html)
            .throw_on_error(false)
            .build()
            .unwrap();

        EquationBlock { katex_opts }
    }
}

impl BlockReflection for EquationBlock {
    fn start_tag(&self) -> &str {
        "equation"
    }

    fn end_tag(&self) -> &str {
        "endequation"
    }

    fn description(&self) -> &str {
        ""
    }
}

impl ParseBlock for EquationBlock {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        mut tokens: TagBlock<'_, '_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        // no arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;

        let raw_content = tokens.escape_liquid(false)?.to_string();
        let content = katex::render_with_opts(&raw_content, &self.katex_opts).unwrap();

        tokens.assert_empty();
        Ok(Box::new(Equation { content }))
    }

    fn reflection(&self) -> &dyn BlockReflection {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid_core::parser;
    use liquid_core::runtime;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .blocks
            .register("equation".to_string(), Box::new(EquationBlock::new()));
        options
    }

    fn unit_parse(text: &str) -> String {
        let options = options();
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let mut runtime = Runtime::new();

        template.render(&mut runtime).unwrap()
    }

    #[test]
    fn test_equation() {
        let output = unit_parse("{% equation %} This is a test {% endequation %}");
        assert_eq!(output, " This is a test ");
    }
}
