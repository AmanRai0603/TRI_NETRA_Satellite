function [pos, vel] = state(center, target, jdTDB, eph)
%DE440.STATE  Position/velocity of TARGET relative to CENTER at a TDB Julian date.
%   [pos,vel] = de440.state(center,target,jdTDB)        (km, km/s; ICRF)
%   [pos,vel] = de440.state(center,target,jdTDB,eph)    reuse a loaded kernel
%
%   Body codes (this kernel): 0 SSB, 10 Sun, 3 EMB, 399 Earth, 301 Moon,
%   1..9 planet barycenters. Segments present: see de440.open output .keys.
%   jdTDB is Barycentric Dynamical Time. TT may be passed (diff < 1.7 ms).
    if nargin < 4 || isempty(eph); eph = de440.open(); end
    et = (jdTDB - 2451545.0) * 86400.0;                 % seconds past J2000 TDB
    key = sprintf('%d_%d', center, target);
    idx = find(strcmp(eph.keys, key), 1);
    assert(~isempty(idx), 'de440:state', 'segment %d->%d not in kernel', center, target);
    sa = eph.seg(idx,1); ea = eph.seg(idx,2);

    foot   = eph.D(ea-3:ea);                             % INIT, INTLEN, RSIZE, N
    init   = foot(1); intlen = foot(2); rsize = foot(3); n = foot(4);
    ncoef  = (rsize - 2) / 3;
    k = floor((et - init) / intlen);
    k = min(max(k, 0), n-1);
    rec = eph.D(sa + k*rsize : sa + k*rsize + rsize - 1);
    mid = rec(1); radius = rec(2);
    tau = (et - mid) / radius;
    C   = reshape(rec(3:end), ncoef, 3);                 % columns: X, Y, Z coeffs

    [px,vx] = de440.chebval(C(:,1), tau, radius);
    [py,vy] = de440.chebval(C(:,2), tau, radius);
    [pz,vz] = de440.chebval(C(:,3), tau, radius);
    pos = [px; py; pz];                                  % km
    vel = [vx; vy; vz];                                  % km/s
end
