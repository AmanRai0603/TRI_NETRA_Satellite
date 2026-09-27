function Rbw = bodyToWind(alpha, beta)
%DRAG.BODYTOWIND  Body->wind DCM from alpha (AoA), beta (sideslip). Wind x-axis
%   is along the relative velocity. F_wind = Rbw*F_body gives [-D; S; -L]
%   (drag, side force, lift). (Stevens & Lewis 2003, eq. for wind axes.)
    ca=cos(alpha); sa=sin(alpha); cb=cos(beta); sb=sin(beta);
    Rbw = [ cb*ca,  sb,  cb*sa ;
           -sb*ca,  cb, -sb*sa ;
           -sa,      0,  ca    ];
end
