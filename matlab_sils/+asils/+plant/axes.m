function A = axes(M, d)
%ASILS.PLANT.AXES  Current rotor spin axes (3 x nr) at gimbal angles d.
    A = M.A0;
    for i = 1:M.nr
        j = M.gi(i);
        if j > 0, A(:,i) = cos(d(j))*M.A0(:,i) + sin(d(j))*M.T0(:,i); end
    end
end
