function B = igrf_ned(gh, lat_rad, lon_rad, alt_km, nmax)
%ASILS.ENV.IGRF_NED  IGRF field [North; East; Down] in nT at a geodetic point.
%   Synthesis PORTED from TRI-NETRa Standard Code env.igrfScalar
%   (Detumbling.m:2523): geodetic->geocentric conversion, Schmidt
%   semi-normalised recursion, rotation back to geodetic NED. CHANGE: optional
%   truncation degree nmax (default 13) for speed.
    if nargin < 5 || isempty(nmax), nmax = 13; end
    Re = 6371.2;
    costheta = cos(pi/2 - lat_rad); sintheta = sin(pi/2 - lat_rad);
    a = 6378.137; f = 1/298.257223563; b = a*(1 - f);
    rho = sqrt((a*sintheta)^2 + (b*costheta)^2);
    r = sqrt(alt_km^2 + 2*alt_km*rho + (a^4*sintheta^2 + b^4*costheta^2)/rho^2);
    cd = (alt_km + rho)/r;
    sd = (a^2 - b^2)/rho*costheta*sintheta/r;
    oldcos = costheta;
    costheta = costheta*cd - sintheta*sd;
    sintheta = sintheta*cd + oldcos*sd;
    phi = lon_rad;
    cosphi = cos((1:nmax)*phi); sinphi = sin((1:nmax)*phi);
    Pmax = (nmax+1)*(nmax+2)/2;
    Br = 0; Bt = 0; Bp = 0;
    P = zeros(1, Pmax); P(1) = 1; P(3) = sintheta;
    dP = zeros(1, Pmax); dP(3) = costheta;
    m = 1; n = 0; ci = 1;
    a_r = (Re/r)^2;
    for Pi = 2:Pmax
        if n < m
            m = 0; n = n + 1; a_r = a_r*(Re/r);
        end
        if m < n && Pi ~= 3
            l1 = Pi - n; l2 = Pi - 2*n + 1;
            k1 = (2*n - 1)/sqrt(n^2 - m^2); k2 = sqrt(((n-1)^2 - m^2)/(n^2 - m^2));
            P(Pi)  = k1*costheta*P(l1) - k2*P(l2);
            dP(Pi) = k1*(costheta*dP(l1) - sintheta*P(l1)) - k2*dP(l2);
        elseif Pi ~= 3
            ln = Pi - n - 1; k = sqrt(1 - 1/(2*m));
            P(Pi)  = k*sintheta*P(ln);
            dP(Pi) = k*(sintheta*dP(ln) + costheta*P(ln));
        end
        if m == 0
            c = a_r*gh(ci);
            Br = Br + (n+1)*c*P(Pi); Bt = Bt - c*dP(Pi);
            ci = ci + 1;
        else
            gc = gh(ci)*cosphi(m) + gh(ci+1)*sinphi(m);
            gs = -gh(ci)*sinphi(m) + gh(ci+1)*cosphi(m);
            c = a_r*gc;
            Br = Br + (n+1)*c*P(Pi); Bt = Bt - c*dP(Pi);
            if sintheta == 0
                Bp = Bp - costheta*a_r*gs*dP(Pi);
            else
                Bp = Bp - 1/sintheta*a_r*m*gs*P(Pi);
            end
            ci = ci + 2;
        end
        m = m + 1;
    end
    Bx = -Bt; By = Bp; Bz = -Br;
    B = [Bx*cd + Bz*sd; By; Bz*cd - Bx*sd];
end
