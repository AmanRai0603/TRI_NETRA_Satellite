function [U, a, prop] = actuate(U, dev, cmd, x, a, prop, dt)
%ASILS.DEVICES.ACTUATE  The commands applied to the coils, the momentum devices and the thrusters (propellant counted; an
%   empty tank fails every valve): the engine's run.rs actuate, in MATLAB. The models are act's methods (+asils/+models:
%   coilset_apply, rotorset_apply, thrusters_apply, rcsprop's tank_update, thrusters_empty). Unfitted devices keep what
%   they last delivered (a: m_b, m_coil, hdot, gdot, tau_rcs, p_mtq, p_mex, p_rcs).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    if dev.mtq.fitted
        [a.m_b, a.m_coil, a.p_mtq, U.mtq] = asils.models.coilset.coilset_apply(U.mtq, dev.mtq.rec, cmd.m_body, dt);
    end
    if dev.mex.rec.n > 0
        [a.hdot, a.gdot, a.p_mex, U.mex] = asils.models.rotorset.rotorset_apply(U.mex, dev.mex.rec, cmd.cmd_r, cmd.cmd_g, x.h, dt);
    end
    if ~isempty(U.rcs)
        nc = dev.rcs.rec.nc;
        duty = zeros(6, 1); duty(1:nc) = cmd.duty(1:nc);
        [a.tau_rcs, mdot, a.p_rcs] = asils.models.thrusters.thrusters_apply(U.rcs, dev.rcs.rec, duty, dt);
        [prop, empty] = asils.models.rcsprop.tank_update(prop, mdot, dt, dev.rcs.rec.prop_kg);   % l3_rcs_row_08
        if empty, U.rcs = asils.models.thrusters.thrusters_empty(U.rcs); end
    end
end
