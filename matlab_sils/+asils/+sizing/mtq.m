function [p, pp] = mtq(Dm, caseId)
%ASILS.SIZING.MTQ  Magnetorquer (OUR product, every family): one coil per axis.
%   The dipole is the largest of
%     dumping    2 x tau_dist / (0.5 B_min)      (the coil must out-torque the
%                disturbance with half the field usable, x2 margin)
%     momentum   2 x h_secular / (0.3 B_mean T_orbit)   (dump an orbit's
%                secular momentum within the orbit)
%     detumble   h_detumble / (0.3 B_mean 0.5 T_detumble)  (half the allowed
%                detumble time, case req.detumble, 3 orbits when blank)
%   floored at 0.05 A m^2. Coil: at a fixed current density and loop area the
%   dipole, the copper mass and the resistive power all scale with the turns:
%   mass and power linear in the dipole. Anchor: SYN-CT-1 (0.45 A m^2, 0.3 W,
%   30 g). No jitter (no moving parts).
%   pp: the POINTING-grade coil of the magnetorquers-only family, where the
%   coils hold the attitude against the disturbance at every point of the
%   orbit: the dumping term with x4 margin instead of x2.
    Td = Dm.req.detumble; if isnan(Td), Td = 3*Dm.orbit_period_s/60; end
    m_dump = 2*Dm.tau_dist/(0.5*Dm.B_min);
    m_mom = 2*Dm.h_secular/(0.3*Dm.B_mean*Dm.orbit_period_s);
    m_det = Dm.h_detumble/(0.3*Dm.B_mean*0.5*Td*60);
    m = max([m_dump, m_mom, m_det, 0.05]);
    p = coil_(m, caseId, 'MTQ', m_dump, m_mom, m_det);
    pp = coil_(max(m, 2*m_dump), caseId, 'MTQP', m_dump, m_mom, m_det);
    pp.name = ['Sized pointing-grade magnetorquer coil (our product, coils-only family) — ' caseId];
end

function p = coil_(m, caseId, tag, m_dump, m_mom, m_det)
    k = m/0.45;
    p = struct('part_number', sprintf('SZ-%s-%s', caseId, tag), 'kind', 'coil_tile', ...
        'name', ['Sized magnetorquer coil (our product) — ' caseId], 'status', 'sized', 'source', 'asils.sizing', ...
        'made', 'in-house', 'descriptor_version', 1);
    p.nominal = struct('dipole_max_Am2', m, 'dipole_per_amp_Am2_per_A', 4.5, 'current_max_A', m/4.5, ...
        'resistance_ohm', 30, 'time_constant_s', 0.005, 'mass_kg', 0.03*k, 'power_at_max_W', 0.3*k, ...
        'volume_L', 0.012*k);
    p.dispersion = struct('dipole_scale', struct('dist', 'normal', 'mean', 1, 'sigma', 0.02), ...
        'axis_misalignment_rad', struct('dist', 'normal', 'mean', 0, 'sigma', 0.005));
    p.sizing = struct('m_dump_Am2', m_dump, 'm_momentum_Am2', m_mom, 'm_detumble_Am2', m_det, ...
        'law', 'max(dumping, momentum, detumble) dipole; mass and power linear in the dipole (SYN-CT-1 anchor)');
end
