pub trait AirbagResult<E>: Sized {
    fn airbag_drop(self) {
        drop(self.airbag())
    }

    fn airbag_drop_with_dedup_key<S: Into<String>, F: Fn() -> S>(self, dedup_key_factory: F) {
        drop(self.airbag_with_dedup_key(dedup_key_factory))
    }

    fn airbag_with_dedup_key<S: Into<String>, F: Fn() -> S>(self, dedup_key_factory: F) -> Self;

    fn airbag_if<F: Fn(&E) -> bool>(self, f: F) -> Self;

    fn airbag(self) -> Self;
}

impl<T, E: std::fmt::Debug + 'static> AirbagResult<E> for Result<T, E> {
    fn airbag(self) -> Self {
        if let Err(e) = &self {
            log::error!("Airbag: handling error {e:?}");
            crate::trigger(crate::alert::Alert::build_error_alert(e));
        }
        self
    }

    fn airbag_if<F: Fn(&E) -> bool>(self, f: F) -> Self {
        if let Err(e) = &self {
            if f(e) {
                return self.airbag();
            }
        }
        self
    }

    fn airbag_with_dedup_key<S: Into<String>, F: Fn() -> S>(self, dedup_key_factory: F) -> Self {
        if let Err(e) = &self {
            crate::trigger(
                crate::alert::Alert::build_error_alert(e).dedup_key(dedup_key_factory()),
            );
        }
        self
    }
}
