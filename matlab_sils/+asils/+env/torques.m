function [tau, parts] = torques(q, r_eci, v_rel_eci, B_eci, sun_rel_eci, nu, P_srp, rho, I, G, m_res, mu, on)
%ASILS.ENV.TORQUES  Environmental disturbance torques [N m], body frame.
%
%   Every input that describes the world comes from the in-loop precision
%   orbit (asils.orbit): r and the atmosphere-relative velocity from the POP
%   state, rho from the POP drag model (DTM2020), the Sun and P_srp from
%   DE440, B from IGRF at the POP position, nu from the conical shadow.
%
%   Models (upgrade of the Standard Code env.disturbances, which used one
%   drag/SRP area at a fixed CP-CM offset and the J2-propagated state):
%     gg   gravity gradient   3 mu/|r|^5 (r_B x I r_B)
%     aero free-molecular flat-plate per facet (Schaaf & Chambre; Hughes 1986
%          eq. 8.34): normal and tangential momentum accommodation sigma_n,
%          sigma_t, re-emission speed ratio vb/v; torque about the CM
%     srp  per facet with specular/diffuse reflection (Wertz 1978, eq. 17-6),
%          scaled by the conical-shadow factor nu; plus the Earth's albedo and
%          infrared on the same facets, arriving from nadir (asils.env.earth_pressure)
%     mag  residual dipole  m_res x B_B
%   on = [gg aero srp mag] switches (logical).
    R = asils.quat.dcm(q);
    rB = R*r_eci;
    tau_gg = zeros(3,1); tau_aero = zeros(3,1); tau_srp = zeros(3,1); tau_mag = zeros(3,1);
    if on(1)
        rn = sqrt(rB'*rB);
        Ir = I*rB;
        tau_gg = 3*mu/rn^5*[rB(2)*Ir(3) - rB(3)*Ir(2); rB(3)*Ir(1) - rB(1)*Ir(3); rB(1)*Ir(2) - rB(2)*Ir(1)];
    end
    if on(2) && rho > 0
        vB = R*v_rel_eci; V = sqrt(vB'*vB); vh = vB/V;
        c = vh'*G.n;                       % cos(angle) between flow direction and facet normal
        k = find(c > 0);
        for j = k
            cj = c(j); nj = G.n(:,j);
            F = -rho*V^2*G.A(j)*cj*( G.sigma_t*vh + (G.sigma_n*G.vb_ratio + (2 - G.sigma_n - G.sigma_t)*cj)*nj );
            p = G.rho(:,j);
            tau_aero = tau_aero + [p(2)*F(3)-p(3)*F(2); p(3)*F(1)-p(1)*F(3); p(1)*F(2)-p(2)*F(1)];
        end
    end
    if on(3)
        % radiation: the Sun, and the Earth's reflected (albedo) and emitted (IR) light
        if nu > 0
            sB = R*sun_rel_eci; sB = sB/sqrt(sB'*sB);
            tau_srp = radiation_(G, sB, nu*P_srp);
        end
        [p_alb, p_ir] = asils.env.earth_pressure(r_eci, sun_rel_eci, P_srp);
        eB = -rB/sqrt(rB'*rB);
        tau_srp = tau_srp + radiation_(G, eB, p_alb + p_ir);
    end
    if on(4)
        bB = R*B_eci;
        tau_mag = [m_res(2)*bB(3)-m_res(3)*bB(2); m_res(3)*bB(1)-m_res(1)*bB(3); m_res(1)*bB(2)-m_res(2)*bB(1)];
    end
    tau = tau_gg + tau_aero + tau_srp + tau_mag;
    if nargout > 1
        parts = [tau_gg, tau_aero, tau_srp, tau_mag];
    end
end

function tau = radiation_(G, sB, P)
%RADIATION_  Torque of light of pressure P arriving from body direction sB (unit,
%   towards the source) on the lit facets (Wertz 1978, eq. 17-6).
    tau = zeros(3,1);
    c = sB'*G.n;
    for j = find(c > 0)
        cj = c(j); nj = G.n(:,j);
        F = -P*G.A(j)*cj*( (1 - G.rho_spec)*sB + 2*(G.rho_spec*cj + G.rho_diff/3)*nj );
        p = G.rho(:,j);
        tau = tau + [p(2)*F(3)-p(3)*F(2); p(3)*F(1)-p(1)*F(3); p(1)*F(2)-p(2)*F(1)];
    end
end
