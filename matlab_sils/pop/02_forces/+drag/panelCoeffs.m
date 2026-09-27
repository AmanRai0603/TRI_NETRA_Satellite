function [cp, ct] = panelCoeffs(model, s, delta, gsi)
%DRAG.PANELCOEFFS  Switchable GSI panel coefficients.
%   model: 'sentman' | 'dria' | 'cll' | 'sesam'
%   gsi fields: Tw, Talt/T; sentman->aT; dria->nO,T; cll->sig_n,sig_t; sesam->nO,T
%
%   A NOTE ON 'sesam' -- it is NOT a panel model, and treating it as one was a
%   category error that made drag.force('sesam') die with "'ct' undefined".
%   drag.sesam(nO,T) returns aT, an ACCOMMODATION COEFFICIENT: a single number
%   saying how completely the incident gas thermalises on the surface. It answers a
%   different question from sentman/dria/cll, which return the pressure and shear
%   coefficients of a facet.
%
%   What people mean by "the SESAM model" is SENTMAN DRIVEN BY A SESAM-DERIVED aT
%   -- the standard VLEO combination, because at VLEO aT is dominated by adsorbed
%   atomic oxygen and is emphatically not the 0.9 everyone types in. That is what
%   this case now does, and it is why the sweep is worth running: 'sentman' with a
%   guessed aT and 'sesam' differ by exactly the physics you are trying to test.
    switch lower(model)
        case 'sentman', [cp,ct]=drag.sentman(s,delta,gsi.aT,gsi.Tw,gsi.Talt);
        case 'dria',    [cp,ct]=drag.dria(s,delta,gsi.nO,gsi.T,gsi.Tw);
        case 'cll',     [cp,ct]=drag.cll(s,delta,gsi.sig_n,gsi.sig_t,gsi.Tw,gsi.Talt);
        case 'sesam'
            % aT from the local atomic-oxygen environment, THEN Sentman.
            aT = drag.sesam(gsi.nO, gsi.T);
            [cp,ct] = drag.sentman(s, delta, aT, gsi.Tw, gsi.Talt);
        otherwise
            error('drag:panelCoeffs', ...
              ['unknown GSI model "%s". Known: sentman (needs gsi.aT), dria ' ...
               '(nO,T), cll (sig_n,sig_t), sesam (nO,T -> aT -> sentman).'], model);
    end
end
