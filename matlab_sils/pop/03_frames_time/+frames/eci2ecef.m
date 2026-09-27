function [C, Ct, info] = eci2ecef(utc, build, opt)
%FRAMES.ECI2ECEF  Unified ECI(GCRF/J2000) -> ECEF(ITRF) rotation dispatcher.
%   [C, Ct, info] = frames.eci2ecef(utc, build[, opt])
%     utc   : datetime (UTC) or [Y Mo D H Mi S] row.
%     build : 'A' | 'B' | 'C' | 'gmst'
%             'A'  finals2000A (Bulletin A/B)         -> matches dcmeci2ecef
%             'B'  EOP 20 C04 (ITRF2020/ICRF3)         [recommended precise]
%             'C'  C04 + sub-daily tidal + zonal (max accuracy)
%             'gmst' offline GMST-only approx (no EOP fetch; ~arcsec level)
%     opt   : passthrough options for builds A/B/C (see eci2ecef_B header);
%             for 'gmst' opt may carry .gmst_rad,.xp,.yp (else derived from utc).
%   Returns C (r_ecef=C*r_eci), Ct=C', and info (EOP/source actually used).
%
%   Builds A/B/C download & cache IERS EOP once (needs internet the first time).
%   'gmst' needs nothing and is the default used by the shipped examples so they
%   run in any environment; production runs should use 'B' or 'C'.
    if nargin<2||isempty(build), build='gmst'; end
    if nargin<3, opt=struct(); end
    info=struct('build',build);
    switch upper(build)
        case 'A', [C,Ct,info]=frames.eci2ecef_A(utc,opt);
        case 'B', [C,Ct,info]=frames.eci2ecef_B(utc,opt);
        case 'C', [C,Ct,info]=frames.eci2ecef_C(utc,opt);
        case 'GMST'
            if isfield(opt,'gmst_rad')&&~isempty(opt.gmst_rad)
                g=opt.gmst_rad;
            else
                v=frames.utcvec(utc);
                T=timeconv.convertUTC(v(1),v(2),v(3),v(4),v(5),v(6), getf(opt,'dUT1',0));
                g=T.gmst_rad;
            end
            xp=getf(opt,'xp',0); yp=getf(opt,'yp',0);
            [C,Ct]=frames.eci2ecefGMST(g,xp,yp);
            info.gmst_rad=g;
        otherwise, error('frames:eci2ecef','unknown build "%s"',build);
    end
end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
