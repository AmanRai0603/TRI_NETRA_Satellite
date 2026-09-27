function phiQ = phiQuad(a_sat, GM_body, r_body, e_body, mu)
%THIRDBODY.SECULAR.PHIQUAD  Secular quadrupole frequency [rad/s].
%   phiQ = (3/4)(GM_body/r_body^3)/n_sat /(1-e_body^2)^{3/2}
%   a_sat [m]; GM_body [m^3/s^2]; r_body = perturber distance [m]; e_body its ecc.
    if nargin < 4 || isempty(e_body), e_body = 0; end
    if nargin < 5 || isempty(mu),     Kc = de440.constants(); mu = Kc.mu_earth; end   % no chained-call indexing
    n = sqrt(mu/a_sat^3);
    phiQ = 0.75*(GM_body/r_body^3)/n/(1-e_body^2)^1.5;
end
