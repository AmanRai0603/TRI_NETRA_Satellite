function H = pace(H, t)
%ASILS.HAL.PACE  Hold the loop to the wall clock (OILS/HILS need real time).
    if H.realtime > 0
        lag = t/H.realtime - toc(H.t0_wall);
        if lag > 0, pause(lag); end
    end
    H.frames = H.frames + 1;
end
