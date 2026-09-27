function [cp, ct] = cll(s, delta, sig_n, sig_t, Tw, Talt)
%DRAG.CLL  Cercignani-Lampis-Lord / Schaaf-Chambre form: independent NORMAL
%   (sig_n) and TANGENTIAL (sig_t) momentum accommodation - captures quasi-
%   specular re-emission (drag-reducing materials). Reduces exactly to Sentman
%   (full energy accommodation) at sig_n=sig_t=1; specular at sig=0.
%     cp = (2-sig_n) Ip + sig_n Dp ;  ct = sig_t * St
    c=cos(delta); sn=s.*c; E=1+erf(sn); P=exp(-sn.^2); Vr=sqrt(Tw./Talt);
    Ip = c./(sqrt(pi)*s).*P + (1./(2*s.^2)+c.^2).*E;             % incident normal
    Dp = (Vr./(2*s)).*(sqrt(pi)*c.*E + (1./s).*P);              % diffuse re-emit
    St = sin(delta)./(sqrt(pi)*s).*(P + sqrt(pi)*sn.*E);        % shear
    cp = (2-sig_n).*Ip + sig_n.*Dp;
    ct = sig_t.*St;
end
