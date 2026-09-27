function m = bdot(b1, b2, dt, Bnorm, k, m_max)
%ASILS.FSW.BDOT  B-dot detumble dipole (catalogue algorithm 'bdot', Avanzini & Giulietti 2012).
%   m = -(k/|B|) * d(b)/dt,  b = B/|B|, the derivative between the averaged
%   clean field directions of two consecutive coil-off windows, dt apart
%   (the measure-then-drive duty of the Standard Code ctrl.bdotScheduler). k = gain_scale * 2 n (1 + sin xi_m) J_min
%   is computed at mode entry (asils.config). Saturation is DIRECTION-
%   PRESERVING, ported from Standard Code act.saturateDipole (theory doc 3.1).
    bd = (b2/norm(b2) - b1/norm(b1))/dt;
    m0 = -(k/Bnorm)*bd;
    a = min(1, min(m_max ./ max(abs(m0), 1e-30)));
    m = a*m0;
end
