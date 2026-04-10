use std::fmt;

#[derive(Debug)]
pub enum BulkheadError<E = std::convert::Infallible> {
    Full { max_concurrent: usize },
    Inner(E),
}

impl<E: fmt::Display> fmt::Display for BulkheadError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BulkheadError::Full { max_concurrent } => {
                write!(
                    f,
                    "bulkhead full: max concurrent calls ({max_concurrent}) reached"
                )
            }
            BulkheadError::Inner(e) => write!(f, "{e}"),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for BulkheadError<E> {}
