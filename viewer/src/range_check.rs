use std::fmt;

#[derive(Debug)]
pub struct RangeCheck<T> {
    value: T,
    min: Option<(T, bool)>,
    max: Option<(T, bool)>,
}

impl<T: std::cmp::PartialOrd + Clone> RangeCheck<T> {
    pub fn lower_bound(value: T, min: (T, bool)) -> Self {
        Self {
            value,
            min: Some(min),
            max: None,
        }
    }
    pub fn upper_bound(value: T, max: (T, bool)) -> Self {
        Self {
            value,
            min: None,
            max: Some(max),
        }
    }
    pub fn upper_and_lower_bound(value: T, min: (T, bool), max: (T, bool)) -> Self {
        Self {
            value,
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn check(&self) -> bool {
        if let Some((min, min_eq)) = &self.min {
            if *min_eq {
                if self.value < *min {
                    return false;
                }
            } else {
                if self.value <= *min {
                    return false;
                }
            }
        }
        if let Some((max, max_eq)) = &self.max {
            if *max_eq {
                if self.value > *max {
                    return false;
                }
            } else {
                if self.value >= *max {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn check_result(&self, name: String) -> Result<(), OutOfRangeError<T>> {
        if self.check() {
            Ok(())
        } else {
            Err(OutOfRangeError {
                name,
                value: self.value.clone(),
                min: self.min.clone(),
                max: self.max.clone(),
            })
        }
    }
}

#[derive(Debug)]
pub struct OutOfRangeError<T> {
    name: String,
    value: T,
    min: Option<(T, bool)>,
    max: Option<(T, bool)>,
}

impl<T: fmt::Display> fmt::Display for OutOfRangeError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Value {}={} is out of range: expected ",
            self.name, self.value
        )?;
        if let Some((min, eq)) = &self.min {
            if *eq {
                write!(f, "{} <= ", min)?;
            } else {
                write!(f, "{} < ", min)?;
            }
        }
        write!(f, "{}", self.name)?;
        if let Some((max, eq)) = &self.max {
            if *eq {
                write!(f, " <= {}", max)?;
            } else {
                write!(f, " < {}", max)?;
            }
        }
        Ok(())
    }
}

impl<T: fmt::Display + fmt::Debug> std::error::Error for OutOfRangeError<T> {}
