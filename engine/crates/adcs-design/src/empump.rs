//! Electromagnetic design of the fluid momentum loop (our product): the galinstan loop and its
//! DC conduction (Faraday) pump with an ELECTROMAGNET, designed together per ring.
//!
//! Loop:     h = N rho A 2 S v            (N loops of bore d round a face of enclosed area S)
//! Duty:     cruise at 0.4 v_max (friction), and the ring's share of tau_req (acceleration)
//! Pressure: dp_f = f (L/d) rho v^2 / 2   (Darcy; 64/Re laminar, Blasius above Re 2300)
//!           dp_a = rho L dv/dt,  dv/dt = tau / (N rho A 2 S)
//! Pump:     the bore flattened to a b x a duct in the gap (b = d/2 along B, a = A/b along the
//!           current), active length Lp.  dp = B I / b  ->  I = dp b / B   (I <= 10 A: a practical low-voltage driver)
//!           fluid resistance R = rho_e a / (b Lp), bypass efficiency 0.7, driver 0.85:
//!           P_e = (I^2 R + dp Q) / (0.7 x 0.85)
//! Magnet:   C-core, gap g = b + 2 x 0.5 mm walls, NI = 1.3 B g / mu0 (iron + leakage)
//!           coil: P_c = rho_cu l_t (NI)^2 / A_cu, m_cu = d_cu l_t A_cu  ->  P_c m_cu = K
//!           (fixed by the geometry): for a mass/power exchange rate lambda [kg/W] the best
//!           copper is m_cu = sqrt(lambda K), P_c = sqrt(K / lambda)
//!           iron: flux B a Lp at 1.2 T, path 2(a + Lp) + 20 mm, 7650 kg/m^3
//! Design:   grid over v_max (0.1-2 m/s), d (2-6 mm), B (0.05-1 T), Lp (10-40 mm); keep the
//!           design with the least  mass + lambda x (steady power)  that meets h and tau.
//!
//! lambda is the knob of the design loop: a power failure raises it (more copper, less coil
//! power), a mass failure lowers it (less copper, faster flow, less fluid).
//! Anchor: IDMAS v2 §03C (1-3 W per pump unit); galinstan rho 6440 kg/m^3, mu 2.4 mPa s,
//! resistivity 2.89e-7 Ohm m. Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use serde_json::{json, Value};
use std::f64::consts::PI;

const RHO: f64 = 6440.0;
const MU: f64 = 0.0024;
const RHO_E: f64 = 2.89e-7;
const MU0: f64 = 4e-7*PI;
const RHO_CU: f64 = 1.72e-8;
const D_CU: f64 = 8960.0;
const D_FE: f64 = 7650.0;
const B_SAT: f64 = 1.2;
const I_MAX: f64 = 10.0;
const ETA_BYPASS: f64 = 0.7;
const ETA_DRV: f64 = 0.85;

#[derive(Clone, Debug, Default)]
pub struct Design {
    pub d: f64, pub loops: f64, pub v_max: f64, pub h_max: f64, pub s: f64, pub l: f64, pub re: f64,
    pub b: f64, pub lp: f64, pub gap: f64, pub ni: f64, pub i_design: f64, pub i_cruise: f64,
    pub dp_design: f64, pub dp_cruise: f64, pub tau_max: f64,
    pub p_coil: f64, p_elec_cruise: f64, pub p_steady: f64, pub p_peak: f64, pub eta_cruise: f64,
    pub m_fluid: f64, pub m_channel: f64, pub m_cu: f64, pub m_fe: f64, pub mass: f64, pub cost: f64,
}

fn friction(v: f64, d: f64, l: f64) -> (f64, f64) {
    let re = (RHO*v*d/MU).max(1e-9);
    let f = if re < 2300.0 { 64.0/re } else { 0.316*re.powf(-0.25) };
    (f*(l/d)*RHO*v*v/2.0, re)
}

/// Best design for one ring: momentum h, torque tau, face (enclosed area s0 = 0.8 face, loop length l1).
pub fn design(h: f64, tau: f64, s0: f64, l1: f64, lambda: f64) -> Option<Design> {
    let mut best: Option<Design> = None;
    for iv in 0..20 {
        let v_max = 0.1 + 0.1*iv as f64;
        for id in 0..9 {
            let d = 0.002 + 0.0005*id as f64;
            let a_c = PI*d*d/4.0;
            let n = (h/(RHO*a_c*2.0*s0*v_max)).ceil().max(1.0);
            let l = n*l1;
            let vc = 0.4*v_max;
            let (dpf_max, _) = friction(v_max, d, l);
            let (dpf_c, re) = friction(vc, d, l);
            let dvdt = tau/(n*RHO*a_c*2.0*s0);
            let dp_design = dpf_max + RHO*l*dvdt;
            let b_duct = d/2.0;
            let a_duct = a_c/b_duct;
            let gap = b_duct + 1e-3;
            let m_fluid = RHO*a_c*l;
            let m_channel = 0.03 + 0.012*(n - 1.0);
            for ib in 0..20 {
                let bf = 0.05*(ib + 1) as f64;
                for il in 0..4 {
                    let lp = 0.010*(il + 1) as f64;
                    let i_d = dp_design*b_duct/bf;
                    if i_d > I_MAX { continue; }
                    let r = RHO_E*a_duct/(b_duct*lp);
                    let i_c = dpf_c*b_duct/bf;
                    let q_c = a_c*vc;
                    let p_ec = (i_c*i_c*r + dpf_c*q_c)/(ETA_BYPASS*ETA_DRV);
                    let p_ed = (i_d*i_d*r + dp_design*a_c*v_max)/(ETA_BYPASS*ETA_DRV);
                    let ni = 1.3*bf*gap/MU0;
                    let a_fe = bf*a_duct*lp/B_SAT;
                    let l_t = 4.0*a_fe.sqrt() + 0.01;
                    let k = RHO_CU*D_CU*l_t*l_t*ni*ni;
                    let m_cu = (lambda*k).sqrt().max(0.002);
                    let p_c = k/m_cu;
                    let m_fe = D_FE*a_fe*(2.0*(a_duct + lp) + 0.02);
                    let mass = m_fluid + m_channel + m_cu + m_fe + 0.010;
                    let p_steady = p_c + p_ec;
                    let cost = mass + lambda*p_steady;
                    if best.as_ref().map(|x| cost < x.cost).unwrap_or(true) {
                        // total pump torque capacity (holding against viscous spin-down + accelerating),
                        // as the engine's ring model clamps it: dp_max x 2 S A / L at the current limit
                        let tau_max = I_MAX*bf/b_duct*n*a_c*2.0*s0/l;
                        best = Some(Design { d, loops: n, v_max, h_max: n*RHO*a_c*2.0*s0*v_max, s: n*s0, l, re,
                            b: bf, lp, gap, ni, i_design: i_d, i_cruise: i_c, dp_design, dp_cruise: dpf_c, tau_max,
                            p_coil: p_c, p_elec_cruise: p_ec, p_steady, p_peak: p_c + p_ed,
                            eta_cruise: if p_ec > 0.0 { dpf_c*q_c/p_ec } else { 0.0 },
                            m_fluid, m_channel, m_cu, m_fe, mass, cost });
                    }
                }
            }
        }
    }
    best
}

impl Design {
    pub fn json(&self, lambda: f64) -> Value {
        json!({"bore_m": self.d, "loops": self.loops, "v_max_m_s": self.v_max, "h_max_Nms": self.h_max, "enclosed_area_m2": self.s,
            "channel_length_m": self.l, "reynolds_cruise": self.re, "v_cruise_m_s": 0.4*self.v_max,
            "pump": {"type": "DC conduction pump, electromagnet C-core", "B_gap_T": self.b, "active_length_m": self.lp, "gap_m": self.gap,
                     "ampere_turns": self.ni, "electrode_current_design_A": self.i_design, "electrode_current_cruise_A": self.i_cruise,
                     "dp_design_Pa": self.dp_design, "dp_cruise_Pa": self.dp_cruise, "coil_power_W": self.p_coil,
                     "electrode_power_cruise_W": self.p_elec_cruise, "copper_kg": self.m_cu, "iron_kg": self.m_fe, "lambda_kg_per_W": lambda,
                     "efficiency_cruise": self.eta_cruise},
            "pump_torque_max_Nm": self.tau_max, "field_power_W": self.p_coil, "power_steady_W": self.p_steady, "power_peak_W": self.p_peak,
            "pump_efficiency": self.eta_cruise.max(0.01), "fluid_mass_kg": self.m_fluid, "mass_kg": self.mass})
    }
}

/// The mass / steady-power Pareto front of one ring over lambda (for the report).
pub fn pareto(h: f64, tau: f64, s0: f64, l1: f64) -> Vec<Value> {
    [0.003, 0.01, 0.03, 0.1, 0.3, 1.0, 3.0].iter().filter_map(|&lam| design(h, tau, s0, l1, lam)
        .map(|d| json!({"lambda": lam, "mass_kg": d.mass, "power_W": d.p_steady, "coil_W": d.p_coil, "v_max": d.v_max, "bore_m": d.d, "B_T": d.b, "loops": d.loops}))).collect()
}
