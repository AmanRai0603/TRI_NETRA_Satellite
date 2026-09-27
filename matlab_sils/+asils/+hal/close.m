function close(H)
%ASILS.HAL.CLOSE  Release the link.
    if strcmp(H.backend, 'udp') && isfield(H, 'sock')
        try, delete(H.sock); catch, end
    end
end
