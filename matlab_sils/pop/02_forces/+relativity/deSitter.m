function a = deSitter(rSat, vSat, earthHelioPos, earthHelioVel, gamma)
%RELATIVITY.DESITTER  Geodesic (de Sitter) precession from Earth orbiting Sun.
%   a = (1+2 gamma) [ v_E x (-GM_sun R_E/(c^2 |R_E|^3)) ] x v_sat
%   R_E, v_E = Earth position/velocity w.r.t. Sun (ephemInputs.earth_helio_*).
    if nargin<5||isempty(gamma), gamma=1; end
    % NOT de440.constants().GM_sun -- chained indexing on a function call is
    % illegal in Octave and in MATLAB before R2019b. This line fires on EVERY call
    % (no nargin guard), so the de Sitter term could never have been executed here.
    c=299792458.0; Kc=de440.constants(); GMs=Kc.GM_sun;
    v=vSat(:); RE=earthHelioPos(:); vE=earthHelioVel(:);
    aE=-GMs*RE/(c^2*norm(RE)^3);            % solar accel of Earth / c^2
    a=(1+2*gamma)*cross(cross(vE,aE), v);
end
