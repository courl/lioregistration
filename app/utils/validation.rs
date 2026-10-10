use crate::error::{AppError, AppResult};
use chrono::NaiveDate;

/// Upper bound for free-text form fields; generous, but stops megabyte-sized payloads in the DB.
pub const MAX_FIELD_LEN: usize = 200;

pub fn ensure_date_range(start: NaiveDate, end: NaiveDate, what: &str) -> AppResult<()> {
    if end < start {
        return Err(AppError::BadRequest(format!(
            "{what}: end date ({end}) is before start date ({start})"
        )));
    }
    Ok(())
}

/// Trims `value` in place and checks it is non-empty and at most [`MAX_FIELD_LEN`] chars.
pub fn required_text(value: &mut String, field: &str) -> AppResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{field} is required")));
    }
    if trimmed.chars().count() > MAX_FIELD_LEN {
        return Err(AppError::BadRequest(format!(
            "{field} must be at most {MAX_FIELD_LEN} characters"
        )));
    }
    if trimmed.len() != value.len() {
        *value = trimmed.to_owned();
    }
    Ok(())
}

/// Minimal structural check (`local@domain.tld`, no whitespace). Deliverability is proven by
/// the credentials email, not here.
pub fn ensure_email(value: &str) -> AppResult<()> {
    let valid = !value.chars().any(char::is_whitespace)
        && value.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && !domain.contains('@')
                && domain
                    .split_once('.')
                    .is_some_and(|(host, tld)| !host.is_empty() && !tld.is_empty())
        });
    if valid {
        Ok(())
    } else {
        Err(AppError::BadRequest("email is not a valid address".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn date_range() {
        assert!(ensure_date_range(d(2026, 1, 1), d(2026, 1, 1), "x").is_ok());
        assert!(ensure_date_range(d(2026, 1, 1), d(2026, 2, 1), "x").is_ok());
        assert!(ensure_date_range(d(2026, 2, 1), d(2026, 1, 1), "x").is_err());
    }

    #[test]
    fn required_text_trims_and_bounds() {
        let mut v = "  Ada ".to_string();
        required_text(&mut v, "name").unwrap();
        assert_eq!(v, "Ada");
        assert!(required_text(&mut "   ".to_string(), "name").is_err());
        assert!(required_text(&mut "x".repeat(MAX_FIELD_LEN + 1), "name").is_err());
        assert!(required_text(&mut "é".repeat(MAX_FIELD_LEN), "name").is_ok());
    }

    #[test]
    fn email_shape() {
        assert!(ensure_email("a@b.lu").is_ok());
        assert!(ensure_email("first.last@sub.domain.lu").is_ok());
        for bad in [
            "", "a", "a@b", "@b.lu", "a@.lu", "a@b.", "a b@c.lu", "a@b@c.lu",
        ] {
            assert!(ensure_email(bad).is_err(), "{bad} should be rejected");
        }
    }
}
