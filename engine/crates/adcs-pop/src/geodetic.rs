//! ECEF -> geodetic: env's method env_geodetic (the port of `matlab_sils/pop/01_core/+op/geodetic.m`: a Bowring-style
//! fixed 5-iteration solution on WGS84, `de440.constants`' `Re_earth` and `f_earth`), generated from the design into
//! `gen::geodesy` (tools/engine_build.py). POP has no inverse (geodetic -> ECEF).
use crate::la::V3;

/// `[lat, lon, alt] = op.geodetic(r_ecef)`: WGS84 geodetic latitude, longitude (rad) and altitude (m).
pub fn geodetic(r_ecef: &V3) -> (f64, f64, f64) { crate::gen::geodesy::geodetic_wgs84(*r_ecef) }

/// `[lat, lon, alt] = op.geodetic(r_ecef, Re, f)` with an explicit ellipsoid.
pub fn geodetic_with(r_ecef: &V3, re: f64, f: f64) -> (f64, f64, f64) { crate::gen::geodesy::geodetic_with(*r_ecef, re, f) }
