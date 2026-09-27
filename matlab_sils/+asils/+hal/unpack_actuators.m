function out = unpack_actuators(b, out, L)
%ASILS.HAL.UNPACK_ACTUATORS  Bytes from the flight OBC -> actuator frame:
%   [coil duty i16x3 (Q15)][rotor torque i16 x nr (Q15)][gimbal rate i16 x ng (Q15)][valve ms u8 x nc]
    p = 1; nr = numel(out.cmd_r); ng = numel(out.cmd_g); nc = numel(out.duty);
    v = double(typecast(uint8(b(p:p+5)), 'int16')); p = p + 6; out.m_body = v(:)/2^15*L.m_max;
    if nr, v = double(typecast(uint8(b(p:p+2*nr-1)), 'int16')); p = p + 2*nr; out.cmd_r = v(:)/2^15.*L.tmax; end
    if ng, v = double(typecast(uint8(b(p:p+2*ng-1)), 'int16')); p = p + 2*ng; out.cmd_g = v(:)/2^15*L.gmax; end
    if nc, out.duty = double(b(p:p+nc-1))/100; end
end
