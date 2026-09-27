function nu = shadow(r_sat, r_sun)
%ASILS.ENV.SHADOW  Conical Earth-shadow factor nu in [0,1] (1 = full Sun).
%   Montenbruck & Gill (2000) sec. 3.4.2: apparent radii of Sun and Earth and
%   their separation, with the partial-overlap (penumbra) area.
    Rs = 6.957e8; Re = 6378137.0;
    d  = r_sun - r_sat;
    a  = asin(Rs/norm(d));                  % apparent Sun radius
    b  = asin(Re/norm(r_sat));              % apparent Earth radius
    c  = acos(max(-1, min(1, -r_sat'*d/(norm(r_sat)*norm(d)))));
    if c >= a + b
        nu = 1;
    elseif c < b - a
        nu = 0;
    else
        x = (c^2 + a^2 - b^2)/(2*c);
        y = sqrt(max(0, a^2 - x^2));
        A = a^2*acos(max(-1,min(1,x/a))) + b^2*acos(max(-1,min(1,(c - x)/b))) - c*y;
        nu = 1 - A/(pi*a^2);
    end
end
