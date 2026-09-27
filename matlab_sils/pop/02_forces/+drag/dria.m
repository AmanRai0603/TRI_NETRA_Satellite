function [cp, ct] = dria(s, delta, nO, T, Tw)
%DRAG.DRIA  Diffuse Reflection, Incomplete Accommodation (Mehta 2017). This is
%   Sentman's diffuse model driven by the physical (composition-based) energy
%   accommodation from SESAM - the workhorse for 200-400 km. nO atomic-oxygen
%   number density [m^-3], T ambient [K], Tw wall [K].
    aT = drag.sesam(nO, T);
    [cp, ct] = drag.sentman(s, delta, aT, Tw, T);
end
