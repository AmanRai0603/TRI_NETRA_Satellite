function c = cross3(a, b)
%ASILS.UTIL.CROSS3  3-vector cross product without the argument checks of cross()
%   (the built-in is ~10x slower in Octave and sits in the 10 Hz loop).
    c = [a(2)*b(3)-a(3)*b(2); a(3)*b(1)-a(1)*b(3); a(1)*b(2)-a(2)*b(1)];
end
