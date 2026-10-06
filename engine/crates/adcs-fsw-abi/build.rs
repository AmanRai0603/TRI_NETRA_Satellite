// Compiles the reference flight software (fsw/, C99) with the flags of spec §9.6.
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fsw");
    let mut b = cc::Build::new();
    b.include(root.join("include")).include(root.join("alg").join("include")).std("c99").opt_level(2).flag("-ffp-contract=off").flag("-fno-fast-math").warnings(true);
    // the algorithms, written from the design by tools/flight_build.py
    let mut alg: Vec<_> = std::fs::read_dir(root.join("alg").join("src")).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "c")).collect()).unwrap_or_default();
    alg.sort();
    for p in &alg {
        println!("cargo:rerun-if-changed={}", p.display());
        b.file(p);
    }
    b.flag_if_supported("-Wno-unused-variable").flag_if_supported("-Wno-unused-but-set-variable").flag_if_supported("-Wno-unused-parameter");
    println!("cargo:rerun-if-changed={}", root.join("alg").join("include").display());
    for f in ["adcs_math", "adcs_env", "adcs_est", "adcs_guid", "adcs_ctl", "adcs_alloc", "adcs_drv", "adcs_params", "adcs_modes", "adcs_fdir", "adcs_fsw"] {
        let p = root.join("src").join(format!("{f}.c"));
        println!("cargo:rerun-if-changed={}", p.display());
        b.file(p);
    }
    println!("cargo:rerun-if-changed={}", root.join("include").display());
    println!("cargo:rerun-if-changed={}", root.join("src").join("adcs_fsw_int.h").display());
    println!("cargo:rerun-if-changed={}", root.join("src").join("adcs_alg_glue.h").display());
    b.compile("adcs_fsw_c");
}
