use airbag::{backends::Backend, Alert};
use std::sync::Arc;

#[test]
fn test_middleware_title_prefix() {
    let backend = TestBackend::default();
    let target = backend.target();

    let _guard = airbag::configure_thread_local(backend)
        .install(airbag::middleware::TitlePrefix::new("some_prefix:"));

    airbag::trigger(Alert::builder().title("hello")).wait_processed();

    let alert = target.lock().pop().unwrap();
    assert_eq!(alert.title(), &Some("some_prefix:hello".into()));
}

#[test]
fn test_middleware_dedup_prefix() {
    let backend = TestBackend::default();
    let target = backend.target();

    let _guard = airbag::configure_thread_local(backend)
        .install(airbag::middleware::DedupKeyPrefix::new("some_prefix:"));

    airbag::trigger(Alert::builder().title("hello").dedup_key("dedup_key")).wait_processed();

    let alert = target.lock().pop().unwrap();
    assert_eq!(alert.title(), &Some("hello".into()));
    assert_eq!(alert.dedup_key(), &Some("some_prefix:dedup_key".into()));
}

#[test]
fn test_middleware_map() {
    let backend = TestBackend::default();
    let target = backend.target();

    let _guard = airbag::configure_thread_local(backend).map(|alert| alert.with_field("x", "y"));

    Alert::builder().title("hello").trigger().wait_processed();

    let alert = target.lock().pop().expect("No alert");

    assert_eq!(
        alert.get_field("x").expect("No field x").as_str(),
        Some("y")
    );
}

#[derive(Default)]
pub struct TestBackend {
    target: Arc<parking_lot::Mutex<Vec<airbag::Alert>>>,
}

impl TestBackend {
    pub fn target(&self) -> Arc<parking_lot::Mutex<Vec<airbag::Alert>>> {
        self.target.clone()
    }
}

impl Backend for TestBackend {
    fn send(&mut self, alert: Alert) -> anyhow::Result<()> {
        println!("Alerting: {:?}", alert.get_fields());
        self.target.lock().push(alert);
        Ok(())
    }
}
