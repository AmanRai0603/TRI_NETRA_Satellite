function dcs = fromModel(jd_tt, tbl)
%OCEANTIDES.FROMMODEL  Full ocean-tide correction from a loaded model table
%   (FES2004/EOT/GOT), fetched & parsed by data/fetch_fes2004.m. ADVANCED.
%   tbl fields (struct of arrays): doodson (Nx6), n, m, Cp, Sp, Cm, Sm (Nx1, SI).
%   dcs.dC/dS : (nmax+1)x(nmax+1) normalized coefficient corrections.
    beta = oceantides.doodson(jd_tt);            % 6x1 fundamental arguments
    th = tbl.doodson * beta(:);                  % Nx1 Doodson arguments
    ct = cos(th); st = sin(th);
    nmax = max(tbl.n); dC = zeros(nmax+1); dS = zeros(nmax+1);
    for i = 1:numel(tbl.n)
        n=tbl.n(i); m=tbl.m(i);
        dC(n+1,m+1)=dC(n+1,m+1)+(tbl.Cp(i)+tbl.Cm(i))*ct(i)+(tbl.Sp(i)+tbl.Sm(i))*st(i);
        dS(n+1,m+1)=dS(n+1,m+1)+(tbl.Sp(i)-tbl.Sm(i))*ct(i)-(tbl.Cp(i)-tbl.Cm(i))*st(i);
    end
    dcs.dC=dC; dcs.dS=dS;
end
