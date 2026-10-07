//! Neutral density for the fast orbit: env's method of l3_dist_row_07 (`gen::truthdensity`), over env's static
//! exponential atmosphere (m3_3's table, Vallado 2013 table 8-4, one copy), times an activity factor from the
//! scenario (1 = the table), generated from the design (tools/engine_build.py). The MATLAB twin uses DTM2020 through
//! the POP; at 550 km and F10.7 = 130 the table is ~3x denser, so a parity run sets density_scale (fsw/twin_map.toml).

/// Density [kg/m^3] at geodetic height h [m].
pub fn density(h_m: f64, scale: f64) -> f64 { crate::gen::truthdensity::density_scaled(h_m, scale) }
