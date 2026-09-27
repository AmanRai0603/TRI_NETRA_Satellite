#!/usr/bin/env python3
"""Builds plan/tree.json, the ADCS tree, in the exact seven-key shape that
VLEO_SIMULATOR's cd06/tree.json uses, so the same seeder consumes it.

Why a script and not a hand-written JSON file: the tree is reviewed as a
decomposition, and a decomposition is easier to read as nested Python than as
positional tuples. The JSON is the artefact the seeder reads; this file is the
thing a person edits. Re-run it after any edit:

    python3 tools/build_tree.py            # writes plan/tree.json, plan/case_inputs.toml
                                           # and plan/case_template.csv
    python3 tools/build_tree.py --check    # compares all three without writing
    python3 tools/validate_plan.py         # checks them

Shape rules inherited from the VLEO seeder (tools/cd06_rows.py there):

* HN_MGT is rooted at "mgm", HN_SYS at "prg". Both ids are hard-coded in the
  seeder's install().
* A row with children is a group; a row without is a leaf.
* A leaf's note is its kind: "set here" -> declared, "computed" -> computed,
  "target — required" -> required, "achieved — what the design delivers" ->
  achieved. Any other note on a leaf becomes declared.
* A management leaf whose note contains "door into" crosses into the system
  root. There is exactly one: every case goes through it. No customer is a
  branch of this tree; a customer is a case, uploaded as one CSV
  (plan/case_template.csv) and held as data (SPEC.md §8).
* The 5th field ("extra") is ignored by the seeder. Here it carries the
  configuration family a row is in play for: "" means every family,
  otherwise a comma-separated subset of mtq, rw, fmr, rcs. The ADCS build
  turns it into each group's `cases` filter (SPEC.md §5.6).
* layer3_shape[i].targets must equal the leaf count of the layer-2 group the
  subsystem is mapped to, or the seeder aborts. This script computes it, so it
  cannot drift.
"""

import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

SET = "set here"
COMP = "computed"
REQ = "target — required"
ACH = "achieved — what the design delivers"
DOOR = "the door into this case’s engineering layer"

# --------------------------------------------------------------------------
# Layer 1 — management. The company that sells the ADCS and runs the facility.
# Each leaf: (key, label, note, extra). Keys are local names used to wire
# edges below; ids are assigned positionally as <group>_<n>.
# --------------------------------------------------------------------------

MGT = ("mgm", "ADCS products and test facility", "the company — every case, every order, one architecture", [
    ("cas", "Case intake", "every enquiry is one case: one CSV in the fixed format, never a branch of this tree", [
        ("ci1", "Case", "the rows every case has, whoever sent it", [
            ("i_fmt", "Case input format version", SET, ""),
            ("i_sol", "Satellite ADCS", DOOR, ""),
            ("i_prod", "Catalogue product selected", SET, ""),
            ("i_cst", "Cost & price", SET, ""),
            ("i_acc", "Acceptance criteria", SET, ""),
        ]),
    ]),
    ("cat", "Catalogue", "what the company has designed, proven in SILS and offers", [
        ("ct1", "Products", "a product is a configuration plus its algorithm, designed for a satellite class", [
            ("k_off", "Products offered", SET, ""),
            ("k_cand", "Designed products awaiting promotion", SET, ""),
            ("k_cls", "Satellite classes defined", SET, ""),
            ("k_cov", "Satellite classes with an offered product", SET, ""),
            ("k_frac", "Class coverage", COMP, ""),
        ]),
        ("ct2", "Design runs", "the designer's sweeps over part combinations", [
            ("k_runs", "Combinations evaluated", SET, ""),
            ("k_pass", "Combinations that met their class requirements", SET, ""),
            ("k_yield", "Design yield", COMP, ""),
        ]),
        ("ct3", "Case matching", "how often a case finds an offered product without new design", [
            ("k_cases", "Cases solved", SET, ""),
            ("k_hit", "Cases matched by an offered product", SET, ""),
            ("k_hitr", "Catalogue hit rate", COMP, ""),
        ]),
    ]),
    ("cmr", "Commercial", "what an order is worth, and what it costs to deliver", [
        ("cm1", "Quote", "frozen by the case hash; any change is a new quote", [
            ("q_unit", "Unit price", COMP, ""),
            ("q_oils", "OILS campaign price", COMP, ""),
            ("q_hils", "HILS campaign price", COMP, ""),
            ("q_valid", "Quote validity", SET, ""),
            ("q_total", "Quote total", COMP, ""),
        ]),
        ("cm2", "Cost to deliver", "what the unit and its evidence cost us", [
            ("c_bom", "Bill of materials cost", SET, ""),
            ("c_build", "Build labour", SET, ""),
            ("c_cal", "Calibration labour", SET, ""),
            ("c_day", "Facility cost per day", SET, ""),
            ("c_unit", "Cost to deliver one unit", COMP, ""),
        ]),
        ("cm3", "Margin and terms", "how the price is set from the cost", [
            ("t_margin", "Target gross margin", SET, ""),
            ("t_ach", "Achieved gross margin", COMP, ""),
            ("t_disc", "Data-share discount", SET, ""),
            ("t_mile", "Payment milestones", SET, ""),
        ]),
    ]),
    ("ord", "Order lifecycle", "one path for every order, enquiry to flight data", [
        ("od1", "Stage durations", "each stage ends on a gate, not on a date", [
            ("d_short", "Enquiry to shortlist", SET, ""),
            ("d_quote", "Shortlist to quote", SET, ""),
            ("d_po", "Quote to purchase order", SET, ""),
            ("d_build", "Purchase order to unit built", SET, ""),
            ("d_cal", "Unit built to calibrated", SET, ""),
            ("d_oils", "Calibrated to OILS passed", SET, ""),
            ("d_hils", "OILS passed to HILS passed", SET, ""),
            ("d_cert", "HILS passed to certificate", SET, ""),
            ("d_ship", "Certificate to delivery", SET, ""),
            ("d_lead", "Order lead time", COMP, ""),
        ]),
        ("od2", "Configuration control", "what may change after the order, and at what price", [
            ("x_ecp", "Engineering changes after order", SET, ""),
            ("x_dev", "Deviations raised", SET, ""),
            ("x_waiv", "Waivers granted", SET, ""),
        ]),
    ]),
    ("fac", "Test facility", "the rig time every order is booked against", [
        ("fa1", "SILS capacity", "server compute for client and internal runs", [
            ("f_cores", "Server cores for SILS", SET, ""),
            ("f_mcph", "Monte Carlo runs per hour", COMP, ""),
            ("f_quota", "Client run quota per project", SET, ""),
        ]),
        ("fa2", "OILS rig", "flight OBC in the loop against the real-time plant: the rig host, the interface emulation unit, the flight connectors", [
            ("f_ohours", "OILS rig hours per week", SET, ""),
            ("f_step", "Real-time plant step", SET, ""),
            ("f_miss", "Deadline misses allowed per hour", SET, ""),
            ("f_lat", "Rig loop latency, worst case measured", SET, ""),
            ("f_jit", "Rig loop jitter measured", SET, ""),
            ("f_ieu", "Interface emulation channels", SET, ""),
            ("f_proto", "Flight interface kinds emulated", SET, ""),
            ("f_iso", "Flight-connector isolation voltage", SET, ""),
            ("f_gnsse", "GNSS receiver outputs emulated", SET, ""),
            ("f_link", "Rig host to IEU link rate", SET, ""),
        ]),
        ("fa3", "Magnetic field simulator", "the Helmholtz cage: the orbit's field, made in the lab around the unit", [
            ("f_bmax", "Helmholtz cage field range", SET, ""),
            ("f_bunif", "Cage field uniformity", SET, ""),
            ("f_bvol", "Uniform-field test volume", SET, ""),
            ("f_bacc", "Cage field error against command", SET, ""),
            ("f_bdir", "Cage field direction error", SET, ""),
            ("f_btau", "Cage coil time constant", SET, ""),
            ("f_bref", "Reference magnetometer noise", SET, ""),
        ]),
        ("fa4", "Facility use", "booked against capacity", [
            ("f_days", "Campaign days booked", COMP, ""),
            ("f_util", "Facility utilisation", COMP, ""),
        ]),
        ("fa5", "Air-bearing platform", "real rotational dynamics, with the torques a bearing adds", [
            ("f_tres", "Air-bearing residual torque", SET, ""),
            ("f_tilt", "Bearing tilt range", SET, ""),
            ("f_pin", "Platform inertia", SET, ""),
            ("f_pcap", "Platform payload capacity", SET, ""),
            ("f_bal", "Balance residual centre-of-mass offset", SET, ""),
            ("f_drag", "Bearing aerodynamic drag torque", SET, ""),
            ("f_truth", "Truth attitude accuracy", SET, ""),
            ("f_truthr", "Truth attitude rate", SET, ""),
        ]),
        ("fa6", "Optical and RF stimulators", "the Sun, the stars and the GNSS sky, for the real sensors", [
            ("f_sun", "Sun simulator irradiance", SET, ""),
            ("f_suncol", "Sun simulator collimation half-angle", SET, ""),
            ("f_sununi", "Sun simulator spatial non-uniformity", SET, ""),
            ("f_sunstab", "Sun simulator temporal instability", SET, ""),
            ("f_star", "Star stimulator angular step", SET, ""),
            ("f_starmag", "Star stimulator faintest magnitude", SET, ""),
            ("f_starfr", "Star stimulator frame rate", SET, ""),
            ("f_starlat", "Star stimulator latency", SET, ""),
            ("f_gnss", "GNSS RF simulator channels", SET, ""),
        ]),
        ("fa7", "Actuator test stands", "each actuator family measured on its own stand before it closes a loop", [
            ("f_dynbw", "Force dynamometer bandwidth", SET, ""),
            ("f_imb", "Wheel imbalance resolution", SET, ""),
            ("f_dipres", "Coil dipole measurement resolution", SET, ""),
            ("f_ringt", "Ring torque measurement resolution", SET, ""),
            ("f_thrres", "Thrust stand resolution", SET, ""),
            ("f_ibit", "Impulse bit resolution", SET, ""),
            ("f_dyntq", "Dynamometer torque resolution", SET, ""),
        ]),
        ("fa8", "Rig safety and power", "what stops a run in hardware, and the supplies the unit runs from", [
            ("f_ilock", "Hardware interlocks", SET, ""),
            ("f_estop", "Emergency stop response time", SET, ""),
            ("f_brown", "Brownout injection range", SET, ""),
            ("f_psu", "Programmable supply channels", SET, ""),
        ]),
    ]),
    ("std", "Standards & compliance", "every standard a certificate is issued against", [
        ("st1", "AOCS requirements, ECSS-E-ST-60-30C", "the structure every requirement package follows", [
            ("s_cov", "Requirement clauses covered", SET, ""),
            ("s_tail", "Tailoring items", SET, ""),
        ]),
        ("st2", "Control performance, ECSS-E-ST-60-10C", "how APE, RPE and AKE are stated and budgeted", [
            ("s_conf", "Statistical confidence of the budget", SET, ""),
            ("s_idx", "Performance indices budgeted", SET, ""),
        ]),
        ("st3", "Verification, ECSS-E-ST-10-02C", "test, analysis, review of design, inspection", [
            ("s_meth", "Requirements with a verification method", SET, ""),
            ("s_close", "Verification close-out rate", SET, ""),
        ]),
        ("st4", "Testing, ECSS-E-ST-10-03C", "test levels and the conditions they are run at", [
            ("s_lvl", "Test levels applied", SET, ""),
        ]),
        ("st5", "Software, ECSS-E-ST-40C and ECSS-Q-ST-80C", "flight software process and product assurance", [
            ("s_crit", "Software criticality category", SET, ""),
            ("s_ucov", "Flight software unit-test coverage", SET, ""),
        ]),
        ("st6", "Export control", "checked at quote time, before a foreign client sees a price", [
            ("s_exp", "Export classification checks done", SET, ""),
        ]),
        ("st7", "Quality management", "non-conformances and their closure", [
            ("s_ncr", "Non-conformances open", SET, ""),
        ]),
    ]),
    ("sup", "Supply chain", "what has to be bought, qualified and delivered", [
        ("su1", "Long-lead items", "the parts that set the lead time", [
            ("u_rw", "Reaction wheel lead time", SET, ""),
            ("u_st", "Star tracker lead time", SET, ""),
            ("u_ga", "Gallium alloy stock", SET, ""),
        ]),
        ("su2", "Make or buy", "what is built in-house", [
            ("u_make", "Fraction of parts made in-house", SET, ""),
        ]),
        ("su3", "Supplier qualification", "who may supply a flight part", [
            ("u_qual", "Qualified suppliers", SET, ""),
        ]),
    ]),
    ("hrt", "Heritage", "what the catalogue has already proven", [
        ("hr1", "Flight record", "per catalogue family", [
            ("h_units", "Flight units delivered", SET, ""),
            ("h_hours", "Orbit-hours of heritage", SET, ""),
            ("h_frac", "Fraction of parts with flight heritage", COMP, ""),
        ]),
    ]),
    ("rsk", "Risk management", "what the platform still believes without proof, what tested it, and what is left open; every new node version is concluded here", [
        ("rk1", "Risk register", "each risk R-xx has a level L1 (negligible) to L5 (would stop delivery); only a test lowers a level", [
            ("r_open", "Risks open", SET, ""),
            ("r_high", "Risks open at level L4 or L5", SET, ""),
            ("r_closed", "Risks closed", SET, ""),
            ("r_newq", "Risks opened this quarter", SET, ""),
            ("r_closq", "Risks closed this quarter", SET, ""),
        ]),
        ("rk2", "Beliefs and versions", "every node version records the belief it rests on; a new version exists because a belief broke or needed proof", [
            ("b_broke", "Beliefs a test broke", SET, ""),
            ("b_held", "Beliefs a test held", SET, ""),
            ("b_untest", "Beliefs not yet tested", SET, ""),
            ("b_total", "Beliefs recorded", SET, ""),
            ("b_vers", "Node versions released", SET, ""),
            ("b_cost", "Cost of the tests this quarter", SET, ""),
        ]),
        ("rk3", "Open risk by area", "the highest open level in each area a decision is made in", [
            ("a_node", "Highest open risk: nodes", SET, ""),
            ("a_in", "Highest open risk: case inputs", SET, ""),
            ("a_out", "Highest open risk: outputs and results", SET, ""),
            ("a_model", "Highest open risk: models", SET, ""),
            ("a_math", "Highest open risk: maths", SET, ""),
            ("a_alg", "Highest open risk: algorithms", SET, ""),
            ("a_viz", "Highest open risk: visualisation", SET, ""),
        ]),
        ("rk4", "Conclusion", "what the ledger says about the platform as a whole, release by release", [
            ("c_level", "Highest open risk level", COMP, ""),
            ("c_burn", "Net risks closed this quarter", COMP, ""),
            ("c_learn", "Share of beliefs tested", COMP, ""),
        ]),
    ]),
])

# --------------------------------------------------------------------------
# Layer 2 — the system: the case's satellite, as the ADCS sees it.
# --------------------------------------------------------------------------

def kpi_pair(gid, label, what, leaves, extra=""):
    """A KPI branch: a required group and an achieved group with the same labels."""
    return (gid, label, what, [
        (gid + "k", "KPI required · " + label.lower(), "%d targets this service must hit" % len(leaves),
         [(gid + "k_" + k, lab, REQ, ex) for k, lab, ex in leaves]),
        (gid + "a", "KPI achieved · " + label.lower(), "what the design actually delivers",
         [(gid + "a_" + k, lab, ACH, ex) for k, lab, ex in leaves]),
    ])

SYS = ("prg", "Satellite ADCS", "the case's satellite, as the attitude system sees it", [
    ("svc", "Pointing service the customer needs", "what the case asks for — everything below exists to meet it", [
        kpi_pair("p1", "Pointing performance", "how well it points and knows where it points", [
            ("ape", "Absolute pointing error (APE)", ""),
            ("ake", "Absolute knowledge error (AKE)", ""),
            ("rpe", "Relative pointing error (RPE)", ""),
            ("pde", "Pointing drift error (PDE)", ""),
            ("rks", "Rate stability", ""),
            ("rke", "Relative knowledge error (RKE)", ""),
        ]),
        kpi_pair("p2", "Agility", "how fast it moves from one target to the next", [
            ("slew", "Reference slew time", ""),
            ("settle", "Settling time after a slew", ""),
            ("spo", "Slews per orbit", ""),
            ("track", "Target-tracking rate", ""),
            ("wmax", "Maximum body rate", ""),
        ]),
        kpi_pair("p3", "Robustness and safety", "what it survives, and how fast it recovers", [
            ("detumble", "Detumble time", ""),
            ("sunacq", "Sun-acquisition time in safe mode", ""),
            ("faults", "Faults tolerated", ""),
            ("recover", "Recovery time after a single fault", ""),
            ("hsat", "Momentum saturation margin", ""),
            ("dump", "Momentum dump interval", ""),
        ]),
        kpi_pair("p4", "Resource footprint", "what the ADCS costs the satellite", [
            ("mass", "ADCS mass", ""),
            ("pavg", "ADCS orbit-average power", ""),
            ("ppk", "ADCS peak power", ""),
            ("vol", "ADCS volume", ""),
            ("prop", "RCS propellant per year", "rcs"),
        ]),
    ]),
    ("cpt", "ADCS configuration", "set from the chosen product or part combination — counts, so absence is zero, never a missing row", [
        ("cf", "Hardware fitted", "set from the product or part combination: filled slots' counts, zero for the rest", [
            ("n_mtq", "Magnetorquer axes fitted", SET, "mtq"),
            ("n_rw", "Reaction wheels fitted", SET, "rw"),
            ("n_fmr", "Fluid momentum rings fitted", SET, "fmr"),
            ("n_rcs", "RCS thrusters fitted", SET, "rcs"),
            ("n_st", "Star tracker heads fitted", SET, ""),
            ("n_gyro", "Gyro units fitted", SET, ""),
        ]),
    ]),
    ("msn", "Mission and orbit", "where it flies and what it is asked to do there", [
        ("m1", "Mission requirements", "what the mission sets, before any hardware", [
            ("life", "Mission lifetime", SET, ""),
            ("epoch", "Launch epoch", SET, ""),
            ("duty", "Fine-pointing duty cycle", SET, ""),
            ("spd", "Slews per day", SET, ""),
            ("sangle", "Reference slew angle", SET, ""),
            ("w0", "Initial tumble rate after separation", SET, ""),
        ]),
        ("m2", "Orbit", "the orbit the case flies", [
            ("alt", "Altitude", SET, ""),
            ("inc", "Inclination", SET, ""),
            ("ecc", "Eccentricity", SET, ""),
            ("ltan", "LTAN", SET, ""),
            ("rad", "Orbit radius", COMP, ""),
            ("per", "Orbital period", COMP, ""),
            ("n", "Orbital rate", COMP, ""),
            ("ecl", "Eclipse fraction", COMP, ""),
        ]),
        ("m3", "Environment along the orbit", "what the orbit exposes the satellite to", [
            ("bmin", "Field strength, minimum", COMP, ""),
            ("bmax", "Field strength, maximum", COMP, ""),
            ("mlat", "Magnetic latitude range", COMP, ""),
            ("rho", "Atmospheric density at altitude", COMP, ""),
            ("vorb", "Orbital speed", COMP, ""),
            ("psrp", "Solar radiation pressure", COMP, ""),
        ]),
    ]),
    ("sat", "Satellite as the ADCS sees it", "the vehicle the case describes, before any ADCS part is chosen", [
        ("s1", "Mass properties", "what resists being turned", [
            ("m", "Satellite mass", SET, ""),
            ("imax", "Largest principal inertia", SET, ""),
            ("iint", "Intermediate principal inertia", SET, ""),
            ("imin", "Smallest principal inertia", SET, ""),
            ("cm", "Centre-of-mass offset", SET, ""),
            ("iunc", "Inertia uncertainty", SET, ""),
        ]),
        ("s2", "Surfaces and offsets", "what the air and the light push on", [
            ("afr", "Frontal area", SET, ""),
            ("cpa", "Aerodynamic centre-of-pressure offset", SET, ""),
            ("asun", "Sunlit area", SET, ""),
            ("cps", "Solar centre-of-pressure offset", SET, ""),
            ("refl", "Surface reflectivity", SET, ""),
            ("cd", "Drag coefficient", SET, ""),
        ]),
        ("s3", "Magnetic cleanliness", "the satellite's own field", [
            ("dres", "Residual dipole", SET, ""),
            ("dunc", "Residual dipole uncertainty", SET, ""),
        ]),
        ("s4", "Flexible modes", "what rings when the body is moved", [
            ("fmode", "First flexible mode frequency", SET, ""),
            ("mpart", "Modal participation", SET, ""),
        ]),
        ("s5", "Resources offered to the ADCS", "what the bus gives the ADCS to work with", [
            ("palloc", "Power allocated to the ADCS", SET, ""),
            ("malloc", "Mass allocated to the ADCS", SET, ""),
            ("valloc", "Volume allocated to the ADCS", SET, ""),
            ("vbus", "Bus voltage", SET, ""),
            ("nif", "OBC data interfaces offered", SET, ""),
        ]),
    ]),
    ("sub", "ADCS subsystems", "grouped by what each one is there to do", [
        ("sb0", "Know what pushes it", "the torques the ADCS must reject", [
            ("gd", "Disturbance torques", "what the environment does to this satellite", [
                ("tgg", "Gravity-gradient torque", COMP, ""),
                ("taero", "Aerodynamic torque", COMP, ""),
                ("tsrp", "Solar-pressure torque", COMP, ""),
                ("tmag", "Residual-magnetic torque", COMP, ""),
                ("ttot", "Total disturbance torque", COMP, ""),
                ("hsec", "Secular momentum per orbit", COMP, ""),
            ]),
        ]),
        ("sb1", "Sense it", "attitude sensing and estimation", [
            ("gs", "Attitude sensors", "the error each sensor brings", [
                ("stn", "Star tracker noise, cross-boresight", SET, ""),
                ("str", "Star tracker noise, about boresight", SET, ""),
                ("arw", "Gyro angle random walk", SET, ""),
                ("bi", "Gyro bias instability", SET, ""),
                ("mgn", "Magnetometer noise", SET, ""),
                ("ssa", "Sun sensor accuracy", SET, ""),
                ("ali", "Sensor alignment error", SET, ""),
            ]),
            ("ge", "Attitude estimation", "what the filter knows from those sensors", [
                ("fest", "Estimator update rate", SET, ""),
                ("kst", "Knowledge error, star tracker available", COMP, ""),
                ("kgy", "Knowledge error, gyro-only propagation", COMP, ""),
                ("tgy", "Gyro-only propagation time allowed", COMP, ""),
                ("kmg", "Knowledge error, magnetometer and sun only", COMP, ""),
                ("kfit", "Knowledge error, sensors fitted", COMP, ""),
            ]),
        ]),
        ("sb2", "Act on it", "the actuators, one group per catalogue family", [
            ("gm", "Magnetic actuation", "coils: the safety floor in every family", [
                ("mav", "Dipole available per axis", SET, "mtq"),
                ("mrej", "Dipole needed to reject disturbances", COMP, "mtq"),
                ("mdmp", "Dipole needed to dump momentum", COMP, "mtq"),
                ("tmq", "Magnetic torque authority", COMP, "mtq"),
                ("pmq", "Magnetorquer power", COMP, "mtq"),
                ("wmq", "Magnetorquer mass", COMP, "mtq"),
            ]),
            ("gw", "Reaction wheels", "stored momentum for agility and quiet holds", [
                ("hwc", "Wheel momentum capacity", SET, "rw"),
                ("twc", "Wheel torque capacity", SET, "rw"),
                ("hcyc", "Cyclic momentum to store", COMP, "rw"),
                ("hslw", "Momentum for the reference slew", COMP, "rw,fmr,rcs"),
                ("tpk", "Peak torque for the reference slew", COMP, "rw,fmr,rcs"),
                ("pwh", "Wheel power", COMP, "rw"),
                ("wwh", "Wheel mass", COMP, "rw"),
            ]),
            ("gf", "Fluid momentum rings", "IDMAS V2: liquid metal in a sealed channel", [
                ("bore", "Ring channel bore", SET, "fmr"),
                ("area", "Ring enclosed area", SET, "fmr"),
                ("rhof", "Ring fluid density", SET, "fmr"),
                ("muf", "Ring fluid viscosity", SET, "fmr"),
                ("vcr", "Ring cruise speed", SET, "fmr"),
                ("len", "Ring channel length", SET, "fmr"),
                ("hring", "Ring momentum at cruise speed", COMP, "fmr"),
                ("tspin", "Ring spin-down time", COMP, "fmr"),
                ("phold", "Ring holding power", COMP, "fmr"),
                ("dp", "Ring pump pressure for the reference slew", COMP, "fmr"),
            ]),
            ("gr", "Reaction control thrusters", "IDMAS V3: slew excess, fast dumps, orbit trim", [
                ("thr", "Thrust per thruster", SET, "rcs"),
                ("arm", "Thruster moment arm", SET, "rcs"),
                ("isp", "Specific impulse", SET, "rcs"),
                ("mps", "Propellant per slew", COMP, "rcs"),
                ("mpy", "Propellant per year", COMP, "rcs"),
            ]),
        ]),
        ("sb3", "Close the loop", "control, allocation, the pointing budget and the modes", [
            ("gc", "Control and allocation", "one allocation for every family", [
                ("bw", "Control bandwidth", SET, ""),
                ("zeta", "Damping ratio", SET, ""),
                ("ts", "Settling time", COMP, ""),
                ("tauth", "Torque authority, combined", COMP, ""),
                ("hauth", "Momentum authority, combined", COMP, ""),
                ("tslew", "Reference slew time achievable", COMP, ""),
            ]),
            ("gp", "Pointing error budget", "ECSS-E-ST-60-10C: every contributor, one budget", [
                ("ek", "Knowledge contribution", COMP, ""),
                ("ec", "Control contribution", COMP, ""),
                ("ea", "Alignment contribution", COMP, ""),
                ("et", "Thermal distortion contribution", SET, ""),
                ("ej", "Jitter contribution", COMP, ""),
                ("eape", "APE budget total", COMP, ""),
            ]),
            ("gq", "Modes and FDIR", "detumble, safe mode, and what a fault costs", [
                ("tdet", "Detumble time estimate", COMP, ""),
                ("tsun", "Safe-mode sun acquisition estimate", COMP, ""),
                ("nft", "Faults tolerated by the configuration", COMP, ""),
                ("trec", "Fault recovery time estimate", COMP, ""),
                ("tdmp", "Momentum dump interval estimate", COMP, ""),
            ]),
        ]),
        ("sb4", "Host it", "the software, the OBC and the unit's own budgets", [
            ("gx", "Flight software and OBC interfaces", "what the loop costs the computer", [
                ("fctl", "Control loop rate", SET, ""),
                ("cpu", "OBC CPU load", COMP, ""),
                ("nused", "Interfaces used", COMP, ""),
                ("lat", "Sensor-to-actuator latency", COMP, ""),
                ("tmr", "Telemetry rate", COMP, ""),
            ]),
            ("gb", "ADCS unit budgets", "the totals the case's allocation is checked against", [
                ("bm", "ADCS mass total", COMP, ""),
                ("bpa", "ADCS orbit-average power total", COMP, ""),
                ("bpp", "ADCS peak power total", COMP, ""),
                ("bv", "ADCS volume total", COMP, ""),
            ]),
        ]),
    ]),
    ("ver", "Verification", "how each achieved number was earned", [
        ("v1", "Verification coverage", "which rung supplied each achieved value", [
            ("va", "Requirements verified by analysis", SET, ""),
            ("vs", "Requirements verified by SILS", SET, ""),
            ("vo", "Requirements verified by OILS", SET, ""),
            ("vh", "Requirements verified by HILS", SET, ""),
            ("vn", "Requirements not yet verified", SET, ""),
        ]),
        ("v2", "Campaign evidence", "the size of the evidence behind the numbers", [
            ("nmc", "Monte Carlo runs", SET, ""),
            ("nedge", "Edge cases run", SET, ""),
            ("dso", "SILS to OILS difference, APE", SET, ""),
            ("dsh", "SILS to HILS difference, APE", SET, ""),
        ]),
        ("v3", "OILS rig needs", "what the OBC-in-the-loop rig must do to test this case's unit", [
            ("o_step", "Real-time plant step needed", COMP, ""),
            ("o_ports", "Flight ports to answer", COMP, ""),
            ("o_lat", "Rig loop latency allowed", COMP, ""),
            ("o_link", "Emulator link rate needed", COMP, ""),
            ("o_time", "Real-time run length", COMP, ""),
        ]),
        ("v4", "HILS rig needs", "what the cage, the bearing, the stimulators and the test stands must do for this case's unit", [
            ("h_brng", "Cage field range needed", COMP, ""),
            ("h_bacc", "Cage field accuracy needed", COMP, ""),
            ("h_bslew", "Cage field slew rate needed", COMP, ""),
            ("h_tbear", "Bearing residual torque allowed", COMP, ""),
            ("h_cmoff", "Bearing balance offset allowed", COMP, ""),
            ("h_tilt", "Bearing tilt range needed", COMP, ""),
            ("h_iplat", "Platform inertia to match", COMP, ""),
            ("h_truth", "Truth attitude accuracy needed", COMP, ""),
            ("h_sunirr", "Sun simulator irradiance needed", COMP, ""),
            ("h_suncol", "Sun simulator collimation needed", COMP, ""),
            ("h_starerr", "Star stimulator error allowed", COMP, ""),
            ("h_starfr", "Star stimulator frame rate needed", COMP, ""),
            ("h_dip", "Coil dipole to measure", COMP, "mtq"),
            ("h_rwd", "Wheel torque and disturbance to measure", COMP, "rw"),
            ("h_ring", "Ring torque to measure", COMP, "fmr"),
            ("h_thr", "Thrust to measure", COMP, "rcs"),
        ]),
    ]),
])

# --------------------------------------------------------------------------
# Layer 3 — shape only, as in CD-06: which subsystem layers exist, which
# layer-2 group each closes against, and how many rows it holds in total.
# targets is computed from the group, so the seeder's assertion cannot fire.
# --------------------------------------------------------------------------

LAYER3 = [
    # (sid, label, layer-2 group, total rows)
    ("dist", "Disturbance environment", "gd", 25),
    ("sens", "Attitude sensors", "gs", 29),
    ("est", "Attitude estimation", "ge", 22),
    ("mtq", "Magnetic actuation", "gm", 26),
    ("rw", "Reaction wheels", "gw", 26),
    ("fmr", "Fluid momentum rings", "gf", 36),
    ("rcs", "Reaction control thrusters", "gr", 22),
    ("ctl", "Control and allocation", "gc", 28),
    ("pnt", "Pointing error budget", "gp", 25),
    ("modes", "Modes and FDIR", "gq", 22),
    ("fsw", "Flight software and OBC interfaces", "gx", 22),
    ("budget", "ADCS unit budgets", "gb", 17),
    ("oils", "OILS rig", "v3", 20),
    ("hils", "HILS rig", "v4", 48),
]

# --------------------------------------------------------------------------
# Edges, by local key. ED_* = group relations (why), VE = derivation
# (producer -> consumer, why), KE = contribution (variable -> KPI achieved).
# --------------------------------------------------------------------------

ED_MGT = [
    ("cas", "cat", "every case is solved against the catalogue first"),
    ("cat", "cmr", "the selected product is what gets priced"),
    ("cmr", "ord", "a frozen quote becomes an order"),
    ("ord", "fac", "an order books rig time"),
    ("fac", "std", "every campaign is run to a test standard"),
    ("std", "ord", "the certificate closes the order"),
    ("sup", "ord", "long-lead parts set the build stage"),
    ("hrt", "cat", "heritage is recorded per product"),
    ("cat", "rsk", "every product and node decision records the belief it rests on"),
    ("rsk", "std", "open risk levels feed the quality verdict"),
]

ED_SYS = [
    ("svc", "cpt", "the service asked for picks the family"),
    ("m2", "m3", "the orbit sets the environment"),
    ("m3", "gd", "field, density and light set the torques"),
    ("s1", "gd", "inertia sets the gravity-gradient torque"),
    ("s2", "gd", "areas and offsets set aero and solar torques"),
    ("s3", "gd", "the residual dipole sets the magnetic torque"),
    ("gd", "gm", "the coils must reject and dump what the environment adds"),
    ("gd", "gw", "the wheels must store the cyclic part"),
    ("m1", "gw", "the reference slew sizes the stored momentum"),
    ("gw", "gf", "the rings are sized against the same slew"),
    ("gw", "gr", "the thrusters carry what the rings cannot"),
    ("cpt", "gc", "the counts fitted set the combined authority"),
    ("gm", "gc", "coil authority enters the allocation"),
    ("gf", "gc", "ring authority enters the allocation"),
    ("gr", "gc", "thruster authority enters the allocation"),
    ("gs", "ge", "sensor errors set what the filter can know"),
    ("ge", "gp", "knowledge enters the pointing budget"),
    ("gc", "gp", "control error enters the pointing budget"),
    ("s4", "gc", "the first mode caps the bandwidth"),
    ("gc", "gq", "authority sets detumble and recovery"),
    ("gx", "gp", "latency enters the pointing budget"),
    ("cpt", "gb", "every fitted part is in the budget"),
    ("s5", "gb", "the allocation is what the budget closes against"),
    ("gp", "p1a", "the budget is the achieved pointing"),
    ("gc", "p2a", "authority and bandwidth are the achieved agility"),
    ("gq", "p3a", "modes are the achieved robustness"),
    ("gb", "p4a", "the budget totals are the achieved footprint"),
    ("v1", "svc", "each achieved KPI carries the rung that earned it"),
    ("gx", "v3", "the flight loop sets the real-time plant"),
    ("cpt", "v3", "every fitted part is a flight port the rig answers"),
    ("m3", "v4", "the orbit's field is what the cage must make"),
    ("gc", "v4", "control authority sets the bearing torque allowed"),
    ("gs", "v4", "the sensors fitted set the stimulators"),
    ("cpt", "v4", "the hardware fitted sets what the test stands measure"),
    ("v3", "v1", "OILS campaigns verify what OILS can"),
    ("v4", "v1", "HILS campaigns verify the unit"),
]

VE = [
    ("alt", "rad", "radius from altitude"),
    ("rad", "per", "period from the semi-major axis"),
    ("per", "n", "mean motion from the period"),
    ("alt", "ecl", "eclipse from the Earth's angular radius"),
    ("ltan", "ecl", "beta angle from the node's local time"),
    ("inc", "mlat", "the orbit reaches the magnetic latitudes its inclination allows"),
    ("rad", "bmin", "dipole field falls as the cube of radius"),
    ("rad", "bmax", "dipole field falls as the cube of radius"),
    ("mlat", "bmax", "the field doubles from equator to pole"),
    ("alt", "rho", "density from altitude"),
    ("epoch", "rho", "solar activity at the launch epoch"),
    ("rad", "vorb", "circular speed at that radius"),
    ("rad", "tgg", "gravity gradient falls as the cube of radius"),
    ("imax", "tgg", "the inertia difference drives the torque"),
    ("imin", "tgg", "the inertia difference drives the torque"),
    ("rho", "taero", "dynamic pressure"),
    ("vorb", "taero", "dynamic pressure"),
    ("afr", "taero", "the area the air pushes on"),
    ("cpa", "taero", "the lever arm"),
    ("cd", "taero", "the drag coefficient"),
    ("psrp", "tsrp", "the light's pressure"),
    ("asun", "tsrp", "the area it pushes on"),
    ("cps", "tsrp", "the lever arm"),
    ("refl", "tsrp", "reflection adds to absorption"),
    ("dres", "tmag", "residual dipole across the field"),
    ("bmax", "tmag", "the worst-case field"),
    ("tgg", "ttot", "summed worst case"),
    ("taero", "ttot", "summed worst case"),
    ("tsrp", "ttot", "summed worst case"),
    ("tmag", "ttot", "summed worst case"),
    ("ttot", "hsec", "a torque held for an orbit"),
    ("per", "hsec", "one orbit"),
    ("ttot", "mrej", "torque over the weakest field"),
    ("bmin", "mrej", "the weakest field along the orbit"),
    ("hsec", "mdmp", "momentum to remove each orbit"),
    ("bmin", "mdmp", "the weakest field"),
    ("per", "mdmp", "the time available to dump it"),
    ("mav", "tmq", "dipole fitted"),
    ("bmin", "tmq", "the weakest field"),
    ("n_mtq", "tmq", "axes fitted"),
    ("mav", "pmq", "power grows with the square of the dipole"),
    ("n_mtq", "wmq", "one coil set per axis"),
    ("ttot", "hcyc", "cyclic disturbance momentum"),
    ("per", "hcyc", "over a quarter orbit"),
    ("imax", "hslw", "bang-bang slew about the largest axis"),
    ("sangle", "hslw", "the reference slew angle"),
    ("p2k_slew", "hslw", "in the required slew time"),
    ("n_rw", "pwh", "wheels fitted"),
    ("n_rw", "wwh", "wheels fitted"),
    ("bore", "hring", "momentum grows with the bore area"),
    ("area", "hring", "momentum grows with the enclosed area"),
    ("rhof", "hring", "momentum grows with the fluid density"),
    ("vcr", "hring", "momentum grows with the flow speed"),
    ("bore", "tspin", "spin-down grows with the square of the bore"),
    ("rhof", "tspin", "denser fluid coasts longer"),
    ("muf", "tspin", "viscosity drains it"),
    ("hring", "phold", "holding power grows with momentum squared"),
    ("muf", "phold", "viscous loss"),
    ("len", "phold", "loss grows with the channel length"),
    ("rhof", "phold", "denser fluid needs less speed for the same momentum"),
    ("bore", "phold", "a wider bore loses less"),
    ("area", "phold", "more enclosed area needs less speed"),
    ("imax", "tpk", "bang-bang slew about the largest axis"),
    ("sangle", "tpk", "the reference slew angle"),
    ("p2k_slew", "tpk", "in the required slew time"),
    ("tpk", "dp", "the slew's peak torque"),
    ("len", "dp", "the pressure acts along the channel length"),
    ("area", "dp", "the channel's enclosed area"),
    ("bore", "dp", "the channel's bore"),
    ("hslw", "mps", "momentum to add and remove"),
    ("arm", "mps", "the lever arm"),
    ("isp", "mps", "specific impulse"),
    ("mps", "mpy", "per slew"),
    ("spd", "mpy", "slews per day"),
    ("n_rcs", "mpy", "zero when no thrusters are fitted"),
    ("bw", "ts", "settling from the closed-loop poles"),
    ("zeta", "ts", "settling from the closed-loop poles"),
        ("tmq", "tauth", "coil authority"),
    ("twc", "tauth", "wheel torque"),
    ("n_rw", "tauth", "wheels fitted"),
    ("thr", "tauth", "thruster torque"),
    ("arm", "tauth", "thruster torque"),
    ("n_rcs", "tauth", "thrusters fitted"),
    ("hwc", "hauth", "wheel momentum"),
    ("n_rw", "hauth", "wheels fitted"),
    ("hring", "hauth", "ring momentum"),
    ("n_fmr", "hauth", "rings fitted"),
    ("tauth", "tslew", "torque-limited slew"),
    ("hauth", "tslew", "momentum-limited coast"),
    ("imax", "tslew", "about the largest axis"),
    ("sangle", "tslew", "the reference angle"),
    ("epoch", "psrp", "sun distance on that date"),
    ("arw", "tgy", "the gap the random walk allows"),
    ("bi", "tgy", "the gap the bias allows"),
    ("p1k_ake", "tgy", "the knowledge the gap may cost"),
    ("stn", "kst", "star tracker noise"),
    ("str", "kst", "star tracker noise about boresight"),
    ("arw", "kst", "gyro noise between updates"),
    ("fest", "kst", "update interval"),
    ("n_st", "kst", "heads fitted"),
    ("arw", "kgy", "angle random walk over the gap"),
    ("bi", "kgy", "bias instability over the gap"),
    ("tgy", "kgy", "the gap"),
    ("mgn", "kmg", "field direction noise"),
    ("bmin", "kmg", "the weakest field is the noisiest direction"),
    ("ssa", "kmg", "sun direction accuracy"),
    ("kst", "kfit", "the star tracker's knowledge, when one is fitted"),
    ("kmg", "kfit", "magnetometer and sun knowledge, when no star tracker is"),
    ("n_st", "kfit", "which of the two the fitted sensors give"),
    ("kfit", "ek", "knowledge enters the budget"),
    ("ts", "ec", "control error from the loop"),
    ("ttot", "ec", "disturbance the loop must reject"),
    ("tauth", "ec", "authority available to reject it"),
    ("ali", "ea", "alignment enters the budget"),
    ("lat", "ej", "latency against the rate"),
    ("ek", "eape", "summed per ECSS-E-ST-60-10C"),
    ("ec", "eape", "summed per ECSS-E-ST-60-10C"),
    ("ea", "eape", "summed per ECSS-E-ST-60-10C"),
    ("et", "eape", "summed per ECSS-E-ST-60-10C"),
    ("ej", "eape", "summed per ECSS-E-ST-60-10C"),
    ("w0", "tdet", "the rate to remove"),
    ("imax", "tdet", "the inertia to slow"),
    ("tmq", "tdet", "coil authority"),
    ("tauth", "tsun", "authority to acquire"),
    ("n_mtq", "nft", "redundancy per family"),
    ("n_rw", "nft", "redundancy per family"),
    ("n_fmr", "nft", "redundancy per family"),
    ("n_rcs", "nft", "redundancy per family"),
    ("tauth", "trec", "authority after a fault"),
    ("hsec", "tdmp", "momentum gathered per orbit"),
    ("hauth", "tdmp", "momentum that can be stored"),
    ("mdmp", "tdmp", "dipole needed against dipole available"),
    ("fctl", "cpu", "loop rate"),
    ("fctl", "lat", "one loop period"),
    ("n_mtq", "nused", "every fitted part needs a port"),
    ("n_rw", "nused", "every fitted part needs a port"),
    ("n_fmr", "nused", "every fitted part needs a port"),
    ("n_rcs", "nused", "every fitted part needs a port"),
    ("n_st", "nused", "every fitted part needs a port"),
    ("n_gyro", "nused", "every fitted part needs a port"),
    ("fctl", "tmr", "one frame per loop"),
    ("wmq", "bm", "coil mass"),
    ("wwh", "bm", "wheel mass"),
    ("pmq", "bpa", "coil power"),
    ("pwh", "bpa", "wheel power"),
    ("phold", "bpa", "ring holding power"),
    ("n_fmr", "bpa", "holding power per ring, times the rings fitted; zero when none"),
    ("pmq", "bpp", "coil peak"),
    ("pwh", "bpp", "wheel peak"),
    ("dp", "bpp", "ring sprint power"),
    ("n_fmr", "bpp", "sprint power per ring, times the rings fitted; zero when none"),
    ("n_mtq", "bv", "volume per part"),
    ("n_rw", "bv", "volume per part"),
    ("n_fmr", "bv", "volume per part"),
    ("n_rcs", "bv", "volume per part"),
    # management
    ("c_bom", "c_unit", "parts"),
    ("c_build", "c_unit", "labour"),
    ("c_cal", "c_unit", "labour"),
    ("c_unit", "q_unit", "price from cost and margin"),
    ("t_margin", "q_unit", "price from cost and margin"),
    ("t_disc", "q_unit", "the data-share discount"),
    ("c_day", "q_oils", "rig days"),
    ("c_day", "q_hils", "rig days"),
    ("q_unit", "q_total", "summed"),
    ("q_oils", "q_total", "summed"),
    ("q_hils", "q_total", "summed"),
    ("q_unit", "t_ach", "price against cost"),
    ("c_unit", "t_ach", "price against cost"),
    ("d_short", "d_lead", "summed"), ("d_quote", "d_lead", "summed"),
    ("d_po", "d_lead", "summed"), ("d_build", "d_lead", "summed"),
    ("d_cal", "d_lead", "summed"), ("d_oils", "d_lead", "summed"),
    ("d_hils", "d_lead", "summed"), ("d_cert", "d_lead", "summed"),
    ("d_ship", "d_lead", "summed"),
    ("f_cores", "f_mcph", "runs scale with cores"),
    ("d_oils", "f_days", "rig days per order"),
    ("d_hils", "f_days", "rig days per order"),
    ("f_days", "f_util", "booked against available"),
    ("f_ohours", "f_util", "booked against available"),
    ("h_units", "h_frac", "units flown"),
    ("k_cov", "k_frac", "classes covered"),
    ("k_cls", "k_frac", "of classes defined"),
    ("k_pass", "k_yield", "combinations that passed"),
    ("k_runs", "k_yield", "of combinations evaluated"),
    ("k_hit", "k_hitr", "cases matched"),
    ("k_cases", "k_hitr", "of cases solved"),
    # verification: what each rung's rig must do for this case (layer 2)
    ("fctl", "o_step", "the plant runs several steps per flight cycle"),
    ("n_mtq", "o_ports", "each fitted part is a port"), ("n_rw", "o_ports", "each fitted part is a port"),
    ("n_fmr", "o_ports", "each fitted part is a port"), ("n_rcs", "o_ports", "each fitted part is a port"),
    ("n_st", "o_ports", "each fitted part is a port"), ("n_gyro", "o_ports", "each fitted part is a port"),
    ("nused", "o_ports", "the interfaces the flight software uses"),
    ("lat", "o_lat", "the rig may add only a share of the loop's latency"),
    ("fctl", "o_lat", "within one flight cycle"),
    ("o_ports", "o_link", "every port's traffic crosses the link"),
    ("fctl", "o_link", "once per flight cycle"),
    ("per", "o_time", "a real-time run lasts at least the orbit it flies"),
    ("bmax", "h_brng", "the strongest field on the orbit"),
    ("bmin", "h_bacc", "the weakest field sets the error allowed"),
    ("mgn", "h_bacc", "the magnetometer must not see the cage's error"),
    ("bmax", "h_bslew", "the field turns once per orbit, twice as fast near the poles"),
    ("n", "h_bslew", "at the orbital rate"),
    ("tauth", "h_tbear", "the bearing's torque must stay small beside the authority"),
    ("ttot", "h_tbear", "and beside the orbit's disturbance it stands in for"),
    ("h_tbear", "h_cmoff", "torque is weight times offset"),
    ("m", "h_cmoff", "the weight on the bearing"),
    ("sangle", "h_tilt", "the reference slew, about an axis the bearing may tilt"),
    ("imax", "h_iplat", "the platform must carry the unit's inertia"),
    ("iint", "h_iplat", "the platform must carry the unit's inertia"),
    ("imin", "h_iplat", "the platform must carry the unit's inertia"),
    ("kfit", "h_truth", "truth must be finer than the knowledge it checks"),
    ("psrp", "h_sunirr", "the Sun's pressure is its irradiance over the speed of light"),
    ("ssa", "h_suncol", "the simulated Sun must be sharper than the sensor"),
    ("stn", "h_starerr", "the rendered sky must be finer than the tracker"),
    ("str", "h_starerr", "the rendered sky must be finer than the tracker"),
    ("fest", "h_starfr", "a frame for every estimator update"),
    ("mav", "h_dip", "the dipole each coil must be shown to give"),
    ("twc", "h_rwd", "the torque the wheel must be shown to give"),
    ("hwc", "h_rwd", "the momentum at which its imbalance is measured"),
    ("tpk", "h_ring", "the torque the rings must be shown to give"),
    ("hring", "h_ring", "at the momentum they carry"),
    ("thr", "h_thr", "the thrust each thruster must be shown to give"),
    ("a_node", "c_level", "highest over the areas"), ("a_in", "c_level", "highest over the areas"),
    ("a_out", "c_level", "highest over the areas"), ("a_model", "c_level", "highest over the areas"),
    ("a_math", "c_level", "highest over the areas"), ("a_alg", "c_level", "highest over the areas"),
    ("a_viz", "c_level", "highest over the areas"),
    ("r_closq", "c_burn", "closed less opened"),
    ("r_newq", "c_burn", "closed less opened"),
    ("b_untest", "c_learn", "tested share of the beliefs"),
    ("b_total", "c_learn", "tested share of the beliefs"),
]

KE = [
    ("eape", "p1a_ape", "the budget total is the achieved APE"),
    ("kfit", "p1a_ake", "knowledge with the sensors fitted"),
    ("ej", "p1a_rpe", "jitter sets the relative error"),
    ("et", "p1a_pde", "thermal drift sets the drift error"),
    ("ts", "p1a_rks", "the loop sets rate stability"),
    ("arw", "p1a_rke", "gyro noise sets relative knowledge"),
    ("tslew", "p2a_slew", "the reference slew achievable"),
    ("ts", "p2a_settle", "settling after the slew"),
    ("tslew", "p2a_spo", "slews that fit in an orbit"),
    ("tauth", "p2a_track", "authority sets the tracking rate"),
    ("hauth", "p2a_wmax", "stored momentum sets the maximum rate"),
    ("tdet", "p3a_detumble", "detumble estimate"),
    ("tsun", "p3a_sunacq", "sun acquisition estimate"),
    ("nft", "p3a_faults", "faults the configuration tolerates"),
    ("trec", "p3a_recover", "recovery estimate"),
    ("hauth", "p3a_hsat", "stored momentum against the cyclic load"),
    ("hcyc", "p3a_hsat", "stored momentum against the cyclic load"),
    ("tdmp", "p3a_dump", "dump interval estimate"),
    ("bm", "p4a_mass", "mass total"),
    ("bpa", "p4a_pavg", "average power total"),
    ("bpp", "p4a_ppk", "peak power total"),
    ("bv", "p4a_vol", "volume total"),
    ("mpy", "p4a_prop", "propellant per year"),
]


# --------------------------------------------------------------------------
# Who supplies each declared layer-2 leaf. Every one has exactly one supplier,
# so every value a run reads has exactly one place it comes from (SPEC.md §8.2):
#   case     — the case CSV, row by row (CASE_INPUTS and the requirement rows)
#   product  — the catalogue product or part combination, through
#              adcs-config's supply map (SPEC.md §8.4)
#   tuned    — the product's algorithm parameters, set per case by adcs-tune
#              within the algorithm's bounds (SPEC.md §8.5)
#   evidence — the evidence tooling, from finished campaigns (SPEC.md §14)
#   derisk   — layer 1 only: the de-risking ledger, counted by `cargo xtask derisk
#              rollup` (SPEC.md §5.13)
#   lab      — layer 1 only: the facility's measured capabilities, read from the
#              lab file the rig runs from (SPEC.md §12.10)
# The build refuses a declared leaf with no supplier, or with two.
# --------------------------------------------------------------------------

PRODUCT = [
    "n_mtq", "n_rw", "n_fmr", "n_rcs", "n_st", "n_gyro",
    "stn", "str", "arw", "bi", "mgn", "ssa", "ali",
    "mav", "hwc", "twc",
    "bore", "area", "rhof", "muf", "len",
    "thr", "arm", "isp",
]
TUNED = ["fest", "vcr", "bw", "zeta", "fctl"]
EVIDENCE = ["va", "vs", "vo", "vh", "vn", "nmc", "nedge", "dso", "dsh"]
# Layer 1: the risk rows are counted from the de-risking ledger (derisk/, SPEC.md
# §5.13) by `cargo xtask derisk rollup`, never set by a form. Every declared leaf of the
# rsk branch is here, and nothing else.
# Layer 1: the facility's measured capabilities are read from the lab file
# (rig/labs/<bay>.toml, adcs-lab/1) the rig itself runs from, never set by a
# form: supplier `lab`, with the field each row reads (SPEC.md §12.10). A field
# that is `nan` answers NotMeasured. `max` takes the largest element of a list;
# `len` counts a list.
LAB = [
    ("f_lat", "rig_host.worst_latency_s", ""), ("f_jit", "rig_host.jitter_s", ""),
    ("f_ieu", "ieu.channels", ""), ("f_proto", "ieu.interface_kinds", "len"),
    ("f_iso", "ieu.isolation_V", ""), ("f_gnsse", "ieu.gnss_outputs_emulated", ""), ("f_link", "ieu.link_rate_bps", ""),
    ("f_bmax", "helmholtz_cage.field_range_T", ""), ("f_bunif", "helmholtz_cage.uniformity_in_test_volume", ""),
    ("f_bvol", "helmholtz_cage.test_volume_edge_m", ""), ("f_bacc", "helmholtz_cage.field_error_fraction", ""),
    ("f_bdir", "helmholtz_cage.direction_error_rad", ""), ("f_btau", "helmholtz_cage.time_constant_s", ""),
    ("f_bref", "helmholtz_cage.reference_magnetometer.noise_T_rms", ""),
    ("f_tres", "air_bearing.residual_gravity_torque_Nm", ""), ("f_tilt", "air_bearing.tilt_limit_rad", ""),
    ("f_pin", "air_bearing.platform_inertia_kgm2", "max"), ("f_pcap", "air_bearing.payload_capacity_kg", ""),
    ("f_bal", "air_bearing.balance_offset_m", ""), ("f_drag", "air_bearing.aero_drag_Nms", ""),
    ("f_truth", "air_bearing.metrology.accuracy_rad", ""), ("f_truthr", "air_bearing.metrology.rate_Hz", ""),
    ("f_sun", "sun_simulator.irradiance_W_m2", ""), ("f_suncol", "sun_simulator.collimation_half_angle_rad", ""),
    ("f_sununi", "sun_simulator.non_uniformity", ""), ("f_sunstab", "sun_simulator.temporal_instability", ""),
    ("f_star", "star_stimulator.angular_step_rad", ""), ("f_starmag", "star_stimulator.faintest_magnitude", ""),
    ("f_starfr", "star_stimulator.frame_rate_Hz", ""), ("f_starlat", "star_stimulator.latency_s", ""),
    ("f_gnss", "gnss_rf_simulator.channels", ""),
    ("f_dynbw", "test_stands.dynamometer_bandwidth_Hz", ""), ("f_imb", "test_stands.imbalance_resolution_kgm", ""),
    ("f_dipres", "test_stands.dipole_resolution_Am2", ""), ("f_ringt", "test_stands.ring_torque_resolution_Nm", ""),
    ("f_thrres", "test_stands.thrust_resolution_N", ""), ("f_ibit", "test_stands.impulse_bit_resolution_Ns", ""),
    ("f_dyntq", "test_stands.dynamometer_torque_resolution_Nm", ""),
    ("f_ilock", "safety.hardware_interlocks", "len"), ("f_estop", "safety.estop_response_s", ""),
    ("f_brown", "power.brownout_range_V", ""), ("f_psu", "power.supply_channels", ""),
]

DERISK = ["r_open", "r_high", "r_closed", "r_newq", "r_closq",
          "b_broke", "b_held", "b_untest", "b_total", "b_vers", "b_cost",
          "a_node", "a_in", "a_out", "a_model", "a_math", "a_alg", "a_viz"]

# The case CSV, adcs-case/1. Changing this list changes the format: bump the
# version, and write the migration from the old one (SPEC.md §8.3).
#   (local key, section, unit, blank, lo/hi allowed)
#   blank = "stated":  a blank leaves the row unstated; every row that reads it
#                      is blocked, naming the CSV key. Nothing is guessed.
#   blank = "default": a blank takes the sheet's reference value, and the
#                      import report lists it as an assumption the case made.
#   lo/hi: the range a Monte Carlo draws from and an edge campaign visits.
CASE_INPUTS = [
    ("life", "mission", "Year", "stated", False),
    ("epoch", "mission", "Year", "stated", False),
    ("duty", "mission", "One", "stated", False),
    ("spd", "mission", "Count", "stated", False),
    ("sangle", "mission", "Degree", "stated", False),
    ("w0", "mission", "DegreePerSecond", "stated", True),
    ("alt", "orbit", "Kilometre", "stated", True),
    ("inc", "orbit", "Degree", "stated", True),
    ("ecc", "orbit", "One", "default", False),
    ("ltan", "orbit", "Hour", "stated", True),
    ("m", "mass", "Kilogram", "stated", True),
    ("imax", "mass", "KilogramSquareMetre", "stated", True),
    ("iint", "mass", "KilogramSquareMetre", "stated", True),
    ("imin", "mass", "KilogramSquareMetre", "stated", True),
    ("cm", "mass", "Millimetre", "stated", True),
    ("iunc", "mass", "One", "stated", False),
    ("afr", "surface", "SquareMetre", "stated", True),
    ("cpa", "surface", "Metre", "stated", True),
    ("asun", "surface", "SquareMetre", "stated", True),
    ("cps", "surface", "Metre", "stated", True),
    ("refl", "surface", "One", "default", True),
    ("cd", "surface", "One", "default", True),
    ("dres", "magnetic", "AmpereSquareMetre", "stated", True),
    ("dunc", "magnetic", "AmpereSquareMetre", "stated", False),
    ("fmode", "flex", "Hertz", "stated", True),
    ("mpart", "flex", "One", "stated", False),
    ("palloc", "resources", "Watt", "stated", False),
    ("malloc", "resources", "Kilogram", "stated", False),
    ("valloc", "resources", "Litre", "stated", False),
    ("vbus", "resources", "Volt", "stated", False),
    ("nif", "resources", "Count", "stated", False),
    ("et", "pointing", "Degree", "stated", False),
]

# Requirement rows are case inputs too, in section "req". A blank is an
# unwritten requirement: its closures stay seeded. `level` is the ensemble
# probability the requirement is stated at (ECSS-E-ST-60-10C); a blank level on
# a written requirement takes DEFAULT_LEVEL and is listed as an assumption.
REQ_UNITS = {
    "ape": "Degree", "ake": "Degree", "rpe": "Degree", "pde": "Degree",
    "rks": "DegreePerSecond", "rke": "Degree",
    "slew": "Second", "settle": "Second", "spo": "Count", "track": "DegreePerSecond",
    "wmax": "DegreePerSecond",
    "detumble": "Minute", "sunacq": "Minute", "faults": "Count", "recover": "Minute",
    "hsat": "One", "dump": "Hour",
    "mass": "Kilogram", "pavg": "Watt", "ppk": "Watt", "vol": "Litre", "prop": "Kilogram",
}
DEFAULT_LEVEL = 99.73
CASE_SCHEMA = "adcs-case/1"

# Help printed in the template's note column, so the blank form explains the
# keys a person is most likely to misread. A sender may overwrite the note.
CASE_HELP = {
    # What each key means, in the words a customer or a team member uses. It is
    # printed in the blank template's note column, shown beside each row in the
    # case editor, and listed in the user manual. It never suggests a value.
    "req.ape": "largest angle allowed between where the payload axis points and where it should point, at the level given",
    "req.ake": "largest error allowed in what the ADCS believes its attitude is, at the level given",
    "req.rpe": "largest pointing wander allowed within a short window (jitter), at the level given",
    "req.pde": "largest slow pointing drift allowed over a long window (thermal, bias), at the level given",
    "req.rks": "largest body rate allowed while holding a target, at the level given",
    "req.rke": "largest error allowed in the attitude change the ADCS believes happened over a short window",
    "req.slew": "longest time allowed to turn through the reference slew angle (mission.sangle)",
    "req.settle": "longest time allowed after a slew before pointing is back within its requirement",
    "req.spo": "fewest slews the ADCS must be able to make per orbit",
    "req.track": "slowest rate at which the ADCS must be able to follow a moving target",
    "req.wmax": "lowest top body rate the ADCS must be able to reach",
    "req.detumble": "longest time allowed from separation, at mission.w0, to a slow stable spin",
    "req.sunacq": "longest time allowed in safe mode to point the panels at the Sun",
    "req.faults": "fewest single failures the ADCS must survive and keep working",
    "req.recover": "longest time allowed to recover full service after one fault",
    "req.hsat": "fraction of the stored momentum left unused, 0 to 1",
    "req.dump": "shortest time allowed between momentum dumps",
    "req.mass": "heaviest the whole ADCS may be",
    "req.pavg": "most power the ADCS may draw on average over an orbit",
    "req.ppk": "most power the ADCS may draw at any moment",
    "req.vol": "most volume the whole ADCS may take",
    "req.prop": "most thruster propellant the ADCS may use per year",
    "mission.life": "how long the satellite must work in orbit",
    "mission.epoch": "years after J2000.0: 27.0 is early 2027",
    "mission.duty": "fraction of the orbit in fine pointing, 0 to 1",
    "mission.spd": "slews per day",
    "mission.sangle": "the slew angle the slew-time requirement is stated for",
    "mission.w0": "body rate just after separation from the launcher; give lo and hi if the launcher states a range",
    "orbit.alt": "mean altitude of a circular orbit",
    "orbit.inc": "orbit inclination; about 97 to 98 degrees for sun-synchronous orbits at a few hundred km",
    "orbit.ecc": "orbit eccentricity; blank means circular, and the report lists it as assumed",
    "orbit.ltan": "local time of the ascending node, hours",
    "mass.m": "the whole satellite, wet, at launch",
    "mass.imax": "largest principal moment of inertia of the whole satellite",
    "mass.iint": "middle principal moment of inertia",
    "mass.imin": "smallest principal moment of inertia",
    "mass.cm": "centre-of-mass offset from the geometric centre",
    "mass.iunc": "fraction, 0 to 1",
    "surface.afr": "area facing the direction of flight, in the attitude it flies most",
    "surface.cpa": "distance between the centre of pressure for drag and the centre of mass",
    "surface.asun": "area facing the Sun, in the attitude it flies most",
    "surface.cps": "distance between the centre of pressure for sunlight and the centre of mass",
    "surface.refl": "fraction of sunlight the surfaces reflect, 0 to 1; blank takes the reference value, listed",
    "surface.cd": "drag coefficient; blank takes the reference value, listed",
    "magnetic.dres": "the satellite's own magnetic dipole, from a magnetic test or a budget",
    "magnetic.dunc": "how uncertain that residual dipole is",
    "flex.fmode": "lowest structural or panel mode frequency; leave blank if the satellite is rigid",
    "flex.mpart": "fraction, 0 to 1",
    "resources.palloc": "power the platform can give the ADCS",
    "resources.malloc": "mass the platform can give the ADCS",
    "resources.valloc": "volume the platform can give the ADCS",
    "resources.vbus": "voltage of the bus the ADCS is powered from",
    "resources.nif": "OBC data interfaces offered to the ADCS",
    "pointing.et": "pointing error from thermal distortion between the payload and the ADCS sensors, if known",
}

# Rows that are not tree values: they identify the case and narrow the search.
META = [
    ("schema", "Case format", "fixed; the importer refuses any other version it cannot migrate"),
    ("case_id", "Case id", "lower-case letters, digits and _; unique among the sender's cases"),
    ("title", "Title", "free text"),
    ("class", "Satellite class", "optional: a class id in catalogue/classes.toml; blank lets the solver infer it"),
    ("families", "Families to search", "optional: family ids joined by ';'; blank searches all four"),
]
CSV_COLUMNS = ["section", "key", "label", "unit", "value", "lo", "hi", "level", "note"]


# --------------------------------------------------------------------------
# Emission
# --------------------------------------------------------------------------

def flatten(node, parent, out, keymap):
    """Walk (id, label, note, children) groups and (key, label, note, extra) leaves.

    Groups keep their own id. Leaves get <group>_<n>, the positional form
    CD-06 used, and the local key maps to it for edge wiring.
    """
    gid, label, note, kids = node
    out.append([gid, label, parent, note, ""])
    keymap[gid] = gid
    n = 0
    for k in kids:
        if isinstance(k[3], list):
            flatten(k, gid, out, keymap)
        else:
            key, lab, knote, extra = k
            lid = "%s_%d" % (gid, n)
            n += 1
            if key in keymap:
                raise SystemExit("duplicate key %s" % key)
            keymap[key] = lid
            out.append([lid, lab, gid, knote, extra])


def _under(t, gid, rows):
    parent = {r[0]: r[2] for r in rows}
    while t is not None:
        if t == gid:
            return True
        t = parent.get(t)
    return False


def case_files(hn_sys, ks, hn_mgt=(), km=None):
    """plan/case_inputs.toml (the registry) and plan/case_template.csv (the
    blank form), from the supplier lists above. Both are generated, so the
    format cannot drift from the tree."""
    import csv
    import io
    label = {r[0]: r[1] for r in hn_sys}
    note = {r[0]: r[3] for r in hn_sys}
    parent = {r[0]: r[2] for r in hn_sys}
    is_group = {r[2] for r in hn_sys}
    rev = {v: k for k, v in ks.items()}
    declared = [r[0] for r in hn_sys if r[0] not in is_group and note[r[0]] == SET]
    required = [r[0] for r in hn_sys if r[0] not in is_group and note[r[0]] == REQ]
    supplier = {}

    def give(key, kind):
        tid = ks.get(key)
        if tid is None:
            raise SystemExit("supplier list names %s, which is not a layer-2 leaf" % key)
        if tid in supplier:
            raise SystemExit("%s (%s) has two suppliers: %s and %s" % (key, tid, supplier[tid], kind))
        supplier[tid] = kind

    for k in PRODUCT:
        give(k, "product")
    for k in TUNED:
        give(k, "tuned")
    for k in EVIDENCE:
        give(k, "evidence")
    for k, *_ in CASE_INPUTS:
        give(k, "case")
    for tid in required:
        supplier[tid] = "case"
    missing = [t for t in declared if t not in supplier]
    if missing:
        raise SystemExit("declared leaves with no supplier: %s" % ", ".join("%s (%s)" % (t, rev[t]) for t in missing))
    extra = [t for t in supplier if t not in declared and t not in required]
    if extra:
        raise SystemExit("suppliers named for rows that are not declared: %s" % extra)

    inputs = []
    for tid in required:
        k = rev[tid].split("_", 1)[1]
        if k not in REQ_UNITS:
            raise SystemExit("requirement %s has no unit in REQ_UNITS" % k)
        inputs.append({"key": "req." + k, "section": "req", "label": label[tid], "unit": REQ_UNITS[k],
                       "blank": "stated", "range": False, "level": True, "tree_id": tid})
    order = {r[0]: i for i, r in enumerate(hn_sys)}
    for k, sec, unit, blank, rng in sorted(CASE_INPUTS, key=lambda x: order[ks[x[0]]]):
        tid = ks[k]
        inputs.append({"key": sec + "." + k, "section": sec, "label": label[tid], "unit": unit,
                       "blank": blank, "range": rng, "level": False, "tree_id": tid})

    km = km or {}
    mlabel = {r[0]: r[1] for r in hn_mgt}
    mgroup = {r[2] for r in hn_mgt}
    mnote = {r[0]: r[3] for r in hn_mgt}
    rsk_declared = [r[0] for r in hn_mgt if r[0] not in mgroup and r[3] == SET and _under(r[0], "rsk", hn_mgt)]
    derisk = []
    for k in DERISK:
        tid = km.get(k)
        if tid is None or tid in mgroup:
            raise SystemExit("DERISK names %s, which is not a layer-1 leaf" % k)
        derisk.append(tid)
    if sorted(derisk) != sorted(rsk_declared):
        raise SystemExit("DERISK must list exactly the declared leaves of the rsk branch: %s" % sorted(set(derisk) ^ set(rsk_declared)))
    labrows = []
    for k, field, reduce_ in LAB:
        tid = km.get(k)
        if tid is None or tid in mgroup or mnote.get(tid) != SET:
            raise SystemExit("LAB names %s, which is not a declared layer-1 leaf" % k)
        labrows.append((tid, field, reduce_))

    q = lambda v: json.dumps(v, ensure_ascii=False)
    t = io.StringIO()
    t.write("# GENERATED by tools/build_tree.py from the tree and its supplier lists.\n"
            "# Do not edit by hand: edit tools/build_tree.py and re-run it.\n#\n"
            "# The case format: every key a case CSV holds, where each one\n"
            "# goes, and who supplies every other declared row. SPEC.md §8.2–§8.3;\n"
            "# the derisk supplier is the de-risking ledger, SPEC.md §5.13.\n\n")
    t.write('schema = %s\ncolumns = %s\ndefault_level = %s\n\n' % (q(CASE_SCHEMA), q(CSV_COLUMNS), DEFAULT_LEVEL))
    for k, lab, what in META:
        t.write("[[meta]]\nkey = %s\nlabel = %s\nnote = %s\n\n" % (q("meta." + k), q(lab), q(what)))
    for i in inputs:
        t.write("[[input]]\nkey = %s\nsection = %s\nlabel = %s\nunit = %s\nblank = %s\nrange = %s\nlevel = %s\ntree_id = %s\nhelp = %s\n\n"
                % (q(i["key"]), q(i["section"]), q(i["label"]), q(i["unit"]), q(i["blank"]),
                   str(i["range"]).lower(), str(i["level"]).lower(), q(i["tree_id"]), q(CASE_HELP.get(i["key"], ""))))
    for tid in sorted(supplier, key=lambda x: order[x]):
        if supplier[tid] != "case":
            t.write("[[supplier]]\ntree_id = %s\nlabel = %s\nby = %s\n\n" % (q(tid), q(label[tid]), q(supplier[tid])))
    for tid, field, reduce_ in labrows:
        t.write("[[supplier]]\ntree_id = %s\nlabel = %s\nby = %s\nfield = %s\n%s\n" % (q(tid), q(mlabel[tid]), q("lab"), q(field),
                ("reduce = %s\n" % q(reduce_)) if reduce_ else ""))
    for tid in derisk:
        t.write("[[supplier]]\ntree_id = %s\nlabel = %s\nby = %s\n\n" % (q(tid), q(mlabel[tid]), q("derisk")))

    c = io.StringIO()
    w = csv.writer(c, lineterminator="\n")
    w.writerow(CSV_COLUMNS)
    for k, lab, what in META:
        w.writerow(["meta", "meta." + k, lab, "", CASE_SCHEMA if k == "schema" else "", "", "", "", ""])
    for i in inputs:
        w.writerow([i["section"], i["key"], i["label"], i["unit"], "", "", "", "", CASE_HELP.get(i["key"], "")])
    return t.getvalue(), c.getvalue(), len(inputs)


def main():
    args = sys.argv[1:]
    if args not in ([], ["--check"]):
        print("usage: python3 tools/build_tree.py [--check]\n  (no argument writes plan/tree.json; --check compares without writing)")
        return 2
    check = args == ["--check"]
    hn_mgt, hn_sys, km, ks = [], [], {}, {}
    flatten(MGT, None, hn_mgt, km)
    flatten(SYS, None, hn_sys, ks)

    def res(k, *maps):
        for m in maps:
            if k in m:
                return m[k]
        raise SystemExit("unresolved key %s" % k)

    def leaves_of(gid, rows):
        return [r for r in rows if r[2] == gid and not any(x[2] == r[0] for x in rows)]

    shape = []
    for sid, label, grp, total in LAYER3:
        t = len(leaves_of(grp, hn_sys))
        need = 1 + 2 * t
        if total < need:
            raise SystemExit("%s: %d rows cannot hold an interface and %d target pairs" % (sid, total, t))
        shape.append({"id": sid, "label": label, "group": grp, "targets": t, "nodes": total})

    tree = {
        "_source": "ADCS platform plan, tools/build_tree.py — the ADCS equivalent of VLEO_SIMULATOR cd06/tree.json",
        "_note": "Layers 1 and 2 are named in full. Layer 3 is shape only, as in CD-06: interface, one required and one achieved row per layer-2 target, and the rest to be named. The 5th field of each row names the configuration families it is in play for; empty means all.",
        "layer3_shape": shape,
        "HN_MGT": hn_mgt,
        "ED_MGT": [[res(a, km), res(b, km), w] for a, b, w in ED_MGT],
        "HN_SYS": hn_sys,
        "ED_SYS": [[res(a, ks), res(b, ks), w] for a, b, w in ED_SYS],
        "VE": [[res(a, ks, km), res(b, ks, km), w] for a, b, w in VE],
        "KE": [[res(a, ks), res(b, ks), w] for a, b, w in KE],
    }
    inputs_text, template_text, n_inputs = case_files(hn_sys, ks, hn_mgt, km)
    outputs = [
        (os.path.join(ROOT, "plan", "tree.json"), json.dumps(tree, ensure_ascii=False, indent=1) + "\n"),
        (os.path.join(ROOT, "plan", "case_inputs.toml"), inputs_text),
        (os.path.join(ROOT, "plan", "case_template.csv"), template_text),
    ]
    if check:
        stale = [os.path.relpath(p, ROOT) for p, text in outputs
                 if not (os.path.exists(p) and open(p, encoding="utf-8").read() == text)]
        if stale:
            print("STALE: %s — run python3 tools/build_tree.py" % ", ".join(stale))
            return 1
        print("plan/tree.json, plan/case_inputs.toml and plan/case_template.csv are current")
        return 0
    for p, text in outputs:
        with open(p, "w", encoding="utf-8") as f:
            f.write(text)
    print("wrote plan/tree.json: %d management rows, %d system rows, %d subsystem layers, %d/%d/%d/%d edges"
          % (len(hn_mgt), len(hn_sys), len(shape),
             len(tree["ED_MGT"]), len(tree["ED_SYS"]), len(tree["VE"]), len(tree["KE"])))
    print("wrote plan/case_inputs.toml and plan/case_template.csv: %d case inputs + %d meta rows"
          % (n_inputs, len(META)))
    return 0

if __name__ == "__main__":
    sys.exit(main())
