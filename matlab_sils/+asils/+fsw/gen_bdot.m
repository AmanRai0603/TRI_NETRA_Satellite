function m0 = gen_bdot(B, Bdot, w_d, k)
%ASILS.FSW.GEN_BDOT  L1 generalised B-dot (Cubas, Farrahi & Pindado, JGCD 2015).
%   PORTED from Standard Code ctrl.genBdot (theory doc sec. 4.1, eq 4.1):
%       m0 = -k (Bdot + w_d x B)
%   w_d = 0 is plain B-dot (detumble); w_d = sigma w_s e_z drives the body rate
%   to a spin about +Z_B (spin-up). Bdot is the field derivative the
%   MAGNETOMETER sees (finite difference of the coil-off averages) -- never
%   -w x B from the gyro, which would erase the inertial field rotation the
%   law relies on. Unsaturated: the caller saturates once (act.saturateDipole).
    m0 = -k*(Bdot + asils.util.cross3(w_d, B));
end
