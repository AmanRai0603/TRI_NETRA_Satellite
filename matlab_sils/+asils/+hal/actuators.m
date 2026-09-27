function out = actuators(H, out, t)
%ASILS.HAL.ACTUATORS  Actuator frames across the FSW boundary (PWM, torque, rate, valve time).
%   In the 'udp' backend (OILS/HILS) the commands come back from the flight
%   OBC instead of the MATLAB FSW, and replace it.
    L = H.lsb;
    q15 = @(x, full) max(-32767, min(32767, round(x./full*2^15)))/2^15.*full;
    out.m_body = q15(out.m_body, L.m_max);
    if ~isempty(out.cmd_r), out.cmd_r = q15(out.cmd_r, L.tmax); end
    if ~isempty(out.cmd_g), out.cmd_g = q15(out.cmd_g, L.gmax); end
    out.duty = round(out.duty*100)/100;               % 1 ms valve steps at 10 Hz
    if strcmp(H.backend, 'udp') && H.sock.NumBytesAvailable > 0
        out = asils.hal.unpack_actuators(read(H.sock, H.sock.NumBytesAvailable, 'uint8'), out, L);
    end
end
