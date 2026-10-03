function M = species(name)
%DRAG.SPECIES  Molar mass [kg/kmol] of a thermospheric species.
%   O=16 O2=32 N2=28 He=4 H=1 N=14 Ar=40. Used to get per-species speed ratios
%   (multi-species drag works across LEO where composition changes).
    t=struct('O',15.9994,'O2',31.9988,'N2',28.0134,'He',4.0026, ...
             'H',1.0079,'N',14.0067,'Ar',39.948);
    if ~isfield(t,name), error('drag:species','unknown species %s',name); end
    M=t.(name);
end
