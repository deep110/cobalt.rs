use pulldown_cmark as cmark;

/// Iterator adapter that adds IDs to headings that don't have them
pub(super) struct TOCGenerator<'a, I>
where
    I: Iterator<Item = cmark::Event<'a>>,
{
    iter: I,
    id: Option<String>,
    heading_text: Option<String>,
    in_heading: bool,
}

impl<'a, I: Iterator<Item = cmark::Event<'a>>> TOCGenerator<'a, I> {
    pub(super) fn new(iter: I) -> Self {
        Self {
            iter,
            id: None,
            heading_text: None,
            in_heading: false,
        }
    }

    fn generate_id(text: &str) -> String {
        // Convert to lowercase
        let mut id = text.to_lowercase();

        // Replace non-alphanumeric characters with hyphens
        id = id
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();

        // Remove consecutive hyphens
        while id.contains("--") {
            id = id.replace("--", "-");
        }

        // Trim hyphens from start and end
        id.trim_matches('-').to_string()
    }
}

impl<'a, I: Iterator<Item = cmark::Event<'a>>> Iterator for TOCGenerator<'a, I> {
    type Item = cmark::Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(cmark::Event::Start(cmark::Tag::Heading { id, .. })) => {
                self.in_heading = true;
                if let Some(id) = id {
                    self.id = Some(id.to_string());
                }

                Some(cmark::Event::Text(cmark::CowStr::Borrowed("")))
            }
            Some(cmark::Event::Text(text)) => {
                if self.in_heading {
                    self.heading_text = Some(text.to_string());
                    Some(cmark::Event::Text(cmark::CowStr::Borrowed("")))
                } else {
                    Some(cmark::Event::Text(text))
                }
            }
            Some(cmark::Event::End(cmark::TagEnd::Heading(level))) => {
                self.in_heading = false;
                if self.id.is_none() {
                    let generated_id = Self::generate_id(&self.heading_text.as_ref().unwrap());
                    self.id = Some(generated_id);
                }

                let tag_name = match level {
                    cmark::HeadingLevel::H1 => "h1",
                    cmark::HeadingLevel::H2 => "h2",
                    cmark::HeadingLevel::H3 => "h3",
                    cmark::HeadingLevel::H4 => "h4",
                    cmark::HeadingLevel::H5 => "h5",
                    cmark::HeadingLevel::H6 => "h6",
                };

                let html = format!(
                    "<{tag} id=\"{id}\">{text}<a hidden=\"\" class=\"anchor\" aria-hidden=\"true\" href=\"#{id}\">#</a></{tag}>",
                    tag = tag_name,
                    id = self.id.as_ref().unwrap(),
                    text = self.heading_text.as_ref().unwrap()
                );

                // reset the id and heading text
                self.id = None;
                self.heading_text = None;

                Some(cmark::Event::Html(cmark::CowStr::Boxed(
                    html.into_boxed_str(),
                )))
            }
            item => item,
        }
    }
}
