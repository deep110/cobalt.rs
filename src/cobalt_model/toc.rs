use liquid::model::{Object, Value};
use pulldown_cmark as cmark;
use std::collections::HashMap;

/// Represents a heading in the document's table of contents
#[derive(Debug, Clone)]
struct TocEntry {
    /// The heading text
    pub title: String,

    /// The ID used for linking to this heading
    pub id: String,
}

impl TocEntry {
    fn new(title: String, id: String) -> Self {
        Self { title, id }
    }

    /// Convert a TocEntry to a Liquid Value for use in templates
    fn to_liquid(&self) -> Value {
        let mut obj = Object::new();
        obj.insert("title".into(), Value::scalar(self.title.clone()));
        obj.insert("permalink".into(), Value::scalar(format!("#{}", self.id)));

        Value::Object(obj)
    }
}

/// Iterator adapter that adds IDs to headings that don't have them
pub(super) struct TOCGenerator<'a, I>
where
    I: Iterator<Item = cmark::Event<'a>>,
{
    iter: I,
    id: Option<String>,
    heading_text: Option<String>,
    in_heading: bool,
    toc_entries: HashMap<String, Vec<TocEntry>>,
}

impl<'a, I: Iterator<Item = cmark::Event<'a>>> TOCGenerator<'a, I> {
    pub(super) fn new(iter: I) -> Self {
        Self {
            iter,
            id: None,
            heading_text: None,
            in_heading: false,
            toc_entries: HashMap::new(),
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

    /// Add a heading entry to the TOC structure
    fn add_toc_entry(&mut self, title: String, id: String, level: String) {
        let entry = TocEntry::new(title, id);
        self.toc_entries
            .entry(level)
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Get the table of contents as a Liquid Value
    pub(super) fn get_toc(&self) -> Option<Value> {
        if self.toc_entries.is_empty() {
            return None;
        }

        let mut toc_obj = Object::new();

        // Convert each HashMap entry directly to a key-value pair in the Object
        for (level, entries) in &self.toc_entries {
            let entries_array: Vec<Value> = entries.iter().map(|entry| entry.to_liquid()).collect();

            toc_obj.insert(level.clone().into(), Value::Array(entries_array.into()));
        }
        Some(Value::Object(toc_obj))
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

                // Add entry to TOC
                self.add_toc_entry(
                    self.heading_text.clone().unwrap(),
                    self.id.clone().unwrap(),
                    tag_name.to_string(),
                );

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
