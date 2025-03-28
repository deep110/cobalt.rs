use pulldown_cmark as cmark;
use serde::Serialize;

use crate::error::Result;
use crate::syntax_highlight::decorate_markdown;
use super::toc::TOCGenerator;

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarkdownBuilder {
    pub theme: Option<liquid::model::KString>,
    #[serde(skip)]
    pub syntax: std::sync::Arc<crate::SyntaxHighlight>,
}

impl MarkdownBuilder {
    pub fn build(self) -> Markdown {
        Markdown {
            theme: self.theme,
            syntax: self.syntax,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Markdown {
    theme: Option<liquid::model::KString>,
    syntax: std::sync::Arc<crate::SyntaxHighlight>,
}

impl Markdown {
    pub fn parse(&self, content: &str) -> Result<String> {
        let mut buf = String::new();
        let options = cmark::Options::ENABLE_FOOTNOTES
            | cmark::Options::ENABLE_TABLES
            | cmark::Options::ENABLE_STRIKETHROUGH
            | cmark::Options::ENABLE_TASKLISTS
            | cmark::Options::ENABLE_HEADING_ATTRIBUTES
            | cmark::Options::ENABLE_GFM
            | cmark::Options::ENABLE_SUPERSCRIPT
            | cmark::Options::ENABLE_SUBSCRIPT
            | cmark::Options::ENABLE_DEFINITION_LIST
            | cmark::Options::ENABLE_MATH;
        let parser = cmark::Parser::new_ext(content, options);

        let code_block_parser =
            decorate_markdown(parser, self.syntax.clone(), self.theme.as_deref())?;
        let toc_parser = TOCGenerator::new(code_block_parser);

        cmark::html::push_html(&mut buf, toc_parser);
        Ok(buf)
    }
}
