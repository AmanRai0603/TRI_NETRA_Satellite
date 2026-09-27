function f = facet(type, n, A, alpha, rho_s, rho_d, axis, dbl)
%SRP.FACET  Construct one facet struct (body panel or solar array).
%   Fields: type ('body'|'array'), n (unit normal, body frame), A (m^2),
%           alpha/rho_s/rho_d (absorb/specular/diffuse, sum=1),
%           axis (array pivot axis, body frame), double (both sides illuminable).
    f = struct('type',type, 'n',n(:), 'A',A, 'alpha',alpha, ...
               'rho_s',rho_s, 'rho_d',rho_d, 'axis',axis(:), 'double',dbl);
end
