function [cp, ct] = sentman(s, delta, aT, Tw, Talt)
%DRAG.SENTMAN  Sentman (1961) flat-panel free-molecular coefficients: normal
%   pressure cp and tangential shear ct. Diffuse re-emission, incomplete ENERGY
%   accommodation aT. s speed ratio; delta = angle between panel outward normal
%   and the incoming flow (v_rel) [rad]; Tw wall, Talt ambient temperature [K].
%   Force/area = q_d*(ct*tgas - cp*n),  q_d = 0.5*rho*Vrel^2.  (Eqs. 26a/26b.)
    c=cos(delta); sn=s.*c; Ti=(2/3)*s.^2.*Talt;
    E=1+erf(sn); P=exp(-sn.^2);
    cp = c./(sqrt(pi)*s).*P + (1./(2*s.^2)+c.^2).*E ...
       + 0.5*sqrt((2/3)*(1+aT.*(Tw./Ti-1))).*(sqrt(pi)*c.*E + (1./s).*P);
    ct = sin(delta)./(sqrt(pi)*s).*(P + sqrt(pi)*sn.*E);
end
