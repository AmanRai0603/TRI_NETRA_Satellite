function P = rtn_project(d, r, v)
%VALIDATION.RTN_PROJECT  Project a vector series into the RTN triad. [N x 3] -> [N x 3]
%   P(:,1)=radial  P(:,2)=along-track  P(:,3)=cross-track
%
%   rtn_stats already builds this triad and returns RMS + the position components.
%   This is the projection ALONE, so the velocity residual can use the same triad
%   without a second RMS or a second definition of "along-track". One triad, one
%   definition -- the alternative is two functions that agree until one is edited.
    n = size(d,1);
    P = zeros(n,3);
    for k = 1:n
        rk = r(k,:).'; vk = v(k,:).';
        R_ = rk/norm(rk);
        Cx = cross(rk,vk);
        N_ = Cx/norm(Cx);              % cross-track (orbit normal)
        T_ = cross(N_,R_);             % along-track completes a right-handed triad
        P(k,:) = [dot(d(k,:).',R_), dot(d(k,:).',T_), dot(d(k,:).',N_)];
    end
end
