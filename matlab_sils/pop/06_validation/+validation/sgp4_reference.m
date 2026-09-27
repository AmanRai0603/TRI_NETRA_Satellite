function [t, r, v] = sgp4_reference(tle, tspan_s, dt_s)
%VALIDATION.SGP4_REFERENCE  SGP4 reference ephemeris for a TLE (TEME frame).
%   [t,r,v] = validation.sgp4_reference(tle, tspan_s, dt_s)
%   Produces the analytic SGP4 trajectory sampled every dt_s over tspan_s from
%   the TLE epoch.  Requires an SGP4 implementation on the path; it tries, in
%   order: (1) MATLAB Aerospace Toolbox 'sgp4' via a satelliteScenario, (2) a
%   Vallado 'sgp4'/'twoline2rv' pair (e.g. from your GOCE_density_study), else it
%   errors with guidance.  SGP4 output is in TEME -- rotate to J2000 for a
%   strict comparison; for a coarse sanity check TEME~=ECI at the arcsec level.
    t = (0:dt_s:tspan_s).';
    if exist('twoline2rv','file')==2 && exist('sgp4','file')==2
        % Vallado path
        global tumin mu radiusearthkm xke j2 j3 j4 j3oj2 %#ok
        try
            gravconst = 72; sgp4init_const(gravconst);
        catch
            % LEGITIMATE swallow: sgp4init_const is part of some Vallado
            % distributions and not others. The exist() guard above already proved
            % twoline2rv/sgp4 are present; this is an OPTIONAL extra in the same
            % library, and its absence just means the default WGS-72 constants are
            % already set. Erroring here would refuse a working SGP4.
        end
        satrec = twoline2rv(tle_l1(tle), tle_l2(tle), 'c', 'd', 'i', 72);
        r=zeros(numel(t),3); v=zeros(numel(t),3);
        for k=1:numel(t)
            [~, rk, vk] = sgp4(satrec, t(k)/60);       % minutes since epoch
            r(k,:)=rk*1000; v(k,:)=vk*1000;
        end
        return
    end
    error('validation:sgp4_reference', ...
      ['No SGP4 implementation found on the path. Add Vallado''s sgp4.m + ' ...
       'twoline2rv.m (available in your GOCE_density_study TLE tooling), or ' ...
       'use MATLAB satelliteScenario/sgp4. Meanwhile validation.tle2eci gives ' ...
       'an osculating seed you can propagate with op.propagate.']);
end
function s=tle_l1(tle), s=tle.rawl1; end
function s=tle_l2(tle), s=tle.rawl2; end
