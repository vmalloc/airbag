use crate::ProcessingReceipt;
use serde_json::json;
use std::panic::PanicHookInfo;

pub struct Alert {
    id: u64,
    meta: AlertMeta,

    value: serde_json::Value,
}

static ALERT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Alert {
    pub fn builder() -> AlertBuilder {
        AlertBuilder {
            id: ALERT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            meta: AlertMeta::default(),
            value: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn title(&self) -> &Option<String> {
        &self.meta.title
    }

    pub fn dedup_key(&self) -> &Option<String> {
        &self.meta.dedup_key
    }

    pub(crate) fn build_error_alert<E: std::fmt::Debug + 'static>(e: &E) -> AlertBuilder {
        let mut returned = Self::builder();
        let e_any: &dyn std::any::Any = e;
        let e_dbg = format!("{:?}", e);

        log::debug!("Building error alert for {e_dbg}");

        if let Some(e) = e_any.downcast_ref::<anyhow::Error>() {
            returned = returned.title(e.to_string()).description(&e_dbg);
        } else {
            returned = returned.title(&e_dbg);
        }

        if returned.meta.dedup_key.is_none() {
            returned = returned.dedup_key(crate::utils::sha256(&e_dbg));
        }

        returned
    }

    pub(crate) fn build_panic_alert(info: &PanicHookInfo) -> AlertBuilder {
        let location = if let Some(location) = info.location() {
            format!("{:?}", location)
        } else {
            String::from("<unknown>")
        };
        let summary = format!(
            "Panic at {}: {}",
            location,
            info.payload()
                .downcast_ref::<String>()
                .map(String::as_str)
                .unwrap_or("N/A")
        );

        let dedup_key = crate::utils::sha256(&summary);

        Self::builder()
            .title(summary.clone())
            .description(summary)
            .dedup_key(dedup_key)
    }

    pub(crate) fn as_json(&self) -> &serde_json::Value {
        &self.value
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn meta(&self) -> &AlertMeta {
        &self.meta
    }

    pub fn get_field(&self, name: impl AsRef<str>) -> Option<&serde_json::Value> {
        self.value.get(name.as_ref())
    }

    pub fn get_fields(&self) -> &serde_json::Value {
        &self.value
    }

    /// Consumes this alert, sets a field on it and returns a new alert with the set field
    pub fn with_field(
        mut self,
        field_name: impl AsRef<str>,
        field_value: impl serde::Serialize,
    ) -> Self {
        self.value[field_name.as_ref()] = json!(field_value);
        self
    }

    /// Consumes this alert, sets a field on it if it does not exist already and returns a new alert with the set field
    pub fn with_field_if_missing(
        mut self,
        field_name: impl AsRef<str>,
        field_value: impl serde::Serialize,
    ) -> Self {
        let field_name = field_name.as_ref();
        if self.value.get(field_name).is_none() {
            self.value[field_name] = json!(field_value);
        }
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
}

impl From<Severity> for Priority {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Critical => Self::P1,
            Severity::Error => Self::P2,
            Severity::Warning => Self::P3,
            Severity::Info => Self::P4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Priority {
    P1,
    P2,
    P3,
    P4,
    P5,
}

impl From<Priority> for Severity {
    fn from(value: Priority) -> Self {
        match value {
            Priority::P1 => Self::Critical,
            Priority::P2 => Self::Error,
            Priority::P3 => Self::Warning,
            Priority::P4 | Priority::P5 => Self::Info,
        }
    }
}

#[derive(Default)]
pub(crate) struct AlertMeta {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) dedup_key: Option<String>,
    pub(crate) severity: Option<Severity>,
    pub(crate) priority: Option<Priority>,
}

pub struct AlertBuilder {
    id: u64,
    meta: AlertMeta,
    value: serde_json::Value,
}

impl AlertBuilder {
    pub fn field(mut self, field: impl FieldSpec, value: impl serde::Serialize) -> Self {
        field.fill(&mut self.value, value);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.meta.title.replace(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.meta.description.replace(description.into());
        self
    }

    pub fn dedup_key(mut self, dedup_key: impl Into<String>) -> Self {
        self.meta.dedup_key.replace(dedup_key.into());
        self
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.meta.severity.replace(severity);
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.meta.priority.replace(priority);
        self
    }

    pub fn build(self) -> Alert {
        self.into()
    }

    pub fn trigger(self) -> ProcessingReceipt {
        crate::hub::trigger(self.build())
    }
}

impl From<AlertBuilder> for Alert {
    fn from(builder: AlertBuilder) -> Self {
        Self {
            value: builder.value,
            meta: builder.meta,
            id: builder.id,
        }
    }
}

pub trait FieldSpec {
    fn fill(&self, target: &mut serde_json::Value, value: impl serde::Serialize);
}

impl<S: AsRef<str>> FieldSpec for S {
    fn fill(&self, target: &mut serde_json::Value, value: impl serde::Serialize) {
        target[self.as_ref()] = json!(value);
    }
}

pub(crate) mod middleware {

    /// Prefixes alert titles with a specific string. This is useful in cases where you want alert titles to easily convey
    /// instance or context information
    ///
    /// ```
    /// use airbag::prelude::*;
    ///
    /// let _guard = airbag::configure(
    ///   airbag::backends::PagerDuty::builder().token("your token").build()
    /// ).with_middleware(
    ///   airbag::middleware::TitlePrefix::new("your prefix")
    /// );
    /// ```
    pub struct TitlePrefix {
        prefix: String,
    }

    impl TitlePrefix {
        pub fn new(prefix: impl Into<String>) -> Self {
            let prefix = prefix.into();
            Self { prefix }
        }
    }

    impl crate::middleware::Middleware for TitlePrefix {
        fn process(&self, mut alert: super::Alert) -> crate::alert::Alert {
            alert.meta.title.replace(format!(
                "{}{}",
                self.prefix,
                alert.meta.title.as_deref().unwrap_or("")
            ));
            alert
        }
    }

    /// Prefixes dedup keys with a specific string. This is useful in case you have multiple instances of a specific app
    /// and you want the dedup keys from one instance to not interfere with dedup keys of another
    /// ```
    /// use airbag::prelude::*;
    ///
    /// let _guard = airbag::configure(
    ///   airbag::backends::PagerDuty::builder().token("your token").build()
    /// ).with_middleware(
    ///   airbag::middleware::DedupKeyPrefix::new("your prefix")
    /// );
    /// ```

    pub struct DedupKeyPrefix {
        prefix: String,
    }

    impl DedupKeyPrefix {
        pub fn new(prefix: impl Into<String>) -> Self {
            let prefix = prefix.into();
            Self { prefix }
        }
    }

    impl crate::middleware::Middleware for DedupKeyPrefix {
        fn process(&self, mut alert: super::Alert) -> crate::alert::Alert {
            if let Some(dedup_key) = alert.meta.dedup_key.take() {
                alert
                    .meta
                    .dedup_key
                    .replace(format!("{}{}", self.prefix, dedup_key));
            }
            alert
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Alert;

    #[test]
    fn test_with_field() {
        let alert = Alert::builder().build().with_field("x", "y");
        assert_eq!(alert.get_field("x").unwrap().as_str().unwrap(), "y");
    }

    #[test]
    fn test_with_field_if_missing_missing() {
        let alert = Alert::builder().build().with_field_if_missing("x", "y");
        assert_eq!(alert.get_field("x").unwrap().as_str().unwrap(), "y");
    }

    #[test]
    fn test_with_field_if_missing_exists() {
        let alert = Alert::builder()
            .build()
            .with_field("x", "1")
            .with_field_if_missing("x", "y");
        assert_eq!(alert.get_field("x").unwrap().as_str().unwrap(), "1");
    }
}
