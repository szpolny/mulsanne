//! Upcoming LMU daily races, scraped from the RaceControl homepage so the
//! schedule can be browsed without the game running.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

const URL: &str = "https://racecontrol.gg/";
/// Marks the LMU section; the page also lists rFactor 2 daily races.
const LMU_MARKER: &str = "lmu_logo";

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DailyRace {
    pub name: String,
    pub tier: String,
    pub track: String,
    pub duration: String,
    pub image: Option<String>,
    /// Start times as Unix milliseconds (UTC).
    pub starts: Vec<i64>,
}

pub async fn fetch_daily_races() -> Result<Vec<DailyRace>, String> {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    let client = reqwest::Client::builder()
        .user_agent(concat!("Mulsanne/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let html = client
        .get(URL)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Could not reach RaceControl: {e}"))?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    parse(&html, now_ms).ok_or_else(|| "The RaceControl page layout has changed".into())
}

/// Parses the LMU "Upcoming Daily Races" section. Returns `None` when the
/// section cannot be found at all.
pub fn parse(html: &str, now_ms: i64) -> Option<Vec<DailyRace>> {
    let section = html
        .split(r#"class="schedule-header"#)
        .skip(1)
        .find(|s| s.split("</section>").next().is_some_and(|head| head.contains(LMU_MARKER)))?;
    let section = section.split("</section>").next()?;

    let races = section
        .split(r#"class="scheduled-race-card"#)
        .skip(1)
        .filter_map(|card| {
            let name = text_of(between(card, "<h4", "</h4>")?);
            let tier = text_of(between(card, "tier-badge", "</span>")?);
            let image = between(card, r#"<img src=""#, "\"").map(str::to_owned);
            // Detail rows: label span, divider, value span.
            let mut track = String::new();
            let mut duration = String::new();
            for row in card.split("race-header").skip(1) {
                let spans: Vec<String> = row
                    .split("<span")
                    .skip(1)
                    .filter_map(|s| between(s, ">", "</span>").map(text_of))
                    .collect();
                if let [label, value, ..] = spans.as_slice() {
                    match label.to_ascii_lowercase().as_str() {
                        "track" => track = value.clone(),
                        "duration" => duration = value.clone(),
                        _ => {}
                    }
                }
            }
            let starts = card
                .split("bg-gray-900-40\">")
                .skip(1)
                .filter_map(|s| s.split('<').next())
                .filter_map(|t| parse_start(t.trim(), now_ms))
                .collect();
            Some(DailyRace { name, tier: tier.to_lowercase(), track, duration, image, starts })
        })
        .collect();
    Some(races)
}

fn between<'a>(s: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = s.find(start)? + start.len();
    let len = s[from..].find(end)?;
    Some(&s[from..from + len])
}

/// Strips tags and attribute leftovers, collapses whitespace and decodes the
/// common entities.
fn text_of(fragment: &str) -> String {
    // A fragment may start inside an opening tag (`<h4 class="...">Name`).
    let fragment = match (fragment.find('>'), fragment.find('<')) {
        (Some(gt), lt) if lt.is_none_or(|lt| gt < lt) => &fragment[gt + 1..],
        _ => fragment,
    };
    let mut out = String::new();
    let mut in_tag = false;
    for c in fragment.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parses `13 Oct at 8:45pm` (UTC, no year). The year is the one that puts
/// the time closest to `now_ms`, which handles the December/January rollover.
fn parse_start(text: &str, now_ms: i64) -> Option<i64> {
    let mut parts = text.split_whitespace();
    let day: u32 = parts.next()?.parse().ok()?;
    let month = month_index(parts.next()?)?;
    if parts.next()? != "at" {
        return None;
    }
    let clock = parts.next()?.to_ascii_lowercase();
    let (clock, pm) = match clock.strip_suffix("pm") {
        Some(c) => (c.to_owned(), true),
        None => (clock.strip_suffix("am")?.to_owned(), false),
    };
    let (h, m) = clock.split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    if !(1..=12).contains(&h) || m > 59 || !(1..=31).contains(&day) {
        return None;
    }
    let hour = h % 12 + if pm { 12 } else { 0 };

    let now_year = civil_from_days(now_ms.div_euclid(86_400_000)).0;
    (now_year - 1..=now_year + 1)
        .map(|y| (days_from_civil(y, month, day) * 86_400 + hour as i64 * 3600 + m as i64 * 60) * 1000)
        .min_by_key(|t| (t - now_ms).abs())
}

fn month_index(name: &str) -> Option<u32> {
    const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    let key = name.get(..3)?.to_ascii_lowercase();
    MONTHS.iter().position(|m| *m == key).map(|i| i as u32 + 1)
}

// Howard Hinnant's civil date algorithms.
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + if m <= 2 { 1 } else { 0 }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: &str = r##"
<div class="schedule-header mb-3"><h2>Upcoming Daily Races</h2>
  <img src="/assets/clients/race_control/lmu_logo_dark-abc.png" /></div>
<ul><li class="glide__slide">
<div class="scheduled-race-card position-relative">
  <span class="badge tier-badge"><i class="fa-solid fa-medal"></i>
    intermediate
  </span>
  <a href="#"><img src="https://example.com/spa.webp" alt="ELMS" loading="lazy"></a>
  <h4 class="text-white fs-xl mb-2">ELMS Sprint &amp; Trophy</h4>
  <div class="race-header d-flex">
    <span class="text-gray-200"><i class="fa-solid fa-timer me-1"></i>
      Duration
    </span>
    <hr class="w-100">
    <span class="text-gray-200">
      44 minutes
    </span>
  </div>
  <div class="race-header d-flex">
    <span class="text-gray-200"><i class="fa-solid fa-route me-1"></i> Track </span>
    <hr>
    <span class="text-gray-200"> Spa Francorchamps </span>
  </div>
  <span class="badge text-yellow bg-gray-900-40">13 Oct at 8:50pm</span>
  <span class="badge text-yellow bg-gray-900-40">13 Oct at 12:05am</span>
</div></li></ul>
</section>
<section><div class="schedule-header mb-3"><h2>Upcoming Daily Races</h2>
  <img src="/assets/clients/race_control/rc_logo_symbol.png" /></div>
<div class="scheduled-race-card"><h4>Caterham Cup</h4></div>
</section>"##;

    fn utc(y: i64, mo: u32, d: u32, h: i64, mi: i64) -> i64 {
        (days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60) * 1000
    }

    #[test]
    fn parses_lmu_section_only() {
        let now = utc(2026, 10, 13, 20, 40);
        let races = parse(CARD, now).unwrap();
        assert_eq!(
            races,
            vec![DailyRace {
                name: "ELMS Sprint & Trophy".into(),
                tier: "intermediate".into(),
                track: "Spa Francorchamps".into(),
                duration: "44 minutes".into(),
                image: Some("https://example.com/spa.webp".into()),
                starts: vec![utc(2026, 10, 13, 20, 50), utc(2026, 10, 13, 0, 5)],
            }]
        );
    }

    #[test]
    fn missing_section_is_none() {
        assert!(parse("<html></html>", 0).is_none());
    }

    #[test]
    fn year_rolls_over() {
        let now = utc(2026, 12, 31, 23, 50);
        assert_eq!(parse_start("1 Jan at 12:15am", now), Some(utc(2027, 1, 1, 0, 15)));
        assert_eq!(parse_start("31 Dec at 12:00pm", now), Some(utc(2026, 12, 31, 12, 0)));
        assert_eq!(parse_start("garbage", now), None);
    }

    #[test]
    fn civil_round_trip() {
        for days in [-1000, 0, 19_000, 20_739] {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days);
        }
    }
}
