function [dj, de] = kozaiRates(j, e, nhat, phiQ)
%THIRDBODY.SECULAR.KOZAIRATES  Vectorial doubly-averaged quadrupole rates for
%   one perturber. j = (1-e^2)^{1/2} * orbit-normal (|j|=sqrt(1-e^2)),
%   e = eccentricity vector (|e|=ecc), nhat = perturber orbit pole (unit),
%   phiQ = secular quadrupole frequency [rad/s] (thirdbody.secular.phiQuad).
%   (Katz, Dong & Malhotra 2011; Tremaine & Yavetz 2014.)
    j = j(:); e = e(:); nhat = nhat(:)/norm(nhat);
    jn = dot(j,nhat); en = dot(e,nhat);
    dj = phiQ*( jn*cross(j,nhat) - 5*en*cross(e,nhat) );
    de = phiQ*( jn*cross(e,nhat) + 2*cross(j,e) - 5*en*cross(j,nhat) );
end
