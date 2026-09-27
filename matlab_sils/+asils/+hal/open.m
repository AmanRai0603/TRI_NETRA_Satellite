function H = open(P)
%ASILS.HAL.OPEN  Open the flight-software boundary (the MATLAB side of adcs_hal.h).
%
%   Everything the FSW reads from a sensor and writes to an actuator crosses
%   this boundary as the integer register values the flight drivers see, so
%   quantisation, ranges and frame layouts are exercised in SILS exactly as
%   they will be on the OBC (spec/fsw/include/adcs_hal.h, "one header, three
%   implementations").
%
%   P.hal.backend
%     'sils'      (default) values quantised to their register LSBs, in memory
%     'loopback'  every frame is also packed to bytes and unpacked again: the
%                 byte layout the OILS/HILS link carries is checked every tick
%     'udp'       OILS: sensor frames go to the flight OBC and actuator frames
%                 come back (P.hal.host, P.hal.port_tx, P.hal.port_rx);
%                 HILS: the same link to the rig's interface unit. Needs
%                 udpport (MATLAB R2020b+) or the Octave instrument-control pkg.
%   P.hal.realtime  0 = as fast as possible; 1 = wall clock paces the loop
%                   (required for OILS/HILS); 0.5 = half speed, etc.
%   P.hal.stimulus  true: also emit the lab stimulus each tick (field for the
%                   Helmholtz cage, Sun direction for the Sun simulator, body
%                   rate for the air-bearing reference) -- asils.hal.stimulus.
    h = asils.util.getf(P, 'hal', struct());
    H.backend = asils.util.getf(h, 'backend', 'sils');
    H.realtime = asils.util.getf(h, 'realtime', 0);
    H.stimulus = asils.util.getf(h, 'stimulus', false);
    H.lsb = asils.hal.lsb(P.dev);
    H.t0_wall = tic; H.frames = 0; H.bytes = 0;
    switch H.backend
        case {'sils', 'loopback'}
        case 'udp'
            H.host = asils.util.getf(h, 'host', '127.0.0.1');
            H.port_tx = asils.util.getf(h, 'port_tx', 47001); H.port_rx = asils.util.getf(h, 'port_rx', 47002);
            if exist('udpport', 'file') || exist('udpport', 'builtin')
                H.sock = udpport('LocalPort', H.port_rx);
            elseif exist('udpport', 'class')
                H.sock = udpport('LocalPort', H.port_rx);
            else
                error('asils:hal:udp', ['backend ''udp'' needs udpport (MATLAB R2020b+) or the Octave ' ...
                    'instrument-control package (pkg load instrument-control). SILS runs need neither.']);
            end
        otherwise
            error('asils:hal:backend', 'unknown HAL backend %s', H.backend);
    end
end
