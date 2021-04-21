use palantir_proto::palantir::shared::tag::Tag as ProtoTag;

#[derive(Clone)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl From<&ProtoTag> for Tag {
    fn from(t: &ProtoTag) -> Self {
        Self {
            key: t.key.clone(),
            value: t.value.clone(),
        }
    }
}
