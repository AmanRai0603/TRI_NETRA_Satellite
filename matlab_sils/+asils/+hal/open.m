function H = open(P)
%ASILS.HAL.OPEN  Open the flight-software boundary (the MATLAB side of adcs_hal.h).
%
%   Every sensor reaches the flight software, and every command leaves it, as the bytes of its part: the
%   I2C register blocks, the gyro's SPI response, the star tracker's and the GNSS receiver's UART frames,
%   the rotors' CAN telemetry, the coils' PWM words and the CAN command frames (asils.hal.bus), as on the
%   engine's bus and on the OBC. The counts are the design's emulator scaling (asils.models.emucodec, the
%   inverse of the flight drivers); the framing is the rig's (this package).
%
%   P.hal.backend
%     'sils'      (default) the twin's flight software (asils.fsw.step) on the bus, in memory
%     'loopback'  every tick the bus's sensor bytes and the command bytes are also packed into the link's
%                 frames and unpacked again (asils.hal.link): the layout the OILS/HILS link carries is
%                 exercised every tick
%     'udp'       OILS: the sensor frame goes to the flight OBC and its command frame comes back
%                 (P.hal.host, P.hal.port_tx, P.hal.port_rx); HILS: the same link to the rig's interface
%                 unit. Needs udpport (MATLAB R2020b+) or the Octave instrument-control package.
%   P.hal.realtime  0 = as fast as possible; 1 = wall clock paces the loop
%                   (required for OILS/HILS); 0.5 = half speed, etc.
%   P.hal.stimulus  true: also emit the lab stimulus each tick (asils.hal.stimulus).
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    h = asils.util.getf(P, 'hal', struct());
    H.backend = asils.util.getf(h, 'backend', 'sils');
    H.realtime = asils.util.getf(h, 'realtime', 0);
    H.stimulus = asils.util.getf(h, 'stimulus', false);
    H.scale = asils.models.emucodec.emu_scale();    % the emulators' scaling, from the drivers' conversions
    H.t0_wall = tic; H.frames = 0; H.bytes = 0;
    switch H.backend
        case {'sils', 'loopback'}
        case 'udp'
            H.host = asils.util.getf(h, 'host', '127.0.0.1');
            H.port_tx = asils.util.getf(h, 'port_tx', 47001); H.port_rx = asils.util.getf(h, 'port_rx', 47002);
            if exist('udpport', 'file') || exist('udpport', 'builtin') || exist('udpport', 'class')
                H.sock = udpport('LocalPort', H.port_rx);
            else
                error('asils:hal:udp', ['backend ''udp'' needs udpport (MATLAB R2020b+) or the Octave ' ...
                    'instrument-control package (pkg load instrument-control). SILS runs need neither.']);
            end
        otherwise
            error('asils:hal:backend', 'unknown HAL backend %s', H.backend);
    end
end
