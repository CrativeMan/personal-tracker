//! Dienstplan PDF → CSV / ICS converter logic.
//! Call [`convert`] from the UI; everything else is internal.

use chrono::{NaiveDate, NaiveTime};
use regex::Regex;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;

// ─── Public API ──────────────────────────────────────────────────────────────

pub struct ConvertOptions {
    pub pdf_path: PathBuf,
    /// Directory where output files are written. Defaults to PDF's parent dir.
    pub output_dir: Option<PathBuf>,
    pub write_csv: bool,
    pub write_ics: bool,
    /// Prefix used in ICS event summaries, e.g. "Dienst"
    pub event_prefix: String,
}

pub struct ConvertResult {
    pub csv_path: Option<PathBuf>,
    pub ics_path: Option<PathBuf>,
    pub mitarbeiter: String,
    pub shift_count: usize,
}

/// Top-level entry point called by the UI button.
pub fn convert(opts: ConvertOptions) -> io::Result<ConvertResult> {
    let text = extract_pdf_text(&opts.pdf_path)?;
    let (meta, entries) = parse_dienstplan(&text);

    let out_dir = opts.output_dir.unwrap_or_else(|| {
        opts.pdf_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });
    fs::create_dir_all(&out_dir)?;

    let base = build_base_name(&meta);
    let mut result = ConvertResult {
        csv_path: None,
        ics_path: None,
        mitarbeiter: meta.mitarbeiter.clone(),
        shift_count: entries.iter().filter(|e| !e.is_off).count(),
    };

    if opts.write_csv {
        let path = out_dir.join(format!("{base}.csv"));
        write_csv(&path, &meta, &entries, false)?;
        result.csv_path = Some(path);
    }
    if opts.write_ics {
        let path = out_dir.join(format!("{base}.ics"));
        write_ics(&path, &meta, &entries, &opts.event_prefix)?;
        result.ics_path = Some(path);
    }

    Ok(result)
}

// ─── Internal types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ShiftEntry {
    weekday: String,
    date: NaiveDate,
    kuerzel: String,
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
    arbeit: Option<f64>,
    umkle: Option<f64>,
    bemerkung: String,
    station: String,
    is_off: bool,
}

#[derive(Debug, Default)]
struct DienstplanMeta {
    mitarbeiter: String,
    mandant: String,
    station: String,
    zeitraum_start: Option<NaiveDate>,
    zeitraum_end: Option<NaiveDate>,
    ist_arbeitszeit: Option<f64>,
    soll_arbeitszeit: Option<f64>,
    saldo_monat: Option<f64>,
    urlaubsanspruch: Option<u32>,
    resturlaub: Option<u32>,
}

// ─── PDF extraction ───────────────────────────────────────────────────────────

fn extract_pdf_text(path: &Path) -> io::Result<String> {
    let output = Command::new("pdftotext")
        .args(["-layout", path.to_str().unwrap_or(""), "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "pdftotext not found. Install poppler-utils:\n\
                 • Linux:   sudo apt install poppler-utils\n\
                 • macOS:   brew install poppler\n\
                 • Windows: choco install poppler",
            )
        })?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "pdftotext exited with status {}",
            output.status
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

// ─── Parsing ─────────────────────────────────────────────────────────────────

fn parse_dienstplan(text: &str) -> (DienstplanMeta, Vec<ShiftEntry>) {
    let mut meta = DienstplanMeta::default();
    let mut entries: Vec<ShiftEntry> = Vec::new();

    let re_mitarbeiter = Regex::new(r"Mitarbeiter:\s+([\w,\s]+?)\s{2,}").unwrap();
    let re_mandant = Regex::new(r"Mandant:\s+(.+)").unwrap();
    let re_station_header = Regex::new(r"Station/Bereich:\s+(\S+(?:\s+\S+)?)").unwrap();
    let re_zeitraum =
        Regex::new(r"Zeitraum:\s+(\d{2}\.\d{2}\.\d{4})\s*-\s*(\d{2}\.\d{2}\.\d{4})").unwrap();
    let re_shift = Regex::new(
        r"^\s*(Mo|Di|Mi|Do|Fr|Sa|So)\s+(\d{2}\.\d{2}\.\d{4})\s+(\S+)\s+(\d{2}:\d{2}-\d{2}:\d{2})\s+([\d,]+)\s*([\d,]*)\s*(.*?)\s*$",
    )
    .unwrap();
    let re_off = Regex::new(r"^\s*(Mo|Di|Mi|Do|Fr|Sa|So)\s+(\d{2}\.\d{2}\.\d{4})\s+x\s*$").unwrap();
    let re_ist = Regex::new(r"IST-Arbeitszeit\s+([\d,.-]+)\s+Std").unwrap();
    let re_soll = Regex::new(r"Vertr\. Sollarbeitszeit\s+([\d,.-]+)\s+Std").unwrap();
    let re_saldo = Regex::new(r"Saldo lfd\. Monat\s+([\d,.-]+)\s+Std").unwrap();
    let re_urlaub = Regex::new(r"Urlaubsanspruch lfd\. Jahr\s+(\d+)\s+Tag").unwrap();
    let re_resturlaub = Regex::new(r"Resturlaub zu Jahresende\s+(\d+)\s+Tag").unwrap();

    for line in text.lines() {
        if let Some(c) = re_mitarbeiter.captures(line) {
            meta.mitarbeiter = c[1].trim().to_string();
        }
        if let Some(c) = re_mandant.captures(line) {
            meta.mandant = c[1].trim().to_string();
        }
        if let Some(c) = re_station_header.captures(line)
            && meta.station.is_empty()
        {
            meta.station = c[1].trim().to_string();
        }
        if let Some(c) = re_zeitraum.captures(line) {
            meta.zeitraum_start = parse_date_de(&c[1]);
            meta.zeitraum_end = parse_date_de(&c[2]);
        }
        if let Some(c) = re_ist.captures(line) {
            meta.ist_arbeitszeit = parse_decimal_de(&c[1]);
        }
        if let Some(c) = re_soll.captures(line) {
            meta.soll_arbeitszeit = parse_decimal_de(&c[1]);
        }
        if let Some(c) = re_saldo.captures(line) {
            meta.saldo_monat = parse_decimal_de(&c[1]);
        }
        if let Some(c) = re_urlaub.captures(line) {
            meta.urlaubsanspruch = c[1].trim().parse().ok();
        }
        if let Some(c) = re_resturlaub.captures(line) {
            meta.resturlaub = c[1].trim().parse().ok();
        }

        if let Some(c) = re_off.captures(line)
            && let Some(date) = parse_date_de(&c[2])
        {
            entries.push(ShiftEntry {
                weekday: c[1].to_string(),
                date,
                kuerzel: "x".to_string(),
                start_time: None,
                end_time: None,
                arbeit: None,
                umkle: None,
                bemerkung: String::new(),
                station: String::new(),
                is_off: true,
            });
            continue;
        }

        if let Some(c) = re_shift.captures(line)
            && let Some(date) = parse_date_de(&c[2])
        {
            let (start, end) = parse_time_range(&c[4]);
            let trailing = c[7].trim().to_string();
            let (bemerkung, station) = split_bemerkung_station(&trailing, &meta.station);
            entries.push(ShiftEntry {
                weekday: c[1].to_string(),
                date,
                kuerzel: c[3].to_string(),
                start_time: start,
                end_time: end,
                arbeit: parse_decimal_de(&c[5]),
                umkle: if c[6].trim().is_empty() {
                    None
                } else {
                    parse_decimal_de(&c[6])
                },
                bemerkung,
                station,
                is_off: false,
            });
        }
    }

    (meta, entries)
}

fn split_bemerkung_station(trailing: &str, fallback: &str) -> (String, String) {
    let re = Regex::new(r"((?:RW|NEF|KTW|RTW|NAW|SEG)\s*\S+)\s*$").unwrap();
    if let Some(c) = re.captures(trailing) {
        let station = c[1].trim().to_string();
        let bemerkung = trailing[..c.get(0).unwrap().start()].trim().to_string();
        return (bemerkung, station);
    }
    (trailing.to_string(), fallback.to_string())
}

fn parse_date_de(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%d.%m.%Y").ok()
}

fn parse_decimal_de(s: &str) -> Option<f64> {
    s.trim().replace(',', ".").parse().ok()
}

fn parse_time_range(s: &str) -> (Option<NaiveTime>, Option<NaiveTime>) {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return (None, None);
    }
    (
        NaiveTime::parse_from_str(parts[0].trim(), "%H:%M").ok(),
        NaiveTime::parse_from_str(parts[1].trim(), "%H:%M").ok(),
    )
}

fn build_base_name(meta: &DienstplanMeta) -> String {
    let name = meta.mitarbeiter.replace(", ", "_").replace([' ', '/'], "_");
    if let Some(start) = meta.zeitraum_start {
        format!("{}_{}", name, start.format("%Y-%m"))
    } else {
        name
    }
}

// ─── CSV ─────────────────────────────────────────────────────────────────────

fn write_csv(
    path: &Path,
    meta: &DienstplanMeta,
    entries: &[ShiftEntry],
    skip_off: bool,
) -> io::Result<()> {
    let mut f = fs::File::create(path)?;
    writeln!(f, "# Mitarbeiter,{}", meta.mitarbeiter)?;
    writeln!(f, "# Mandant,{}", meta.mandant)?;
    writeln!(f, "# Station,{}", meta.station)?;
    if let (Some(s), Some(e)) = (meta.zeitraum_start, meta.zeitraum_end) {
        writeln!(
            f,
            "# Zeitraum,{},{}",
            s.format("%d.%m.%Y"),
            e.format("%d.%m.%Y")
        )?;
    }
    if let Some(h) = meta.ist_arbeitszeit {
        writeln!(f, "# IST-Arbeitszeit,{h:.2}")?;
    }
    if let Some(h) = meta.soll_arbeitszeit {
        writeln!(f, "# Sollarbeitszeit,{h:.2}")?;
    }
    if let Some(h) = meta.saldo_monat {
        writeln!(f, "# Saldo,{h:.2}")?;
    }
    if let Some(u) = meta.urlaubsanspruch {
        writeln!(f, "# Urlaubsanspruch,{u}")?;
    }
    if let Some(r) = meta.resturlaub {
        writeln!(f, "# Resturlaub_Jahresende,{r}")?;
    }

    writeln!(
        f,
        "Wochentag,Datum,Kuerzel,Dienstbeginn,Dienstende,Arbeit_Std,Umkleide_Std,Bemerkung,Station,Frei"
    )?;
    for e in entries {
        if skip_off && e.is_off {
            continue;
        }
        writeln!(
            f,
            "{},{},{},{},{},{},{},\"{}\",{},{}",
            e.weekday,
            e.date.format("%d.%m.%Y"),
            e.kuerzel,
            e.start_time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_default(),
            e.end_time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_default(),
            e.arbeit.map(|v| format!("{v:.2}")).unwrap_or_default(),
            e.umkle.map(|v| format!("{v:.2}")).unwrap_or_default(),
            e.bemerkung.replace('"', "\"\""),
            e.station,
            if e.is_off { "ja" } else { "nein" },
        )?;
    }
    Ok(())
}

// ─── ICS ─────────────────────────────────────────────────────────────────────

fn write_ics(
    path: &Path,
    meta: &DienstplanMeta,
    entries: &[ShiftEntry],
    prefix: &str,
) -> io::Result<()> {
    let mut f = fs::File::create(path)?;
    writeln!(f, "BEGIN:VCALENDAR")?;
    writeln!(f, "VERSION:2.0")?;
    writeln!(f, "PRODID:-//dienstplan_converter//BRK Dienstplan//DE")?;
    writeln!(f, "CALSCALE:GREGORIAN")?;
    writeln!(f, "METHOD:PUBLISH")?;
    writeln!(f, "X-WR-CALNAME:Dienstplan {}", meta.mitarbeiter)?;
    writeln!(f, "X-WR-TIMEZONE:Europe/Berlin")?;

    writeln!(f, "BEGIN:VTIMEZONE")?;
    writeln!(f, "TZID:Europe/Berlin")?;
    writeln!(f, "BEGIN:STANDARD")?;
    writeln!(f, "DTSTART:19701025T030000")?;
    writeln!(f, "TZOFFSETFROM:+0200")?;
    writeln!(f, "TZOFFSETTO:+0100")?;
    writeln!(f, "TZNAME:CET")?;
    writeln!(f, "RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10")?;
    writeln!(f, "END:STANDARD")?;
    writeln!(f, "BEGIN:DAYLIGHT")?;
    writeln!(f, "DTSTART:19700329T020000")?;
    writeln!(f, "TZOFFSETFROM:+0100")?;
    writeln!(f, "TZOFFSETTO:+0200")?;
    writeln!(f, "TZNAME:CEST")?;
    writeln!(f, "RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3")?;
    writeln!(f, "END:DAYLIGHT")?;
    writeln!(f, "END:VTIMEZONE")?;

    let dtstamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    for e in entries {
        if e.is_off {
            continue;
        }
        let (start, end) = match (e.start_time, e.end_time) {
            (Some(s), Some(en)) => (s, en),
            _ => continue,
        };
        let end_date = if end <= start {
            e.date.succ_opt().unwrap_or(e.date)
        } else {
            e.date
        };
        writeln!(f, "BEGIN:VEVENT")?;
        writeln!(f, "UID:{}", Uuid::new_v4())?;
        writeln!(f, "DTSTAMP:{dtstamp}")?;
        writeln!(
            f,
            "DTSTART;TZID=Europe/Berlin:{}T{}",
            e.date.format("%Y%m%d"),
            start.format("%H%M%S")
        )?;
        writeln!(
            f,
            "DTEND;TZID=Europe/Berlin:{}T{}",
            end_date.format("%Y%m%d"),
            end.format("%H%M%S")
        )?;
        writeln!(
            f,
            "SUMMARY:{}",
            ics_escape(&build_summary(prefix, &e.kuerzel, &e.bemerkung))
        )?;
        writeln!(f, "DESCRIPTION:{}", ics_escape(&build_description(e, meta)))?;
        writeln!(f, "LOCATION:{}", ics_escape(&e.station))?;
        writeln!(f, "CATEGORIES:Dienst")?;
        writeln!(f, "END:VEVENT")?;
    }

    writeln!(f, "END:VCALENDAR")?;
    Ok(())
}

fn build_summary(prefix: &str, kuerzel: &str, bemerkung: &str) -> String {
    if bemerkung.is_empty() {
        format!("{prefix}: {kuerzel}")
    } else {
        format!("{prefix}: {kuerzel} ({bemerkung})")
    }
}

fn build_description(e: &ShiftEntry, meta: &DienstplanMeta) -> String {
    let mut parts = vec![
        format!("Mitarbeiter: {}", meta.mitarbeiter),
        format!("Dienstart: {}", e.kuerzel),
        format!("Station: {}", e.station),
    ];
    if let Some(h) = e.arbeit {
        parts.push(format!("Arbeitszeit: {h:.2} Std"));
    }
    if let Some(u) = e.umkle {
        parts.push(format!("Umkleidezeit: {u:.2} Std"));
    }
    if !e.bemerkung.is_empty() {
        parts.push(format!("Bemerkung: {}", e.bemerkung));
    }
    parts.join("\\n")
}

fn ics_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
}
