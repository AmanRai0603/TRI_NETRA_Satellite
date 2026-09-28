// Compiles the reference flight software (fsw/, C99) with the flags of spec §9.6.
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fsw");
    let mut b = cc::Build::new();
    b.include(root.join("include")).std("c99").opt_level(2).flag("-ffp-contract=off").flag("-fno-fast-math").warnings(true);
    for f in ["adcs_math", "adcs_env", "adcs_est", "adcs_ctl", "adcs_alloc", "adcs_drv", "adcs_params", "adcs_fsw"] {
        let p = root.join("src").join(format!("{f}.c"));
        println!("cargo:rerun-if-changed={}", p.display());
        b.file(p);
    }
    println!("cargo:rerun-if-changed={}", root.join("include").display());
    b.compile("adcs_fsw_c");
}
