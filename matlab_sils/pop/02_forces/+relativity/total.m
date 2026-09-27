function [a, parts] = total(rSat, vSat, E, terms, mu)
%RELATIVITY.TOTAL  Post-Newtonian acceleration (IERS 2010), switchable terms.
%   terms: cellstr subset of {'schwarzschild','lensethirring','desitter'}
%   (default all). E = ephemInputs struct (for de Sitter Earth heliocentric).
%   mu:    Earth GM. Pass the GRAVITY FIELD's mu, not de440's -- they differ at
%          ~7.5e-10 relative and the field's is what the dominant term uses.
%          Omitted -> de440.constants().mu_earth.
    if nargin<4||isempty(terms), terms={'schwarzschild','lensethirring','desitter'}; end
    if nargin<5||isempty(mu), K=de440.constants(); mu=K.mu_earth; end
    a=[0;0;0]; parts=struct();
    for k=1:numel(terms)
        switch lower(terms{k})
            case 'schwarzschild', parts.schwarzschild=relativity.schwarzschild(rSat,vSat,mu); a=a+parts.schwarzschild;
            case 'lensethirring', parts.lensethirring=relativity.lenseThirring(rSat,vSat,mu); a=a+parts.lensethirring;
            case 'desitter'
                if isempty(E) || ~isfield(E,'earth_helio_pos')
                    error('relativity:total:noEphem', ...
                      ['the de Sitter term needs the Earth''s heliocentric state ' ...
                       '(E.earth_helio_pos/vel from ephemInputs). Pass E, or drop ' ...
                       '''desitter'' from terms.']);
                end
                parts.desitter=relativity.deSitter(rSat,vSat,E.earth_helio_pos,E.earth_helio_vel);
                a=a+parts.desitter;
            otherwise
                % There was no otherwise: an unknown term silently contributed
                % NOTHING, so a typo in cfg.forces.relativity.terms quietly turned
                % the term off and the run looked fine.
                error('relativity:total:term', ...
                  ['unknown relativity term "%s". Known: schwarzschild, ' ...
                   'lensethirring, desitter.'], terms{k});
        end
    end
end
