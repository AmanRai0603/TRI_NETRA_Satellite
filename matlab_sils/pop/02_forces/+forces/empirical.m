function a = empirical(ctx)
%FORCES.EMPIRICAL  Empirical along/cross/radial acceleration (ECI) for OD.
%   Reads ctx.cfg.forces.empirical.acc = [aR aT aN] (radial, transverse, normal)
%   [m/s^2] constant biases in the RTN (Gauss) frame.  Useful as estimable
%   parameters that absorb unmodelled dynamics during orbit determination.
    c = ctx.cfg.forces.empirical;
    acc = getf(c,'acc',[0 0 0]);
    r=ctx.r_eci; v=ctx.v_eci;
    R=r/norm(r); N=cross(r,v); N=N/norm(N); T=cross(N,R);
    a = acc(1)*R + acc(2)*T + acc(3)*N;
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
