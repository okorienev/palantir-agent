use palantir_proto::palantir::apm::v1::action::ApmV1Action;
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use palantir_proto::palantir::shared::tag::Tag;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub trait Checksum {
    fn checksum(&self) -> u64;
}

impl Checksum for ProtoMessage {
    fn checksum(&self) -> u64 {
        match self {
            Self::ApmV1Action(action) => action.checksum(),
        }
    }
}

impl Checksum for Vec<Tag> {
    fn checksum(&self) -> u64 {
        let mut checksums: Vec<u64> = Vec::with_capacity(self.len());
        for tag in self {
            checksums.push(tag.checksum());
        }
        checksums.sort();

        let mut hasher = DefaultHasher::new();
        for checksum in &checksums {
            hasher.write_u64(*checksum);
        }

        hasher.finish()
    }
}

impl Checksum for Tag {
    fn checksum(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.key.as_bytes());
        hasher.write(self.value.as_bytes());

        hasher.finish()
    }
}

impl Checksum for ApmV1Action {
    fn checksum(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.realm.as_bytes());
        hasher.write(self.application.as_bytes());
        hasher.write(self.application_hash.as_bytes());
        hasher.write(self.action_kind.as_bytes());
        hasher.write(self.action_name.as_bytes());
        hasher.write_u64(self.additional_dimensions.checksum());

        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::util::checksum::Checksum;
    use palantir_proto::palantir::shared::tag::Tag;

    #[test]
    fn test_same_checksum_different_order() {
        let t1 = Tag {
            key: "a".to_string(),
            value: "a".to_string(),
        };
        let t2 = Tag {
            key: "b".to_string(),
            value: "b".to_string(),
        };
        let t3 = Tag {
            key: "c".to_string(),
            value: "c".to_string(),
        };

        let v1 = vec![t1.clone(), t2.clone(), t3.clone()];
        let v2 = vec![t3, t2, t1];

        assert_eq!(v1.checksum(), v2.checksum());
    }
}
