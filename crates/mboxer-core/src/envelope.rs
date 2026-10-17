//! Recognition of mbox envelope lines.
//!
//! An envelope line separates messages in an mbox file. It is recognised by shape
//! rather than by a producer-specific pattern, so Gmail Takeout
//! (`From 1234@xxx Tue Jan 13 06:30:04 +0000 2026`), classic Unix mbox
//! (`From alice@example.com Mon Jan  5 09:00:00 2026`) and Thunderbird
//! (`From - Sat Jan 03 11:52:46 2026`) all match, while a body line such as
//! `From Anna wrote:` does not.

const WEEKDAYS: [&[u8]; 7] = [b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat", b"Sun"];
const MONTHS: [&[u8]; 12] = [
    b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec",
];

/// Returns `true` when `line` is an mbox envelope line.
///
/// The line may carry its trailing `\n` or `\r\n`. The expected shape is
/// `From <sender> <weekday> <month> <day> <hh:mm:ss> [<tz>] <year>`, where the
/// sender is any single token (`-` and `MAILER-DAEMON` included), the day has one
/// or two digits, the optional zone is `+hhmm`/`-hhmm`, and the year has four digits.
pub fn is_envelope(line: &[u8]) -> bool {
    let Some(rest) = line.strip_prefix(b"From ") else {
        return false;
    };
    let rest = trim_line_break(rest);
    let mut fields = [&[][..]; 8];
    let mut n = 0;
    for field in rest.split(|b| *b == b' ').filter(|f| !f.is_empty()) {
        if n == fields.len() {
            return false;
        }
        fields[n] = field;
        n += 1;
    }
    match n {
        6 => is_date(&fields[1..6], None),
        7 => is_date(&fields[1..5], Some(fields[5])) && is_year(fields[6]),
        _ => false,
    }
}

/// `fields` = weekday, month, day, time[, year]; `zone` = the optional `+hhmm`.
fn is_date(fields: &[&[u8]], zone: Option<&[u8]>) -> bool {
    let [weekday, month, day, time, tail @ ..] = fields else {
        return false;
    };
    let year_ok = match (tail, zone) {
        ([year], None) => is_year(year),
        ([], Some(zone)) => is_zone(zone),
        _ => false,
    };
    WEEKDAYS.contains(weekday)
        && MONTHS.contains(month)
        && (1..=2).contains(&day.len())
        && day.iter().all(u8::is_ascii_digit)
        && is_time(time)
        && year_ok
}

fn is_time(t: &[u8]) -> bool {
    t.len() == 8
        && t[2] == b':'
        && t[5] == b':'
        && [0, 1, 3, 4, 6, 7].iter().all(|&i| t[i].is_ascii_digit())
}

fn is_zone(z: &[u8]) -> bool {
    z.len() == 5 && (z[0] == b'+' || z[0] == b'-') && z[1..].iter().all(u8::is_ascii_digit)
}

fn is_year(y: &[u8]) -> bool {
    y.len() == 4 && y.iter().all(u8::is_ascii_digit)
}

/// Strips one trailing `\n` or `\r\n`.
pub(crate) fn trim_line_break(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmail_takeout_envelope() {
        assert!(is_envelope(
            b"From 1854182055123456789@xxx Tue Jan 13 06:30:04 +0000 2026\r\n"
        ));
    }

    #[test]
    fn classic_and_thunderbird_envelopes() {
        assert!(is_envelope(
            b"From alice@example.com Mon Jan  5 09:00:00 2026\n"
        ));
        assert!(is_envelope(b"From - Sat Jan 03 11:52:46 2026"));
        assert!(is_envelope(b"From MAILER-DAEMON Wed Jan  7 11:00:00 2026"));
    }

    #[test]
    fn body_lines_are_not_envelopes() {
        assert!(!is_envelope(b"From Anna wrote:\n"));
        assert!(!is_envelope(b"From the top\n"));
        assert!(!is_envelope(
            b">From 1234@xxx Tue Jan 13 06:30:04 +0000 2026\n"
        ));
        assert!(!is_envelope(b"From: Anna <anna@example.com>\n"));
        assert!(!is_envelope(b"From Tue Jan 13 06:30:04 2026\n"));
        assert!(!is_envelope(b"From x Tue Jan 13 06:30:04 +0000 26\n"));
        assert!(!is_envelope(b"From x Tue Jan 13 06:30 +0000 2026\n"));
        assert!(!is_envelope(b"From x Xyz Jan 13 06:30:04 +0000 2026\n"));
        assert!(!is_envelope(
            b"From x Tue Jan 13 06:30:04 +0000 2026 extra\n"
        ));
        assert!(!is_envelope(b""));
        assert!(!is_envelope(b"From "));
    }
}
