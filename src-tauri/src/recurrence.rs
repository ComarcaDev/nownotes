//! Cálculo de la siguiente fecha de una tarea que se repite (en hora local).

use chrono::{
    DateTime, Datelike, Duration, Local, LocalResult, Months, NaiveDateTime, TimeZone, Weekday,
};

pub const RULES: &[&str] = &["none", "daily", "weekdays", "weekly", "monthly", "yearly"];

pub fn normalize(rule: &str) -> &'static str {
    RULES.iter().copied().find(|r| *r == rule).unwrap_or("none")
}

/// Avanza una repetición manteniendo la hora "de reloj" local.
pub fn step<Tz: TimeZone>(dt: &DateTime<Tz>, rule: &str) -> Option<DateTime<Tz>> {
    let tz = dt.timezone();
    let naive = dt.naive_local();
    let next: NaiveDateTime = match rule {
        "daily" => naive + Duration::days(1),
        "weekdays" => {
            let mut n = naive + Duration::days(1);
            while matches!(n.weekday(), Weekday::Sat | Weekday::Sun) {
                n += Duration::days(1);
            }
            n
        }
        "weekly" => naive + Duration::weeks(1),
        // chrono ajusta al último día del mes si el día no existe (31 ene → 28/29 feb).
        "monthly" => naive.checked_add_months(Months::new(1))?,
        "yearly" => naive.checked_add_months(Months::new(12))?,
        _ => return None,
    };
    match tz.from_local_datetime(&next) {
        LocalResult::Single(d) => Some(d),
        LocalResult::Ambiguous(a, _) => Some(a),
        // Hueco por cambio de hora: se usa la hora siguiente.
        LocalResult::None => tz
            .from_local_datetime(&(next + Duration::hours(1)))
            .earliest(),
    }
}

/// Primera repetición estrictamente posterior a `after`.
pub fn next_after<Tz: TimeZone>(
    start: DateTime<Tz>,
    rule: &str,
    after_ms: i64,
) -> Option<DateTime<Tz>> {
    let mut dt = start;
    for _ in 0..200_000 {
        dt = step(&dt, rule)?;
        if dt.timestamp_millis() > after_ms {
            return Some(dt);
        }
    }
    None
}

pub fn next_after_ms(ts_ms: i64, rule: &str, after_ms: i64) -> Option<i64> {
    let start = Local.timestamp_millis_opt(ts_ms).single()?;
    next_after(start, rule, after_ms).map(|d| d.timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn utc(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }

    #[test]
    fn daily_and_weekly() {
        assert_eq!(
            step(&utc(2026, 11, 18, 9), "daily"),
            Some(utc(2026, 11, 19, 9))
        );
        assert_eq!(
            step(&utc(2026, 11, 18, 9), "weekly"),
            Some(utc(2026, 11, 25, 9))
        );
    }

    #[test]
    fn weekdays_skip_weekend() {
        // 20 nov 2026 es viernes → lunes 23
        assert_eq!(
            step(&utc(2026, 11, 20, 9), "weekdays"),
            Some(utc(2026, 11, 23, 9))
        );
        // sábado → lunes
        assert_eq!(
            step(&utc(2026, 11, 21, 9), "weekdays"),
            Some(utc(2026, 11, 23, 9))
        );
    }

    #[test]
    fn monthly_end_of_month_and_leap() {
        assert_eq!(
            step(&utc(2028, 1, 31, 9), "monthly"),
            Some(utc(2028, 2, 29, 9))
        );
        assert_eq!(
            step(&utc(2027, 1, 31, 9), "monthly"),
            Some(utc(2027, 2, 28, 9))
        );
        assert_eq!(
            step(&utc(2028, 2, 29, 9), "yearly"),
            Some(utc(2029, 2, 28, 9))
        );
    }

    #[test]
    fn next_after_skips_missed() {
        let start = utc(2026, 11, 1, 9);
        let after = utc(2026, 11, 18, 12).timestamp_millis();
        assert_eq!(
            next_after(start, "daily", after),
            Some(utc(2026, 11, 19, 9))
        );
    }

    #[test]
    fn none_does_not_repeat() {
        assert_eq!(step(&utc(2026, 11, 18, 9), "none"), None);
        assert_eq!(normalize("loquesea"), "none");
    }
}
