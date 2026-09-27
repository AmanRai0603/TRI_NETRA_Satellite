function s = speedRatio(Vrel, T, Mmol)
%DRAG.SPEEDRATIO  Molecular speed ratio s = Vrel / sqrt(2 R T / Mmol).
%   Vrel [m/s], T ambient [K], Mmol mean molar mass [kg/kmol] (atomic O ~16).
    R=8314.462; s=Vrel./sqrt(2*R*T./Mmol);
end
