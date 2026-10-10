function [a, C, E, rho] = accel(W, t, r, v)
%ASILS.ORBIT.ACCEL  The precision orbit's acceleration [m/s^2] at t [s after the epoch], r, v (ECI): POP's op.accel as
%   the engine's adcs-pop World::accel makes it, every model env's (+asils/+models): the time scales at the UTC instant
%   (timescales.addsec, convert_utc), the ephemeris (asils.orbit.ephem), the Earth-fixed frame (earthframes'
%   GMST build), then the forces in op.accel's order, summed by forcemodel.force_sum: gravity (gravity.sph_accel in
%   the Earth-fixed frame, or two_body), the Sun and the Moon (thirdbody.tb_total), drag (geodesy, drag.probe_geo, the
%   density densitymodel.atmos_density with the run's indices, drag.drag_force), solar pressure (srp.srp_force), the
%   empirical acceleration. Also the frame C (r_ecef = C r), the ephemeris E ([] when no force reads it) and the
%   density rho (0 without drag).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    fs = W.fs; z = zeros(3, 1);
    utc = asils.models.timescales.addsec(W.epoch, t);
    T = asils.models.timescales.convert_utc(utc(1), utc(2), utc(3), utc(4), utc(5), utc(6), 0);
    E = [];
    if W.need_ephem, E = asils.orbit.ephem(T.tdb_jd); end
    C = asils.models.earthframes.eci2ecef_gmst(T.gmst_rad, 0, 0);

    if fs.grav_model == 0                                   % two-body
        ag = asils.models.gravity.two_body(r, W.gm);
    elseif fs.grav_model == 2                               % spherical harmonics, in the Earth-fixed frame
        ae = asils.models.gravity.sph_accel(asils.models.gravity.to_ecef(C, r), W.gm, W.re, W.g_nws, W.g_n, W.g_m, ...
            W.g_c, W.g_s, W.g_f1, W.g_f2, W.g_v, W.g_w);
        ag = asils.models.gravity.from_ecef(C, ae);
    else
        error('asils:orbit', 'gravity model %d: the twin''s orbit flies two-body or the spherical harmonics', fs.grav_model);
    end
    atb = z;
    if fs.tb_on
        atb = asils.models.thirdbody.tb_total(r, E.sun_eci, E.gm_sun, E.moon_eci, E.gm_moon, fs.tb_model, fs.tb_nmax);
    end
    adr = z; rho = 0;
    if fs.drag_on
        [lat, lon, alt] = asils.models.geodesy.geodetic_wgs84(asils.la.mv(C, r));
        [alt_km, lat_deg, lon_deg, lst_h] = asils.models.drag.probe_geo(lat, lon, alt, utc);
        % the run's indices (drivers of kind 1, the space weather); the research and JB2008 drivers are files it has not
        [st, atm] = asils.models.densitymodel.atmos_density(fs.drag_atmos, alt_km, lat_deg, lon_deg, lst_h, T.doy, utc, 1, ...
            W.sw.f107, W.sw.f107a, W.sw.akp, NaN, NaN, NaN, false, false, 0);
        if st ~= 0, error('asils:orbit', 'density model %d at t = %g s: status %d', fs.drag_atmos, t, st); end
        sun = z; if ~isempty(E), sun = E.sun_eci; end
        [st, ~, adr] = asils.models.drag.drag_force(fs.drag_model, fs.drag_panel, r, v, atm, true, W.omega_eci, fs.drag_corotate, ...
            fs.has_drag_cd, fs.drag_cd, true, W.sc.cd, W.sc.aref, W.sc.mass, W.sc.R_bi, asils.models.srp.ScFacets_zero(), ...
            fs.drag_gsi, ~isempty(E), sun);
        if st ~= 0, error('asils:orbit', 'drag at t = %g s: status %d (the twin''s spacecraft has no facets)', t, st); end
        rho = atm.rho;
    end
    asr = z;
    if fs.srp_on
        cr = asils.models.srp.srp_cr(fs.has_srp_cr, fs.srp_cr, true, W.sc.cr);
        asr = asils.models.srp.srp_force(r, E.sun_eci, E.p_srp, fs.srp_model, fs.srp_eclipse, cr, W.sc.aref, W.sc.mass, ...
            W.sc.R_bi, asils.models.srp.ScFacets_zero());
    end
    aem = z;
    if fs.has_empirical, aem = asils.models.forcemodel.empirical_accel(r, v, fs.empirical); end
    a = asils.models.forcemodel.force_sum(ag, atb, adr, asr, z, z, z, z, aem);
end
