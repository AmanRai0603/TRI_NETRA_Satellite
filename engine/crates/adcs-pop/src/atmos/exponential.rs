//! `+atmos/exponential.m`: the Vallado piecewise-exponential density (altitude only,
//! no space weather), with its coarse temperature and mean molar mass.

/// Vallado table: base altitude [km], base density [kg/m^3], scale height [km].
const TABLE: [[f64; 3]; 28] = [
    [0.0, 1.225, 7.249],
    [25.0, 3.899e-2, 6.349],
    [30.0, 1.774e-2, 6.682],
    [40.0, 3.972e-3, 7.554],
    [50.0, 1.057e-3, 8.382],
    [60.0, 3.206e-4, 7.714],
    [70.0, 8.770e-5, 6.549],
    [80.0, 1.905e-5, 5.799],
    [90.0, 3.396e-6, 5.382],
    [100.0, 5.297e-7, 5.877],
    [110.0, 9.661e-8, 7.263],
    [120.0, 2.438e-8, 9.473],
    [130.0, 8.484e-9, 12.636],
    [140.0, 3.845e-9, 16.149],
    [150.0, 2.070e-9, 22.523],
    [180.0, 5.464e-10, 29.740],
    [200.0, 2.789e-10, 37.105],
    [250.0, 7.248e-11, 45.546],
    [300.0, 2.418e-11, 53.628],
    [350.0, 9.518e-12, 53.298],
    [400.0, 3.725e-12, 58.515],
    [450.0, 1.585e-12, 60.828],
    [500.0, 6.967e-13, 63.822],
    [600.0, 1.454e-13, 71.835],
    [700.0, 3.614e-14, 88.667],
    [800.0, 1.170e-14, 124.64],
    [900.0, 5.245e-15, 181.05],
    [1000.0, 3.019e-15, 268.00],
];

/// Avogadro constant [1/mol] (`de440.constants().N_A`).
pub const N_A: f64 = 6.02214076e23;

/// Result of [`exponential`]: `rho` [kg/m^3], `t` [K], `mmol` [kg/kmol], `n_o` [m^-3]
/// (total number density treated as atomic O for the GSI models).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExpAtm {
    /// density [kg/m^3]
    pub rho: f64,
    /// temperature [K]
    pub t: f64,
    /// mean molar mass [kg/kmol]
    pub mmol: f64,
    /// number density treated as atomic O [m^-3]
    pub n_o: f64,
}

/// `meanMolarMass(h)`: crude N2/O2 -> O -> H transition [kg/kmol].
pub fn mean_molar_mass(h: f64) -> f64 {
    let mut m;
    if h < 150.0 {
        m = 29.0 - (29.0 - 22.0) * (h - 100.0) / 50.0;
        m = m.max(22.0);
    } else if h < 500.0 {
        m = 22.0 - (22.0 - 16.0) * (h - 150.0) / 350.0;
    } else {
        m = 16.0 - (16.0 - 4.0) * ((h - 500.0) / 500.0).min(1.0);
    }
    m.max(4.0)
}

/// `atmos.exponential(alt_km)`.
pub fn exponential(alt_km: f64) -> ExpAtm {
    let mut h = alt_km;
    if h < 0.0 {
        h = 0.0;
    }
    let mut idx = 0usize;
    for (i, row) in TABLE.iter().enumerate() {
        if row[0] <= h {
            idx = i;
        }
    }
    let [h0, rho0, hs] = TABLE[idx];
    let rho = rho0 * (-(h - h0) / hs).exp();
    let t = 1000.0 - (1000.0 - 186.0) * (-(h.max(90.0) - 90.0) / 70.0).exp();
    let mmol = mean_molar_mass(h);
    let na = N_A * 1000.0;
    ExpAtm { rho, t, mmol, n_o: rho * na / mmol }
}
