use crate::metrics::histogram::metric::Histogram;
use crate::metrics::tag::Tag;

pub struct HistogramBuilder {
    name: String,
    tags: Vec<Tag>,
}

impl HistogramBuilder {
    pub fn named(name: &str) -> Self {
        return Self {
            name: String::from(name),
            tags: Vec::new(),
        };
    }

    pub fn tag<'a>(&'a mut self, key: &'a str, value: &'a str) -> &'a mut Self {
        self.tags.push(Tag {
            key: String::from(key),
            value: String::from(value),
        });
        self
    }

    pub fn finish(&self) -> Histogram {
        Histogram::new(self.name.clone(), self.tags.clone())
    }
}
