//! Error conversion macros and utilities
//!
//! Provides macros to reduce boilerplate in error type conversions,
//! replacing 15+ manual `impl From<X> for Error` blocks.

/// Macro to implement From<SourceError> for TargetError
///
/// # Example
///
/// ```ignore
/// use ricecoder_common::impl_error_from;
///
/// #[derive(Debug)]
/// enum MyError {
///     Io(std::io::Error),
///     Parse(std::num::ParseIntError),
/// }
///
/// impl_error_from!(std::io::Error => MyError::Io);
/// impl_error_from!(std::num::ParseIntError => MyError::Parse);
/// ```
#[macro_export]
macro_rules! impl_error_from {
    ($source:ty => $target:ident::$variant:ident) => {
        impl From<$source> for $target {
            fn from(err: $source) -> Self {
                $target::$variant(err)
            }
        }
    };

    // Variant with custom transformation
    ($source:ty => $target:ident::$variant:ident, |$e:ident| $transform:expr) => {
        impl From<$source> for $target {
            fn from($e: $source) -> Self {
                $target::$variant($transform)
            }
        }
    };
}

/// Macro to implement multiple From conversions at once
///
/// # Example
///
/// ```ignore
/// use ricecoder_common::impl_errors_from;
///
/// impl_errors_from!(MyError {
///     Io(std::io::Error),
///     Parse(std::num::ParseIntError),
///     Json(serde_json::Error),
/// });
/// ```
#[macro_export]
macro_rules! impl_errors_from {
    ($target:ident { $($variant:ident($source:ty)),* $(,)? }) => {
        $(
            impl From<$source> for $target {
                fn from(err: $source) -> Self {
                    $target::$variant(err)
                }
            }
        )*
    };
}

/// Helper trait for error context
pub trait ErrorContext<T, E> {
    /// Add context to an error
    fn with_context<F, S>(self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> S,
        S: Into<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum TestError {
        Io(std::io::Error),
        Custom(String),
    }

    impl_error_from!(std::io::Error => TestError::Io);

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let test_err: TestError = io_err.into();
        assert!(matches!(test_err, TestError::Io(_)));
    }
}
