//! The `adcs` command as a user meets it: each command does what its help says, a refused input
//! exits 2 with the reason, a failure exits 1, and a run goes where it is told and comes back.
use std::path::PathBuf;
use std::process::{Command, Output};

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../matlab_sils") }

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("adcs-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn adcs(store: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_adcs")).args(args).env("ADCS_ROOT", root()).env("TRINETRA_STORE", store)
        .env("TRINETRA_LOG", store.join("log")).output().unwrap()
}

fn text(o: &Output) -> String { format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)) }

#[test]
fn a_refused_input_exits_2_with_its_reason() {
    let s = tmp("refused");
    for (args, why) in [(vec!["run", "no_such_scenario"], "no scenario"),
                        (vec!["run", "nadir_hold_ais", "--case", "no_such_case"], "no case"),
                        (vec!["run", "../x"], "scenario"),
                        (vec!["run", "nadir_hold_ais", "--set", "engine.no_such_knob=1"], "engine.no_such_knob"),
                        (vec!["run", "nadir_hold_ais", "--set", "engine.duration_s=-5"], "duration"),
                        (vec!["size", "no_such_case"], "no case"),
                        (vec!["results", "show", "/no/such/run"], "")] {
        let o = adcs(&s, &args);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", text(&o));
        assert!(text(&o).contains(why), "{args:?} names why: {}", text(&o));
    }
    for args in [vec!["run"], vec!["fly"], vec!["run", "nadir_hold_ais", "--fsw", "ada"], vec!["run", "nadir_hold_ais", "--seed", "x"]] {
        assert_eq!(adcs(&s, &args).status.code(), Some(2), "a malformed command line: {args:?}");
    }
    let h = adcs(&s, &["help"]);
    assert!(h.status.success() && text(&h).contains("results"), "help names the commands");
    let _ = std::fs::remove_dir_all(&s);
}

#[test]
fn a_run_is_stored_listed_shown_shared_and_flown_again() {
    let s = tmp("run");
    let out = s.join("results_engine/nadir_hold_ais");
    let o = adcs(&s, &["run", "nadir_hold_ais", "--quiet", "--set", "engine.duration_s=60"]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("done in") && out.join("manifest.json").is_file() && out.join("channels.csv").is_file(), "{}", text(&o));

    let l = adcs(&s, &["results", "list"]);
    assert!(l.status.success() && text(&l).contains("nadir_hold_ais"), "{}", text(&l));
    let sh = adcs(&s, &["results", "show", out.to_str().unwrap()]);
    assert!(sh.status.success() && text(&sh).contains("ais_3u"), "{}", text(&sh));
    let st = adcs(&s, &["results", "stale"]);
    assert_eq!(st.status.code(), Some(0), "a fresh run is not stale: {}", text(&st));
    let q = adcs(&s, &["results", "query", "--sql", "SELECT scenario FROM runs"]);
    assert!(q.status.success() && text(&q).contains("nadir_hold_ais"), "{}", text(&q));
    assert_eq!(adcs(&s, &["results", "query", "--sql", "DELETE FROM runs"]).status.code(), Some(2), "the index is read-only");

    let share = s.join("x.trinetra");
    assert!(adcs(&s, &["results", "export", out.to_str().unwrap(), "--out", share.to_str().unwrap()]).status.success());
    let back = s.join("back");
    assert!(adcs(&s, &["results", "import", share.to_str().unwrap(), "--out", back.to_str().unwrap()]).status.success());
    assert_eq!(std::fs::read(out.join("channels.csv")).unwrap(), std::fs::read(back.join("channels.csv")).unwrap());

    let again = s.join("again");
    let r = adcs(&s, &["results", "refly", out.to_str().unwrap(), "--out", again.to_str().unwrap()]);
    assert!(r.status.success(), "{}", text(&r));
    assert_eq!(std::fs::read(out.join("channels.csv")).unwrap(), std::fs::read(again.join("channels.csv")).unwrap(), "the same inputs fly the same run");
    let _ = std::fs::remove_dir_all(&s);
}

#[test]
fn params_parity_and_size_do_what_they_say() {
    let s = tmp("other");
    let blob = s.join("p.bin");
    std::fs::create_dir_all(&s).unwrap();
    let p = adcs(&s, &["params", "nadir_hold_ais", "--out", blob.to_str().unwrap()]);
    assert!(p.status.success(), "{}", text(&p));
    assert_eq!(std::fs::metadata(&blob).unwrap().len() as usize, adcs_fsw::params::BLOB_SIZE);
    assert_eq!(adcs_fsw::Params::decode(&std::fs::read(&blob).unwrap()).is_ok(), true, "the blob is one the flight software accepts");

    let par = adcs(&s, &["parity", "nadir_hold_ais", "--set", "engine.duration_s=60"]);
    assert!(par.status.success() && text(&par).contains("bit-identical"), "{}", text(&par));

    let out = s.join("sized");
    let z = adcs(&s, &["size", "ais_3u", "--out", out.to_str().unwrap()]);
    assert!(z.status.success() && text(&z).contains("tau_dist"), "{}", text(&z));
    assert!(std::fs::read_dir(&out).unwrap().count() > 0, "the sizing is written");
    let _ = std::fs::remove_dir_all(&s);
}

#[test]
fn a_soft_oils_run_is_judged_on_its_deadline() {
    let s = tmp("oils");
    let verdicts = |lat: &str| -> serde_json::Value {
        let out = s.join(format!("lat{lat}"));
        let o = adcs(&s, &["run", "nadir_hold_ais", "--quiet", "--set", "engine.duration_s=60", "--latency-ms", lat, "--out", out.to_str().unwrap()]);
        assert!(o.status.success(), "{}", text(&o));
        serde_json::from_str(&std::fs::read_to_string(out.join("manifest.json")).unwrap()).unwrap()
    };
    let pass = |m: &serde_json::Value, id: &str| m["metrics"].as_array().unwrap().iter().find(|x| x["id"] == id).map(|x| x["pass"].clone());
    // 200 ms period, deadline at half of it: 5 ms lands in time, 150 ms misses the deadline but not the period
    let ok = verdicts("5");
    assert_eq!((pass(&ok, "oils_overruns"), pass(&ok, "oils_worst_case_margin")), (Some(1.into()), Some(1.into())));
    let late = verdicts("150");
    assert_eq!((pass(&late, "oils_overruns"), pass(&late, "oils_worst_case_margin")), (Some(1.into()), Some(0.into())));
    assert!((late["oils"]["deadline_s"].as_f64().unwrap() - 0.1).abs() < 1e-12);
    // a whole period late is an overrun
    assert_eq!(pass(&verdicts("250"), "oils_overruns"), Some(0.into()));
    // a SILS run carries no timing verdicts
    let sils = s.join("sils");
    assert!(adcs(&s, &["run", "nadir_hold_ais", "--quiet", "--set", "engine.duration_s=60", "--out", sils.to_str().unwrap()]).status.success());
    let m: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(sils.join("manifest.json")).unwrap()).unwrap();
    assert!(pass(&m, "oils_overruns").is_none());
    // a worst case below the nominal, or a deadline outside (0, 1], is refused
    let o = adcs(&s, &["run", "nadir_hold_ais", "--oils", "--cpi", "1.5", "--cpi-max", "1.2"]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("worst CPI"));
    for f in ["0", "1.5", "-0.2"] { assert_eq!(adcs(&s, &["run", "nadir_hold_ais", "--deadline-frac", f]).status.code(), Some(2), "{f}"); }
    let _ = std::fs::remove_dir_all(&s);
}
