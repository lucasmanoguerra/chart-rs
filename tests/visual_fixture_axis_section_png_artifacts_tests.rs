use serde::Deserialize;
use std::path::Path;

const FIXTURE_CORPUS_JSON: &str =
    include_str!("fixtures/axis_section_sizing/axis_section_sizing_corpus.json");

#[derive(Debug, Deserialize)]
struct FixtureCorpus {
    fixtures: Vec<AxisSectionSizingFixture>,
}

#[derive(Debug, Deserialize)]
struct AxisSectionSizingFixture {
    id: String,
    #[serde(default)]
    artifacts: FixtureArtifacts,
}

#[derive(Debug, Deserialize, Default)]
struct FixtureArtifacts {
    reference_png_relpath: Option<String>,
}

#[test]
fn axis_section_fixture_png_references_exist_when_declared() {
    let corpus: FixtureCorpus =
        serde_json::from_str(FIXTURE_CORPUS_JSON).expect("fixture corpus should parse");
    let fixture_count = corpus.fixtures.len();
    assert!(fixture_count > 0, "fixture corpus should not be empty");

    let mut missing = Vec::new();
    let mut declared = 0usize;

    for fixture in &corpus.fixtures {
        if let Some(path) = &fixture.artifacts.reference_png_relpath {
            declared += 1;
            if !Path::new(path).exists() {
                missing.push(format!("fixture `{}` missing `{}`", fixture.id, path));
            }
        }
    }

    assert_eq!(
        declared, fixture_count,
        "all fixtures should declare `reference_png_relpath`"
    );
    assert!(
        missing.is_empty(),
        "missing axis-section reference png files:\n{}",
        missing.join("\n")
    );
}
