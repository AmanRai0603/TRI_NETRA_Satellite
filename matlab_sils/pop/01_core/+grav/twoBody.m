function a = twoBody(r, mu)
%GRAV.TWOBODY  Point-mass (Keplerian) gravitational acceleration.
%
%   a = grav.twoBody(r, mu)
%
%   INPUTS
%     r   3x1 position vector of the satellite w.r.t. the central body [m].
%         Frame is irrelevant for the point-mass term (it is central), so you
%         may pass ECI or ECEF interchangeably.
%     mu  gravitational parameter of the central body [m^3/s^2].
%
%   OUTPUT
%     a   3x1 acceleration [m/s^2], pointing toward the central body.
%
%   This is the a0 term of the Cowell sum:  a = -mu/|r|^3 * r.  Every other
%   force in the stack is a *perturbation* added on top of this.
%
%   See also GRAV.SPHERICALHARMONIC, GRAV.J2ACCEL.
    r  = r(:);
    rn = norm(r);
    a  = -mu * r / rn^3;
end
