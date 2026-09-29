use serde_yaml::Value;

const CI_WORKFLOW: &str = include_str!("../.github/workflows/ci.yml");

#[test]
fn workflow_does_not_replace_pending_runs_with_concurrency_groups() {
    let workflow: Value = serde_yaml::from_str(CI_WORKFLOW).expect("CI workflow is valid YAML");
    let workflow = workflow.as_mapping().expect("CI workflow is a mapping");
    let concurrency = Value::String("concurrency".to_string());

    assert!(
        !workflow.contains_key(&concurrency),
        "workflow-level concurrency can replace pending runs"
    );

    let jobs = workflow
        .get(&Value::String("jobs".to_string()))
        .and_then(Value::as_mapping)
        .expect("CI workflow defines jobs");
    for (name, job) in jobs {
        let job = job
            .as_mapping()
            .unwrap_or_else(|| panic!("job {name:?} is a mapping"));
        assert!(
            !job.contains_key(&concurrency),
            "job {name:?} concurrency can replace pending runs"
        );
    }
}
