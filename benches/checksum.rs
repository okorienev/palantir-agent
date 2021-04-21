use criterion::{black_box, criterion_group, criterion_main, Criterion};
use palantir_agent_lib::util::checksum::Checksum;
use palantir_proto::palantir::apm::v1::action::ApmV1Action;
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use palantir_proto::palantir::shared::tag::Tag;

pub fn bench_checksum_tag_l10(c: &mut Criterion) {
    let tag = Tag {
        key: "0123456789".to_string(),
        value: "0123456789".to_string(),
    };

    c.bench_function("Checksum for tag", |b| b.iter(|| black_box(tag.checksum())));
}

pub fn checksum_vec_of_10_tags(c: &mut Criterion) {
    let tag = Tag {
        key: "0123456789".to_string(),
        value: "0123456789".to_string(),
    };
    let tags = vec![tag; 10];

    c.bench_function("Checksum for vec of 10 tags", |b| {
        b.iter(|| black_box(tags.checksum()))
    });
}

pub fn checksum_message_without_additional_dimensions(c: &mut Criterion) {
    let msg = ProtoMessage::ApmV1Action(ApmV1Action {
        realm: "example-realm".to_string(),
        application: "example-application".to_string(),
        application_hash: "3fde5".to_string(),
        action_kind: "http".to_string(),
        action_name: "controllers.example.long.enough.string".to_string(),
        total_us: 55_000_000u64,
        additional_dimensions: vec![],
        measurements: vec![],
    });

    c.bench_function("Checksum for message without additional dimensions", |b| {
        b.iter(|| black_box(msg.checksum()))
    });
}

criterion_group!(
    checksum,
    bench_checksum_tag_l10,
    checksum_vec_of_10_tags,
    checksum_message_without_additional_dimensions,
);

criterion_main!(checksum);
