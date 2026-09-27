function out = subsref_tern(c, a, b)
%SUBSREF_TERN  Inline choose-one. Exists as a package-free FILE, not a script local,
%   because MATLAB hoists a script's local functions and Octave does not -- helpers
%   at the end of a script are never defined there. See validate_OD's footer.
    if c, out = a; else, out = b; end
end
