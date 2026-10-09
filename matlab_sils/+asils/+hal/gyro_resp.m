function b = gyro_resp(ok, w, s)
%ASILS.HAL.GYRO_RESP  The gyro's SPI response: status, three int32 counts (asils.models.emucodec.gyro_counts), little-endian.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    x = asils.hal.le32(asils.models.emucodec.gyro_counts(w, s)).';
    b = [double(ok); x(:)];
end
