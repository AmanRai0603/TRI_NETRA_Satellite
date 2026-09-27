function F = buildBox(Lx, Ly, Lz, opt)
%SRP.BUILDBOX  Six body facets of a rectangular bus (dims [m], optics struct).
%   opt has fields alpha, rho_s, rho_d (sum to 1).
    d = { [1;0;0], Ly*Lz;  [-1;0;0], Ly*Lz;
          [0;1;0], Lx*Lz;  [0;-1;0], Lx*Lz;
          [0;0;1], Lx*Ly;  [0;0;-1], Lx*Ly };
    F = repmat(srp.facet('body',[0;0;1],0,0,0,0,[0;0;0],false), 1, 6);
    for i = 1:6
        F(i) = srp.facet('body', d{i,1}, d{i,2}, opt.alpha, opt.rho_s, opt.rho_d, [0;0;0], false);
    end
end
