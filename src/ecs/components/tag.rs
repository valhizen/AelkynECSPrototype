pub struct Tag {
    pub tag: String,
}

impl Tag {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
        }
    }
}
