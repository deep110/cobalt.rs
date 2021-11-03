use std::io::Write;
use std::path;
use std::fs;

use liquid_core::error::ResultLiquidReplaceExt;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::{BlockReflection, ParseBlock, TagBlock, TagTokenIter};
use svgbob::Render;

#[derive(Clone, Debug)]
struct AsciiArt {
    content: String,
}

impl Renderable for AsciiArt {
    fn render_to(&self, writer: &mut dyn Write, _runtime: &dyn Runtime) -> Result<()> {
        write!(writer, "{}", self.content).replace("AsciiArt Failed to render")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct AsciiArtBlock {
    base_path: String,
    svgbob_opts: svgbob::Settings,
}

impl AsciiArtBlock {
    pub fn new(destination: String) -> Self {
        AsciiArtBlock {
            base_path: destination,
            svgbob_opts: svgbob::Settings::default(),
        }
    }

    fn parse_input_path(&self, input: &str) -> path::PathBuf {
        // remove "" around the image path
        // remove starting `/` since it is a relative path
        let input_path = if &input[1..2] == "/" {
            &input[2..input.len()-1]
        } else {
            &input[1..input.len()-1]
        };
        return path::Path::new(&self.base_path).join(input_path);
    }
}

impl BlockReflection for AsciiArtBlock {
    fn start_tag(&self) -> &str {
        "ascii_art"
    }

    fn end_tag(&self) -> &str {
        "endascii_art"
    }

    fn description(&self) -> &str {
        ""
    }
}

impl ParseBlock for AsciiArtBlock {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        mut tokens: TagBlock<'_, '_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let target = arguments
            .expect_next("Value expected.")?
            .expect_value()
            .into_result()?
            .to_string();

        // no more arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;

        let raw_content = tokens.escape_liquid(false)?.to_string();
        let cb = svgbob::CellBuffer::from(&*raw_content);
        let (node, _width, _height): (svgbob::Node<()>, f32, f32) =
            cb.get_node_with_size(&self.svgbob_opts);
        let svg = node.render_to_string();

        // save svg to a file
        let svg_path = self.parse_input_path(&target);
        match &svg_path.parent() {
            Some(dirs_path) => fs::create_dir_all(dirs_path).expect("unable to create svg image file"),
            None => (),
        };
        fs::write(&svg_path, svg).expect("unable to create svg image file");

        let content = format!(
            "<div class='ascii_art'><img src={}/></div>",
            target
        );

        tokens.assert_empty();
        Ok(Box::new(AsciiArt { content }))
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
    use liquid_core::runtime::RuntimeBuilder;

    fn options() -> Language {
        let mut options = Language::default();
        let path = "/tmp".to_string();

        options
            .blocks
            .register("ascii_art".to_string(), Box::new(AsciiArtBlock::new(path)));
        options
    }

    fn unit_parse(text: &str) -> String {
        let options = options();
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let mut runtime = RuntimeBuilder::new().build();
        template.render(&mut runtime).unwrap()
    }

    #[test]
    fn test_ascii_art_block() {
        let output = unit_parse("{% ascii_art \"img.svg\" %}----->{% endascii_art %}");
        assert_eq!(output, "----->");
    }
}
