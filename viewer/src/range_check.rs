use std::{
    fmt,
    ops::{Bound, RangeBounds},
};

pub fn range_check<T: PartialOrd>(
    range: &impl RangeBounds<T>,
    value: T,
) -> Result<(), OutOfRangeError<T>> {
    if range.contains(&value) {
        Ok(())
    } else {
        Err(OutOfRangeError {
            value,
            start: range.start_bound(),
            end: range.end_bound(),
        })
    }
}

#[derive(Debug)]
pub struct OutOfRangeError<'a, T> {
    value: T,
    start: Bound<&'a T>,
    end: Bound<&'a T>,
}

impl<T: std::fmt::Display> fmt::Display for OutOfRangeError<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Value {} is out of range: expected", self.value)?;
        match self.start {
            Bound::Excluded(v) => {
                write!(f, "{}..", v)?;
            }
            Bound::Included(v) => {
                write!(f, "{}..", v)?;
            }
            Bound::Unbounded => {
                write!(f, "..")?;
            }
        }
        match self.end {
            Bound::Excluded(v) => {
                write!(f, "{}", v)?;
            }
            Bound::Included(v) => {
                write!(f, "={}", v)?;
            }
            Bound::Unbounded => {}
        }
        Ok(())
    }
}

impl<T: fmt::Display + fmt::Debug> std::error::Error for OutOfRangeError<'_, T> {}
