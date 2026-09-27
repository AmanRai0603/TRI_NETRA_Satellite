function facets = buildBox(Lx, Ly, Lz)
%BUILDBOX  Six-facet rectangular bus. Body axes: +x ram. Returns facet structs
%   with .n (outward normal) and .A (area).
    facets=struct('n',{},'A',{});
    add=@(n,A) struct('n',n(:),'A',A);
    facets(end+1)=add([ 1 0 0], Ly*Lz); facets(end+1)=add([-1 0 0], Ly*Lz);
    facets(end+1)=add([0  1 0], Lx*Lz); facets(end+1)=add([0 -1 0], Lx*Lz);
    facets(end+1)=add([0 0  1], Lx*Ly); facets(end+1)=add([0 0 -1], Lx*Ly);
end
