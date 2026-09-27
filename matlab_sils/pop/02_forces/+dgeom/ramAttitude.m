function R_bi = ramAttitude(vrel)
%RAMATTITUDE  Body->inertial DCM that aligns body-x with the relative velocity
%   (ram-pointing, alpha=beta=0). Body-y completes with the +z reference.
    x=vrel(:)/norm(vrel); zref=[0;0;1]; if abs(dot(x,zref))>0.98, zref=[0;1;0]; end
    y=cross(zref,x); y=y/norm(y); z=cross(x,y);
    R_bi=[x y z];
end
