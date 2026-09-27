function dcs = mainLines(jd_tt)
%OCEANTIDES.MAINLINES  Ocean-tide geopotential correction from the 8 dominant
%   constituents, degree 2 (framework with representative amplitudes).
%   ** For real accuracy load a full model (FES2014/EOT) via oceantides.fromModel;
%      the built-in amplitudes here are representative, not authoritative. **
%   dcs.dC/dS (5x5) normalized. ΔC̄nm−iΔS̄nm = Σ_f (Cp∓ i Sp) e^{iθ_f}.
    beta=oceantides.doodson(jd_tt);
    % [Doodson multipliers n1..n6 ; degree n ; order m ; Cplus Splus (1e-11)]
    L={ [2 0 0 0 0 0], 2,2, -3.10,  0.40   % M2
        [2 2 -2 0 0 0],2,2, -1.50,  0.20   % S2
        [2 -1 0 1 0 0],2,2, -0.60,  0.10   % N2
        [2 2 0 0 0 0], 2,2, -0.40,  0.05   % K2
        [1 1 0 0 0 0], 2,1,  1.40, -0.30   % K1
        [1 -1 0 0 0 0],2,1,  1.00, -0.20   % O1
        [1 1 -2 0 0 0],2,1,  0.45, -0.10   % P1
        [1 -2 0 1 0 0],2,1,  0.20, -0.05}; % Q1
    dC=zeros(5); dS=zeros(5);
    for i=1:size(L,1)
        n=L{i,1}; deg=L{i,2}; m=L{i,3}; Cp=L{i,4}*1e-11; Sp=L{i,5}*1e-11;
        th=sum(n.*beta);
        dC(deg+1,m+1)=dC(deg+1,m+1)+(Cp*cos(th)+Sp*sin(th));
        dS(deg+1,m+1)=dS(deg+1,m+1)+(Sp*cos(th)-Cp*sin(th));
    end
    dcs.dC=dC; dcs.dS=dS;
end
