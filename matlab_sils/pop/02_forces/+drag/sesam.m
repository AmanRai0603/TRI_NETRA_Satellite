function aT = sesam(nO, T)
%DRAG.SESAM  Semi-Empirical Satellite Accommodation Model. Energy accommodation
%   from atomic-oxygen number density nO [m^-3] and temperature T [K]:
%     aT = 7.5e-17 nO T / (1 + 7.5e-17 nO T)   (valid ~0.85<=aT<=1)
%   Pilinski et al.; Flying-Laptop form. Feed nO,T from your density model.
    x=7.5e-17*nO.*T; aT=x./(1+x);
end
