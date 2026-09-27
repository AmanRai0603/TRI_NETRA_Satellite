function facets = addArray(facets, normal, area)
%ADDARRAY  Append a double-sided flat panel (solar array) of given area [m^2].
%   Both faces are added so whichever is windward contributes.
    facets(end+1)=struct('n', normal(:),  'A', area);
    facets(end+1)=struct('n',-normal(:),  'A', area);
end
