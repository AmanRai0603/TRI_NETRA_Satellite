function sc = vleo16u()
%VLEO16U  Example 16U-class VLEO smallsat: box bus + 2 double-sided arrays.
%   Returns sc.mass [kg] and sc.facets (struct array). EDIT the optical
%   coefficients to your measured/vendor values (see docs/INPUTS.md).
    sc.mass = 24.0;                              % kg
    % BODY AXES: +x is RAM (drag.ramAttitude aligns body +x with v_rel, and every
    % panel model assumes it). So the 0.34 m LONG axis must NOT be x, or the 16U
    % flies broadside. dgeom.vleo16u used to put 0.34 on x and disagreed with this
    % file by 1.7x in frontal area -- two geometries, one satellite, no complaint.
    Lx = 0.20; Ly = 0.20; Lz = 0.34;            % m: 0.20 x 0.20 ram face, 0.34 long
    bus = struct('alpha',0.30, 'rho_s',0.30, 'rho_d',0.40);   % MLI-like  (sum=1)
    arr = struct('alpha',0.85, 'rho_s',0.05, 'rho_d',0.10);   % solar array (sum=1)

    F = srp.buildBox(Lx, Ly, Lz, bus);
    F = srp.addArray(F, 0.34*0.20, [0;1;0], arr);   % array 1 (pivots about body +Y)
    F = srp.addArray(F, 0.34*0.20, [0;1;0], arr);   % array 2
    sc.facets = F;
end
