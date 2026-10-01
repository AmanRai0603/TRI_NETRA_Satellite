function [hdot, gdot, P_W, D] = mex(cmd_r, cmd_g, h, d, D, m, dt)
%ASILS.DEVICES.MEX  Momentum-exchange devices: command -> actual rotor hdot,
%   gimbal rate and electrical power. One entry per rotor (m.kind{i}):
%
%   'rw'    reaction wheel (SYN-RW-10): motor torque limit, momentum limit,
%           Coulomb + viscous friction of which the driver cancels
%           m.friction_comp, torque noise, scale error. Power = steady + |tau w|/eta.
%           B3.5: the motor's back-EMF torque-speed line and the drive's speed
%           limit (asils.devices.wheel_motor); stiction: the Stribeck excess
%           (Fs - Fc) e^-(w/ws)^2 sgn w, and a rotor at rest (|h| <= Fs dt)
%           stays at rest while |demand| <= Fs (Karnopp).
%   'vscmg' rotor: the wheel model without the B3.5 motor and stiction terms.
%   'fmr'   fluid momentum ring (SYN-MFP-1, IDMAS): galinstan loop driven by a
%           conduction pump. h = rho*Ac*2S*v. Laminar loss hdot_loss = -h/T_sd,
%           T_sd = rho d^2/(32 mu) (0.75 s). The driver adds the loss it
%           estimates from its filtered flow sensor and servoes the flow to the
%           integrated command (closed-loop driver), limited by the pump; |v| <= v_max.
%           Power = pump pressure x flow / pump efficiency, plus the pump
%           field power while pumping (m.field_power).
%   'cmg'   control-moment-gyro rotor: constant momentum h0 held by a speed loop;
%           the torque comes from its gimbal (rate limit m.gimbal_rate_max).
%   'vscmg' variable-speed CMG: a CMG whose rotor is also torqued like a wheel.
%   cmd_r   commanded rotor hdot [N m] (for cmg: ignored)
%   cmd_g   commanded gimbal rates [rad/s]
    n = numel(h); hdot = zeros(n,1); P_W = 0;
    for i = 1:n
        if D.failed(i), hdot(i) = -m.viscous(i)*h(i)/m.J(i) - m.coulomb(i)*sign(h(i)); continue, end
        switch m.kind{i}
            case 'rw'
                om = h(i)/m.J(i);
                fr = (m.coulomb(i)*sign(om) + m.viscous(i)*om) * D.fscale(i);
                nz = m.torque_noise*m.torque_max(i)*randn;
                tc = asils.devices.wheel_motor(m, i, max(-m.torque_max(i), min(m.torque_max(i), cmd_r(i))) * D.tscale(i), om);
                % breakaway excess over Coulomb near zero speed (Stribeck); the
                % driver's compensation knows only the Coulomb + viscous model
                fs = m.f_static(i)*D.fscale(i);
                x = om/m.w_stribeck(i);
                st = (m.f_static(i) - m.coulomb(i))*D.fscale(i)*exp(-x*x)*sign(om);
                if abs(h(i)) <= fs*dt && abs(tc + nz) <= fs
                    hdot(i) = -h(i)/dt;          % stuck (Karnopp): static friction holds it at rest
                else
                    hdot(i) = tc - (1 - m.friction_comp)*fr - st + nz;
                end
                if abs(h(i)) >= m.h_max(i) && sign(hdot(i)) == sign(h(i)), hdot(i) = -(1 - m.friction_comp)*fr; end
                P_W = P_W + m.p_steady(i) + abs(tc*om)/m.eta;
            case 'vscmg'
                tc = max(-m.torque_max(i), min(m.torque_max(i), cmd_r(i))) * D.tscale(i);
                om = h(i)/m.J(i);
                fr = (m.coulomb(i)*sign(om) + m.viscous(i)*om) * D.fscale(i);
                hdot(i) = tc - (1 - m.friction_comp)*fr + m.torque_noise*m.torque_max(i)*randn;
                if abs(h(i)) >= m.h_max(i) && sign(hdot(i)) == sign(h(i)), hdot(i) = -(1 - m.friction_comp)*fr; end
                P_W = P_W + m.p_steady(i) + abs(tc*om)/m.eta;
            case 'fmr'
                % Closed-loop flow driver: integrates the commanded momentum,
                % filters its flow sensor, cancels the MODELLED laminar loss
                % and servoes the measured flow to the target (gain k_flow),
                % so a loss model 20 % wrong does not leave a torque error
                % proportional to the stored momentum.
                Tsd = m.T_sd(i);
                [pump, D] = asils.comp.fluid_loop.drive(i, cmd_r(i), h(i), D, m, dt);   % unit firmware
                hdot(i) = pump - h(i)/Tsd*D.fscale(i);
                if abs(h(i)) >= m.h_max(i) && sign(hdot(i)) == sign(h(i))
                    hdot(i) = 0;                                    % flow held at v_max
                end
                v = h(i)/m.k_hv(i);                                 % flow speed [m/s]
                dp = pump*m.l(i)/(2*m.S(i)*m.Ac(i));                % pump pressure [Pa]
                P_W = P_W + abs(dp*m.Ac(i)*v)/D.eta(i);
                % pump field (yoke electromagnet) is on while the pump drives the
                % loop (IDMAS v2 §03C: 1-3 W per unit); the loop spins down in
                % ~1 s, so it is on whenever the ring holds momentum
                % idle driver (no momentum target, no command): field off
                if abs(D.htgt(i)) > 0.02*m.h_max(i) || abs(cmd_r(i)) > 0.02*m.torque_max(i), P_W = P_W + m.field_power(i); end
            case 'cmg'
                hdot(i) = max(-m.torque_max(i), min(m.torque_max(i), -m.k_speed*(h(i) - m.h0(i))));
                P_W = P_W + m.p_steady(i);
        end
    end
    ng = numel(d); gdot = zeros(ng,1);
    for j = 1:ng
        if D.gfailed(j), continue, end
        gdot(j) = max(-m.gimbal_rate_max, min(m.gimbal_rate_max, cmd_g(j)));
        P_W = P_W + m.gimbal_power*abs(gdot(j))/m.gimbal_rate_max;
    end
end
