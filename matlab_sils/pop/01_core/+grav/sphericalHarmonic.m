function a = sphericalHarmonic(r_ecef, mu, Re, Cbar, Sbar, nmax, mmax)
%GRAV.SPHERICALHARMONIC  Geopotential acceleration from a normalised C/S field.
%
%   a = grav.sphericalHarmonic(r_ecef, mu, Re, Cbar, Sbar, nmax, mmax)
%
%   Pure-MATLAB (no Aerospace Toolbox, no Java) evaluation of the full
%   non-spherical gravitational acceleration using the Cunningham (1970)
%   V/W recursion as formulated by Montenbruck & Gill, "Satellite Orbits",
%   Sec. 3.2.4-3.2.5.  The recursion is built entirely from Cartesian
%   coordinates, so it is NON-SINGULAR at the poles (unlike the spherical
%   latitude/longitude gradient form).
%
%   INPUTS
%     r_ecef  3x1 satellite position in the EARTH-FIXED (ITRF/ECEF) frame [m].
%             MUST be body-fixed -- the field coefficients are defined in ECEF.
%             (The caller rotates ECI->ECEF once per step; see forces.gravity.)
%     mu      GM of the field [m^3/s^2]         (use the field's own GM).
%     Re      reference radius of the field [m] (use the field's own a).
%     Cbar    (nmax+1)x(nmax+1) matrix of 4pi-NORMALISED cosine coeffs, indexed
%             Cbar(n+1,m+1) = \bar C_{n,m}.  Cbar(1,1)=1 gives the central term.
%     Sbar    same size, \bar S_{n,m}.  Sbar(:,1)=0 (order-0 has no sine part).
%     nmax    maximum degree to evaluate.
%     mmax    maximum order to evaluate (mmax<=nmax; pass 0 for zonal-only).
%
%   OUTPUT
%     a       3x1 acceleration in ECEF [m/s^2] (central + all included terms).
%
%   NUMERICS.  Internally the normalised coefficients are converted to the
%   unnormalised C/S that the Cunningham V/W recursion expects.  The
%   denormalisation factor Pi_{nm} = sqrt((2-d0m)(2n+1)(n-m)!/(n+m)!) grows with
%   degree; in IEEE double this stays exact for zonal terms to very high degree
%   but the (n+m)! ratio for near-sectorial tesserals begins to lose precision
%   beyond degree ~40-45.  For the built-in zonal default and low-degree fields
%   this is irrelevant.  For high-degree tesseral fields (EGM2008 @ 70+), prefer
%   the 'toolbox' gravity model (Aerospace Toolbox gravitysphericalharmonic,
%   Pines/Gottlieb normalised) -- see forces.gravity and docs/GRAVITY.md.  A
%   warning is issued once if nmax>45 with non-zero tesserals on this path.
%
%   See also GRAV.TWOBODY, GRAV.J2ACCEL, GRAV.LOADGFC, GRAV.DEFAULTFIELD.

    r = r_ecef(:);
    x = r(1); y = r(2); z = r(3);
    r2 = x*x + y*y + z*z;
    rn = sqrt(r2);

    if nargin < 7 || isempty(mmax), mmax = nmax; end
    mmax = min(mmax, nmax);

    % ---- one-time high-degree precision guard ----
    % NOTE: this path was cross-checked against grav.potential (an INDEPENDENT
    % normalised-Legendre implementation) with a full tesseral field: the relative
    % acceleration error is ~1e-9 at degree 70, i.e. the denormalisation is NOT a
    % practical limit there. The guard is therefore raised to degree 120, beyond
    % which (n+m)! ratios do start to bite. See 08_test/test_gravity.m.
    persistent warned
    if nmax > 120 && mmax > 0 && isempty(warned)
        warning('grav:sphericalHarmonic:highDegree', ...
          ['Native Cunningham path at degree %d with tesserals may lose precision ' ...
           '(denormalisation). Cross-check against the ''toolbox'' model.'], nmax);
        warned = true;
    end

    % ---- denormalise the coefficients we will actually use ----
    % C_{nm} = \bar C_{nm} * Pi_{nm},  Pi_{nm}=sqrt((2-d0m)(2n+1)(n-m)!/(n+m)!)
    N = nmax + 1;                          % need V,W up to degree nmax+1
    C = zeros(N+2); S = zeros(N+2);
    for n = 0:nmax
        for m = 0:min(n, mmax)
            Pi = denormFactor(n, m);
            C(n+1, m+1) = Cbar(n+1, m+1) * Pi;
            if m > 0
                S(n+1, m+1) = Sbar(n+1, m+1) * Pi;
            end
        end
    end

    % ---- Cunningham V,W recursion (unnormalised), Cartesian & pole-safe ----
    % Auxiliary scaled coordinates.
    xf = Re * x / r2;
    yf = Re * y / r2;
    zf = Re * z / r2;
    R2 = Re * Re / r2;

    % V,W up to degree nmax+1, order nmax+1 (indices +1 for 0-based n,m).
    L = nmax + 2;                          % 0..nmax+1
    V = zeros(L+1, L+1);
    W = zeros(L+1, L+1);
    V(1,1) = Re / rn;   W(1,1) = 0;        % V_{0,0}, W_{0,0}

    % Zonal column (m=0), n = 1..nmax+1
    V(2,1) = zf * (2*1-1) * V(1,1);        % V_{1,0}
    for n = 2:nmax+1
        V(n+1,1) = ((2*n-1)*zf*V(n,1) - (n-1)*R2*V(n-1,1)) / n;
        % W(:,1) stays 0
    end

    % Tesseral/sectorial columns (m>=1)
    for m = 1:mmax+1
        % sectorial diagonal V_{m,m}, W_{m,m}
        V(m+1,m+1) = (2*m-1) * ( xf*V(m,m) - yf*W(m,m) );
        W(m+1,m+1) = (2*m-1) * ( xf*W(m,m) + yf*V(m,m) );
        % first sub-diagonal n=m+1
        if m+1 <= nmax+1
            V(m+2,m+1) = (2*m+1) * zf * V(m+1,m+1);
            W(m+2,m+1) = (2*m+1) * zf * W(m+1,m+1);
        end
        % remaining n = m+2 .. nmax+1
        for n = m+2:nmax+1
            f1 = (2*n-1)/(n-m);
            f2 = (n+m-1)/(n-m);
            V(n+1,m+1) = f1*zf*V(n,m+1) - f2*R2*V(n-1,m+1);
            W(n+1,m+1) = f1*zf*W(n,m+1) - f2*R2*W(n-1,m+1);
        end
    end

    % ---- accumulate acceleration (Montenbruck & Gill Eq. 3.33) ----
    ax = 0; ay = 0; az = 0;
    for n = 0:nmax
        for m = 0:min(n, mmax)
            Cnm = C(n+1,m+1);  Snm = S(n+1,m+1);
            if Cnm==0 && Snm==0, continue; end
            if m == 0
                ax = ax - Cnm * V(n+2, 2);
                ay = ay - Cnm * W(n+2, 2);
            else
                fac = (n-m+1)*(n-m+2);     % (n-m+2)!/(n-m)!
                ax = ax + 0.5*( -Cnm*V(n+2,m+2) - Snm*W(n+2,m+2) ...
                       + fac*( Cnm*V(n+2,m) + Snm*W(n+2,m) ) );
                ay = ay + 0.5*( -Cnm*W(n+2,m+2) + Snm*V(n+2,m+2) ...
                       + fac*( -Cnm*W(n+2,m) + Snm*V(n+2,m) ) );
            end
            az = az + (n-m+1)*( -Cnm*V(n+2,m+1) - Snm*W(n+2,m+1) );
        end
    end

    scale = mu / (Re*Re);
    a = scale * [ax; ay; az];
end

% ------------------------------------------------------------------------
function Pi = denormFactor(n, m)
%DENORMFACTOR  Pi_{nm} such that C_{nm} = \bar C_{nm} * Pi_{nm}.
%   Pi = sqrt( (2-delta_{0m}) (2n+1) (n-m)! / (n+m)! )
%   Computed via a product to avoid forming huge factorials directly.
    if m == 0
        d = 1;
    else
        d = 2;
    end
    % ratio (n-m)!/(n+m)! = 1 / prod(n-m+1 : n+m)
    p = 1;
    for k = (n-m+1):(n+m)
        p = p * k;
    end
    Pi = sqrt( d * (2*n+1) / p );
end
