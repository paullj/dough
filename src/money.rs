use serde::{Deserialize, Serialize};
use sqlx::{Database, Decode, Encode, Sqlite, Type};
use std::fmt;

/// Money represented as cents/pence (smallest currency unit)
/// Using i64 to support both positive and negative amounts
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cents(pub i64);

impl Cents {
    #[allow(dead_code)]
    pub fn new(cents: i64) -> Self {
        Cents(cents)
    }

    pub fn from_major_units(units: f64) -> Self {
        Cents((units * 100.0).round() as i64)
    }

    pub fn to_major_units(self) -> f64 {
        self.0 as f64 / 100.0
    }

    pub fn format_gbp(&self) -> String {
        let is_negative = self.0 < 0;
        let abs_cents = self.0.abs();
        let pounds = abs_cents / 100;
        let pence = abs_cents % 100;

        if is_negative {
            format!("-£{}.{:02}", pounds, pence)
        } else {
            format!("£{}.{:02}", pounds, pence)
        }
    }

    pub fn format_currency(&self, symbol: &str) -> String {
        let is_negative = self.0 < 0;
        let abs_cents = self.0.abs();
        let major = abs_cents / 100;
        let minor = abs_cents % 100;

        if is_negative {
            format!("-{}{}.{:02}", symbol, major, minor)
        } else {
            format!("{}{}.{:02}", symbol, major, minor)
        }
    }
}

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_gbp())
    }
}

// SQLx type implementation for automatic conversion
impl Type<Sqlite> for Cents {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for Cents {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as Encode<'q, Sqlite>>::encode_by_ref(&self.0, buf)
    }
}

impl<'r> Decode<'r, Sqlite> for Cents {
    fn decode(value: <Sqlite as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let cents = <i64 as Decode<Sqlite>>::decode(value)?;
        Ok(Cents(cents))
    }
}

// Arithmetic operations
impl std::ops::Add for Cents {
    type Output = Cents;

    fn add(self, other: Cents) -> Cents {
        Cents(self.0 + other.0)
    }
}

impl std::ops::Sub for Cents {
    type Output = Cents;

    fn sub(self, other: Cents) -> Cents {
        Cents(self.0 - other.0)
    }
}

impl std::ops::Neg for Cents {
    type Output = Cents;

    fn neg(self) -> Cents {
        Cents(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cents_creation() {
        let c = Cents::new(10000);
        assert_eq!(c.0, 10000);
    }

    #[test]
    fn test_from_major_units() {
        let c = Cents::from_major_units(100.50);
        assert_eq!(c.0, 10050);
    }

    #[test]
    fn test_formatting() {
        let c = Cents::new(10050);
        assert_eq!(c.format_gbp(), "£100.50");

        let c = Cents::new(-5025);
        assert_eq!(c.format_gbp(), "-£50.25");
    }
}
