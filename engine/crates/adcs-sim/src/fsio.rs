//! Files the engine writes are whole or absent, and a failure is recorded where a
//! person can find it.
//!
//! `write` puts the bytes in a temporary file beside the target and renames it into
//! place, so a crash or a full disk leaves either the old file or the new one, never a
//! half-written result that a report would read as if it were complete.
//!
//! `install_crash_report` turns an internal error (a panic) into one line on standard
//! error and a crash report in the log folder: `$TRINETRA_LOG`, else `.trinetra/log`
//! in the home folder. The report says what failed and where, and nothing about the
//! case beyond the command that was run.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// Write `bytes` to `path` whole: a temporary file in the same folder, flushed, then
/// renamed over the target. The folder is created if it does not exist.
pub fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<(), crate::Error> {
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(|e| crate::Error::io(dir, e))?;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = dir.join(format!(".{name}.{}.{}.tmp", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
    let r = (|| -> std::io::Result<()> {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes.as_ref())?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if let Err(e) = r {
        let _ = std::fs::remove_file(&tmp);
        return Err(crate::Error::io(path, e));
    }
    Ok(())
}

/// The folder crash reports go to: `$TRINETRA_LOG` (when set and not empty), else
/// `~/.trinetra/log`, else the temporary folder.
pub fn log_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("TRINETRA_LOG").filter(|d| !d.is_empty()) { return d.into(); }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).filter(|h| !h.is_empty());
    match home { Some(h) => PathBuf::from(h).join(".trinetra").join("log"), None => std::env::temp_dir().join("trinetra-log") }
}

/// Seconds since 1970 and the UTC date and time they name, from the system clock alone.
pub fn utc_now() -> (u64, String) {
    let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (days, rem) = (s / 86400, s % 86400);
    // civil date from days since 1970-01-01 (Hinnant's algorithm)
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era*146097;
    let yoe = (doe - doe/1460 + doe/36524 - doe/146096)/365;
    let doy = doe - (365*yoe + yoe/4 - yoe/100);
    let mp = (5*doy + 2)/153;
    let d = doy - (153*mp + 2)/5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era*400 + if m <= 2 { 1 } else { 0 };
    (s, format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem/3600, rem/60 % 60, rem % 60))
}

/// Replace the default panic output with a crash report and one line saying where it is.
/// `program` names what crashed (`adcs`, `trinetra-app`).
pub fn install_crash_report(program: &'static str) {
    std::panic::set_hook(Box::new(move |info| {
        let what = info.payload().downcast_ref::<&str>().map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "an internal error".into());
        let at = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let (secs, when) = utc_now();
        let args: Vec<String> = std::env::args().collect();
        let body = format!("{program} crash report\nwhen     {when}\nversion  {}\nwhat     {what}\nwhere    {at}\ncommand  {}\nplatform {} {}\n\n{}\n",
            crate::ENGINE, args.join(" "), std::env::consts::OS, std::env::consts::ARCH, std::backtrace::Backtrace::force_capture());
        let file = log_dir().join(format!("crash-{program}-{secs}-{}.txt", std::process::id()));
        let saved = write(&file, body).is_ok();
        eprintln!("{program}: internal error: {what} ({at})");
        if saved { eprintln!("{program}: a crash report is in {}; send it to whoever looks after the tool", file.display()); }
    }));
}

#[cfg(test)]
mod t {
    #[test]
    fn a_write_is_whole_and_leaves_no_temporary_file() {
        let d = std::env::temp_dir().join(format!("adcs-fsio-{}", std::process::id()));
        let f = d.join("sub").join("x.json");
        super::write(&f, "one").unwrap();
        super::write(&f, "two").unwrap();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "two");
        let left: Vec<_> = std::fs::read_dir(f.parent().unwrap()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(left.len(), 1, "{left:?}");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn a_panic_leaves_a_crash_report() {
        let d = std::env::temp_dir().join(format!("adcs-crash-{}", std::process::id()));
        std::env::set_var("TRINETRA_LOG", &d);
        super::install_crash_report("adcs-test");
        let r = std::thread::spawn(|| panic!("the plant diverged")).join();
        let _ = std::panic::take_hook();
        assert!(r.is_err());
        // the hook is process-wide while it is installed, so another test's panic can leave a report
        // here too: find this test's own report by what it says
        let reports: Vec<String> = std::fs::read_dir(&d).unwrap().map(|e| std::fs::read_to_string(e.unwrap().path()).unwrap()).collect();
        let mine: Vec<&String> = reports.iter().filter(|b| b.contains("what     the plant diverged")).collect();
        assert_eq!(mine.len(), 1, "{reports:?}");
        assert!(mine[0].contains("fsio.rs") && mine[0].contains("adcs-test crash report"), "{}", mine[0]);
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn the_date_is_civil_utc() {
        let (s, w) = super::utc_now();
        assert!(s > 1_700_000_000);
        assert_eq!(w.len(), 20);
        assert!(w.starts_with("20") && w.ends_with('Z'), "{w}");
    }
}
