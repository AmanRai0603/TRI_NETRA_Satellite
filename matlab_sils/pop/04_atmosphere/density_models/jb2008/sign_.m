%--------------------------------------------------------------------------
% sign: returns absolute value of a with sign of b
%
% Input  : a, b
% Process: abs(a) with the sign of b
% Output : signed result
%
%--------------------------------------------------------------------------
function result = sign_(a, b)
if (b>=0)
    result = abs(a);
else
    result = -abs(a);
end