//! The language's random streams are the engine's (adcs-sim-core rng.rs, the toolbox): a stream of a seed and an
//! id, or of a seed and a name, draws in the interpreter the very numbers `Rng` draws, uniform and normal, bit for
//! bit, the spare of a normal draw kept as `Rng` keeps it. The translations are held to the interpreter on the
//! language's self-test (tools/translators.py: uniform draws bit for bit, normal ones within 1e-12, their sin and
//! cos being each platform's).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use adcs_sim_core::rng::{stream_id, Rng};
use trinetra_pcode::{compile, Value};

const SRC: &str = "\
fn draws(seed: int, id: int) -> (u: real[1][40], z: real[1][41], v: vec3[1])
    let g = stream(seed, id)
    for k in 0 .. 40
        u[k] = uniform(g)
    end
    for k in 0 .. 41
        z[k] = normal(g)
    end
    v = normal3(g)
end
fn named(seed: int) -> (a: real[1], b: real[1], c: real[1])
    let g = stream(seed, \"gyro\")
    a = uniform(g)
    b = normal(g)
    let h = stream(seed, \"star tracker\")
    c = uniform(h)
end
";

fn nums(v: &Value) -> Vec<f64> {
    v.flatten()
}

#[test]
fn a_stream_draws_what_rng_rs_draws_bit_for_bit() {
    let p = compile(&[("streams.pc", SRC)]).unwrap_or_else(|e| panic!("{e:?}"));
    let mut n = 0;
    for (seed, id) in [(0i64, 0u64), (1, 1), (42, 3), (7, 19), (123456, 5), (9007199254740991, 2), (-1, 4), (-123, 11)] {
        let out = p.call("draws", &[Value::Num(seed as f64), Value::Num(id as f64)]).unwrap();
        let mut r = Rng::new(seed as u64, id);
        for (k, u) in nums(&out[0]).iter().enumerate() {
            assert_eq!(u.to_bits(), r.uniform().to_bits(), "seed {seed} id {id}: uniform draw {k}");
            n += 1;
        }
        // 41 normals: the last one's spare is taken by normal3's first
        for (k, z) in nums(&out[1]).iter().enumerate() {
            assert_eq!(z.to_bits(), r.normal().to_bits(), "seed {seed} id {id}: normal draw {k}");
            n += 1;
        }
        let w = r.normal3();
        for (k, z) in nums(&out[2]).iter().enumerate() {
            assert_eq!(z.to_bits(), w[k].to_bits(), "seed {seed} id {id}: normal3 draw {k}");
            n += 1;
        }
    }
    for seed in [0i64, 5, 42, 1000003] {
        let out = p.call("named", &[Value::Num(seed as f64)]).unwrap();
        let mut g = Rng::new(seed as u64, stream_id("gyro"));
        let mut h = Rng::new(seed as u64, stream_id("star tracker"));
        assert_eq!(nums(&out[0])[0].to_bits(), g.uniform().to_bits(), "seed {seed}: the gyro stream");
        assert_eq!(nums(&out[1])[0].to_bits(), g.normal().to_bits(), "seed {seed}: the gyro stream's normal");
        assert_eq!(nums(&out[2])[0].to_bits(), h.uniform().to_bits(), "seed {seed}: the star tracker stream");
        n += 3;
    }
    println!("{n} draws, each rng.rs's bit for bit");
}
