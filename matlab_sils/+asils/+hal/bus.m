function b = bus()
%ASILS.HAL.BUS  The bus between the twin's device emulators and its flight software, empty: the MATLAB twin of the
%   engine's in-memory bus (adcs_fsw_abi::Bus, adcs_hal.h): the I2C register blocks of the magnetometer, the Sun
%   sensors and the Earth sensor (7 bytes each, or [] when the part does not answer), the gyro's SPI response (13),
%   three UART byte queues (ports 0 to 2), the CAN frames waiting for the flight software (can_rx) and those it sent
%   (can_tx), one frame a row [id, dlc, data 1 to 8], the eight PWM words, and the bus time [ns].
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    b = struct('now_ns', 0, 'mag', [], 'gyro', [], 'sun', [], 'es', [], 'uart', {{zeros(1, 0), zeros(1, 0), zeros(1, 0)}}, ...
               'can_rx', zeros(0, 10), 'can_tx', zeros(0, 10), 'pwm', zeros(8, 1));
end
