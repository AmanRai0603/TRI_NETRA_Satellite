function c = commands(bus, m_max, tmax, gmax, dt, s, nr, ng, nc)
%ASILS.HAL.COMMANDS  What the flight software commanded, decoded from its bus writes (the engine's emu.rs decode_pwm and
%   decode_can): the coils' dipole from the PWM words, each rotor's torque, each gimbal's rate and each valve's duty from
%   the CAN frames (the values asils.models.emucodec's coil_dipole, command_value, valve_duty).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    w = bus.pwm(1:3); w(w >= 32768) = w(w >= 32768) - 65536;
    c = struct('m_body', asils.models.emucodec.coil_dipole(w, m_max, s), 'cmd_r', zeros(8, 1), 'cmd_g', zeros(4, 1), 'duty', zeros(6, 1));
    for k = 1:size(bus.can_tx, 1)
        f = bus.can_tx(k, :); id = f(1);
        w16 = f(3) + 256*f(4); if w16 >= 32768, w16 = w16 - 65536; end
        if id >= 256 && id < 264
            i = id - 255; c.cmd_r(i) = asils.models.emucodec.command_value(w16, tmax(i), s);
        elseif id >= 320 && id < 324
            c.cmd_g(id - 319) = asils.models.emucodec.command_value(w16, gmax, s);
        elseif id == 768
            for i = 1:6, c.duty(i) = asils.models.emucodec.valve_duty(f(2 + i), dt, s); end
        end
    end
end
