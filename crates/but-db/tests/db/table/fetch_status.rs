use but_db::FetchStatus;

use super::in_memory_db;

#[test]
fn starts_without_status() -> anyhow::Result<()> {
    let db = in_memory_db();

    assert_eq!(
        db.fetch_status().get()?,
        None,
        "a new project has no fetch history"
    );
    Ok(())
}

#[test]
fn records_success() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    db.fetch_status_mut().record_success(10)?;

    assert_eq!(
        db.fetch_status().get()?,
        Some(FetchStatus {
            last_attempted_ms: 10,
            last_successful_ms: Some(10),
            last_error: None,
        }),
        "a successful attempt is also the latest successful fetch"
    );
    Ok(())
}

#[test]
fn failure_preserves_last_success() -> anyhow::Result<()> {
    let mut db = in_memory_db();
    db.fetch_status_mut().record_success(10)?;

    db.fetch_status_mut().record_failure(20, "offline")?;

    assert_eq!(
        db.fetch_status().get()?,
        Some(FetchStatus {
            last_attempted_ms: 20,
            last_successful_ms: Some(10),
            last_error: Some("offline".to_owned()),
        }),
        "a failed attempt must not erase the last successful fetch"
    );
    Ok(())
}

#[test]
fn success_clears_last_error() -> anyhow::Result<()> {
    let mut db = in_memory_db();
    db.fetch_status_mut().record_failure(10, "offline")?;

    db.fetch_status_mut().record_success(20)?;

    assert_eq!(
        db.fetch_status().get()?,
        Some(FetchStatus {
            last_attempted_ms: 20,
            last_successful_ms: Some(20),
            last_error: None,
        }),
        "a later success clears the previous failure"
    );
    Ok(())
}
