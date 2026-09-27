function kp = bint_oe(ap)
%BINT_OE  Convert ap60 to open-ended Kp index
%
% Input  : ap (ap60) value(s)
% Process: piecewise-linear interpolation over the extended ap-Kp table (beyond Kp = 9)
% Output : open-ended Kp
%
%         Piecewise linear interpolation over the extended ap-Kp table
%
% This is the "open-ended" variant: the standard ap-Kp table extended
% beyond Kp=9 to cover extreme storm conditions.
%
% INPUT   ap  - ap60 value (can be any non-negative real)
% OUTPUT  kp  - corresponding open-ended Kp value

aap = [0, 2, 3, 4, 5, 6, 7, 9, 12, 15, 18, 22, 27, 32, 39, 48, 56, 67, ...
       80, 94, 111, 132, 154, 179, 207, 236, 265, 294, 324, 355, 388, 421, ...
       456, 494, 534, 574, 617, 657];

kp_tbl = [0, .33, .66, 1, 1.33, 1.66, 2, 2.33, 2.66, 3, 3.33, 3.66, ...
          4, 4.33, 4.66, 5, 5.33, 5.66, 6, 6.33, 6.66, 7, 7.33, 7.66, ...
          8, 8.33, 8.66, 9, 9.33, 9.66, 10, 10.33, 10.66, 11, 11.33, ...
          11.66, 12, 12.33];

n = numel(aap);

% Exact match (handles the common case of integer ap values)
idx = find(aap == ap, 1);
if ~isempty(idx)
    kp = kp_tbl(idx);
    return;
end

% Piecewise linear interpolation
for i = 1:n
    if aap(i) > ap
        if i == 1
            % Below table minimum: extrapolate
            kp = kp_tbl(1) + ((kp_tbl(2)-kp_tbl(1))/(aap(2)-aap(1))) * (ap - aap(1));
        else
            kp = kp_tbl(i-1) + ((kp_tbl(i)-kp_tbl(i-1))/(aap(i)-aap(i-1))) * (ap - aap(i-1));
        end
        return;
    end
end

% Above table maximum: extrapolate from last interval
kp = kp_tbl(n-1) + ((kp_tbl(n)-kp_tbl(n-1))/(aap(n)-aap(n-1))) * (ap - aap(n-1));
warning('bint_oe: ap=%.2f exceeds table max (%.1f), extrapolating Kp.', ap, aap(n));

end
