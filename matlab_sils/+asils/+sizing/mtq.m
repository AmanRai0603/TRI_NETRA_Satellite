function [pm, pmp] = mtq(Dm, k)
%ASILS.SIZING.MTQ  The coil of every family and the pointing-grade coil of the coils-only family (design/sizemtq.pc:
%   sizemtq's mtq_dipoles, mtq_coil, mtq_dispersion, mtq_pointing_dipole; the engine's adcs-design mtq).
%   [pm, pmp] = asils.sizing.mtq(asils.sizing.demand('cases/ais_3u.csv'), asils.sizing.knobs())
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    [m_dump, m_mom, m_det, m] = asils.models.sizemtq.mtq_dipoles(Dm.tau_dist, Dm.B_min, Dm.h_secular, Dm.B_mean, Dm.orbit_period_s, ...
        Dm.h_detumble, Dm.req.detumble);
    pm = coil_(Dm, k, m*asils.sizing.scale_(k, 'mtq'), 'MTQ', 'Sized magnetorquer coil (our product)', m_dump, m_mom, m_det);
    pmp = coil_(Dm, k, asils.models.sizemtq.mtq_pointing_dipole(m, m_dump)*asils.sizing.scale_(k, 'mtqp'), 'MTQP', ...
        'Sized pointing-grade magnetorquer coil (our product, coils-only family)', m_dump, m_mom, m_det);
end

function p = coil_(Dm, k, m, tag, name, m_dump, m_mom, m_det)
    [dipole_max, per_amp, current_max, resistance, time_constant, mass, power, volume] = asils.models.sizemtq.mtq_coil(m);
    [sd, sm, ss, md, mm, ms] = asils.models.sizemtq.mtq_dispersion();
    p = asils.sizing.part_(Dm.case, tag, 'coil_tile', name, 'in-house');
    p.nominal = struct('dipole_max_Am2', dipole_max, 'dipole_per_amp_Am2_per_A', per_amp, 'current_max_A', current_max, ...
        'resistance_ohm', resistance, 'time_constant_s', time_constant, 'mass_kg', mass, 'power_at_max_W', power, 'volume_L', volume);
    p.dispersion = struct('dipole_scale', asils.sizing.spread_(sd, sm, ss), 'axis_misalignment_rad', asils.sizing.spread_(md, mm, ms));
    if strcmp(tag, 'MTQ'), w = 'mtq'; else, w = 'mtqp'; end
    p.sizing = struct('m_dump_Am2', m_dump, 'm_momentum_Am2', m_mom, 'm_detumble_Am2', m_det, 'scale', asils.sizing.scale_(k, w), ...
        'law', 'max(dumping, momentum, detumble) dipole; mass and power linear in the dipole (SYN-CT-1 anchor)');
end
