use std::fmt;

pub trait Logit<T, E: fmt::Debug> {
    fn logit(self, msg: impl fmt::Display) -> Result<T, E>;
    fn with_logit<F, C>(self, f: F) -> Result<T, E>
    where
        C: fmt::Display,
        F: FnOnce() -> C;

    fn logit_warn(self, msg: impl fmt::Display) -> Result<T, E>;
    fn with_logit_warn<F, C>(self, f: F) -> Result<T, E>
    where
        C: fmt::Display,
        F: FnOnce() -> C;
}

impl<T, E: fmt::Debug> Logit<T, E> for Result<T, E> {
    fn logit(self, msg: impl fmt::Display) -> Result<T, E> {
        self.map_err(|e| {
            tracing::error!(error = ?e, "{}", msg);
            e
        })
    }

    fn with_logit<F, C>(self, f: F) -> Result<T, E>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.logit(f())
    }

    fn logit_warn(self, msg: impl fmt::Display) -> Result<T, E> {
        self.map_err(|e| {
            tracing::warn!(error = ?e, "{}", msg);
            e
        })
    }

    fn with_logit_warn<F, C>(self, f: F) -> Result<T, E>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.logit_warn(f())
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use tracing_test::traced_test;

    use crate::Logit;

    #[traced_test]
    #[test]
    fn test_logit() {
        let err_res: io::Result<()> = Err(io::ErrorKind::Other.into());
        assert!(err_res.logit("message").is_err());
        assert!(logs_contain("message"));
    }
}
