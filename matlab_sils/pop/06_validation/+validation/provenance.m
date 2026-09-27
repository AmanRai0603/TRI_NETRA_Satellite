function P = provenance(cfg, W, REF, SAT, ledger)
%VALIDATION.PROVENANCE  Classify EVERY input by where it actually came from.
%   NOTE ON "MISSING": a MISSING tag here means OUR FETCH DID NOT PRODUCE THE DATA.
%   It does NOT mean the data does not exist. Those got conflated -- the figure said
%   "neutralDensity: not published for this satellite/epoch" while itsg_catalog.csv
%   said has_density = yes for the same satellite. A report that contradicts the
%   catalog it was built from is worse than no report: it is a confident claim about
%   the world derived from a failure of our own URL guessing.

%   P = validation.provenance(cfg, W, REF, SAT)
%
%   ---------------------------------------------------------------------------
%   WHY THIS EXISTS
%   ---------------------------------------------------------------------------
%   A validation number is only worth what its inputs are worth, and this pipeline
%   mixes four completely different KINDS of input in one answer:
%
%     MEASURED  a real instrument on the real satellite at the real epoch
%               (ITSG accelerometer, attitude, orbits; TU Delft density)
%     FETCHED   somebody else's measurement, retrieved from an open source
%               (F10.7, Kp/ap, JB2008 indices, EOP, DE440, gravity field)
%     ASSUMED   a number we chose because nobody published one
%               (Cr, GSI wall temperature, optics, an aref_box shape)
%     MISSING   needed, absent, and therefore standing in the way of a model
%
%   Those look identical once they are floats in a struct. They are not identical:
%   a residual explained by an ASSUMED Cr is not a finding, and a model refused for
%   a MISSING input is not a failure. The whole point of this function is that
%   nobody should have to read the code to know which is which -- that is exactly
%   how CHAMP flew at Cd=2.2 while the report printed 3.0.
%
%   Returns P.items: struct array with .name .value .class .source, where class is
%   one of 'measured' | 'fetched' | 'assumed' | 'missing'.
    if nargin<4, SAT = ''; end
    if nargin<5, ledger = {}; end
    % LEDGER: validate_OD's fetched-product list, each entry with .product,
    % .provider, .url, .frame, .nEpochs. It knows WHERE a downloaded product came
    % from; this function knows WHAT KIND every input is, including the assumed
    % ones that are never downloaded. Passing it in keeps ONE account of the
    % question instead of two that agree until someone edits one of them.
    sc = subsref_default(cfg,'spacecraft',struct());
    F  = subsref_default(cfg,'forces',struct());
    d  = subsref_default(F,'drag',struct());
    it = {};

    add = @(n,v,c,s) struct('name',n,'value',v,'class',c,'source',s);

    % ---------- the truth products: MEASURED, or MISSING -----------------------
    prod = {'RDO','reducedDynamicOrbit (metric 1)','reducedDynamicOrbit'; ...
            'KIN','kinematicOrbit (metric 2)','kinematicOrbit'; ...
            'ACC','nonConservativeForces (metric 3)','nonConservativeForces'; ...
            'ATT','attitude -> A(t), Cd(t)','attitude'; ...
            'DEN','neutralDensity (metric 4)','neutralDensity'; ...
            'TUD','TU Delft density (metric 5)','TU Delft density'};
    for i=1:size(prod,1)
        if isfield(REF,prod{i,1}) && ~isempty(REF.(prod{i,1}))
            n = 0;
            try, n = numel(REF.(prod{i,1}).mjd); end
            % use the LEDGER's real provider/frame when we have it, rather than a
            % generic string -- 'ITSG / TU Delft' is true but useless; the actual
            % product and frame are what you check when a number looks wrong
            src = 'ITSG / TU Delft, this satellite & epoch';
            for k=1:numel(ledger)
                L = ledger{k};
                if isstruct(L) && isfield(L,'product') && ...
                   ~isempty(strfind(lower(L.product), lower(prod{i,3})))
                    src = sprintf('%s, frame=%s, %d epochs', ...
                        subsref_default(L,'provider','?'), subsref_default(L,'frame','?'), ...
                        subsref_default(L,'nEpochs',0));
                    break
                end
            end
            it{end+1} = add(prod{i,2}, n, 'measured', src);
        else
            it{end+1} = add(prod{i,2}, 0, 'missing', ['NOT FETCHED. This says our fetch failed -- NOT that the data is absent. ' ...
             'itsg_catalog.csv claims this satellite HAS it. Run data.itsg_products(dir) ' ...
             'to see what the server really holds.']);
        end
    end

    % ---------- space weather: FETCHED (somebody else's measurement) -----------
    sw = subsref_default(W,'drivers',struct());
    atmos_m = lower(subsref_default(d,'atmos','exponential'));
    switch atmos_m
        case 'exponential'
            it{end+1} = add('space weather', 0, 'assumed', ...
                'exponential atmosphere: NO drivers at all, altitude only');
        case {'nrlmsise'}
            it{end+1} = add('F10.7 + F10.7a', NaN, 'fetched', 'OMNI2 (NASA) via data.spaceweather');
            it{end+1} = add('ap / aph history', NaN, 'fetched', 'OMNI2 / GFZ');
        case {'dtm2020'}
            it{end+1} = add('F10.7 + F10.7a', NaN, 'fetched', 'OMNI2 (NASA) -- DTM2020 operational wants F10.7+Kp');
            it{end+1} = add('Kp', NaN, 'fetched', 'GFZ Potsdam');
        case 'dtm2020_research'
            it{end+1} = add('F30 + F30_bar', NaN, 'fetched', 'CLS/LISIRD -- DTM2020 research wants F30+ap60');
            it{end+1} = add('ap60', NaN, 'fetched', 'GFZ Hpo');
        case 'jb2008'
            it{end+1} = add('F10/S10/M10/Y10', NaN, 'fetched', 'Space Environment Technologies SOLFSMY');
            it{end+1} = add('DSTDTC', NaN, 'fetched', 'SET DTCFILE');
    end
    % the CONVENTIONS are choices, not data
    it{end+1} = add('F10.7 lag convention', subsref_tern(subsref_default(cfg.spaceweather,'lag_f107',false), ...
        't-24h (spec)','same-day (GOCE-validated)'), 'assumed', ...
        'atmos.spaceweather opts.lag_f107 -- a CHOICE, recorded in sw.F107_lag');
    it{end+1} = add('aph mode', subsref_default(cfg.spaceweather,'aph_mode','flat'), 'assumed', ...
        'flat vs real 57h history -- a CHOICE, recorded in sw.aph_source');

    % ---------- frames / ephemeris / gravity: FETCHED --------------------------
    it{end+1} = add('EOP (xp, yp, dUT1)', NaN, 'fetched', 'IERS via data.eop_dir');
    it{end+1} = add('Sun/Moon ephemeris', NaN, 'fetched', 'JPL DE440');
    gf = subsref_default(subsref_default(cfg,'gravityField',struct()),'field','default');
    if strcmpi(gf,'default')
        it{end+1} = add('gravity field', 'zonal J2-J6 (bundled)', 'assumed', ...
            'grav.defaultField -- caps at degree 6. A real field needs ICGEM or gravity_data/');
    else
        it{end+1} = add('gravity field', gf, 'fetched', 'ICGEM, cached in data_cache/gravity');
    end

    % ---------- spacecraft: catalog vs assumed --------------------------------
    it{end+1} = add('mass [kg]', subsref_default(sc,'mass',NaN), 'fetched', ...
        'itsg_catalog.csv -- published mission value');
    it{end+1} = add('Aref [m^2]', subsref_default(sc,'Aref',NaN), 'fetched', ...
        'itsg_catalog.csv -- published cross-section');

    dm = lower(subsref_default(d,'model','cannonball'));
    if strcmp(dm,'cannonball')
        it{end+1} = add('Cd', subsref_default(sc,'Cd',NaN), 'assumed', ...
            'itsg_catalog.csv -- a LITERATURE value, not a measurement. Sweep it.');
    else
        it{end+1} = add('Cd(t)', NaN, 'measured-ish', ...
            sprintf('DERIVED by ''%s'' from the gas-surface physics + measured attitude', dm));
        gsi = subsref_default(d,'gsi',struct());
        it{end+1} = add('GSI wall temp Tw [K]', subsref_default(gsi,'Tw',NaN), 'assumed', ...
            'nobody measured this satellite''s surface temperature');
        if any(strcmp(dm,{'sentman'}))
            it{end+1} = add('GSI aT', subsref_default(gsi,'aT',NaN), 'assumed', ...
                'TYPED accommodation. dria/sesam DERIVE it from atomic O instead.');
        elseif any(strcmp(dm,{'dria','sesam'}))
            it{end+1} = add('GSI aT', NaN, 'measured-ish', ...
                'DERIVED by SESAM from the local atomic-oxygen density');
        elseif strcmp(dm,'cll')
            it{end+1} = add('GSI sig_n, sig_t', [subsref_default(gsi,'sig_n',NaN) subsref_default(gsi,'sig_t',NaN)], ...
                'assumed', 'CLL accommodation -- chosen, not measured');
        end
    end

    sm = lower(subsref_default(subsref_default(F,'srp',struct()),'model','cannonball'));
    if strcmp(sm,'cannonball')
        it{end+1} = add('Cr', subsref_default(sc,'Cr',NaN), 'assumed', ...
            'itsg_catalog.csv default 1.3 -- an ASSUMPTION for nearly every satellite');
    else
        it{end+1} = add('facet optics (alpha,rho_s,rho_d)', NaN, 'assumed', ...
            'chosen, not measured. And ONE set is applied to both visible and thermal-IR.');
    end

    if ~isempty(subsref_default(sc,'facets',[]))
        gsrc = subsref_default(W,'geom_note','');
        if ~isempty(strfind(lower(gsrc),'assumed'))
            it{end+1} = add('geometry (box)', numel(sc.facets), 'assumed', gsrc);
        elseif numel(sc.facets)==1
            it{end+1} = add('geometry (plate)', 1, 'fetched', ...
                'one plate of area Aref -- A(t)=Aref*|n.vhat|, no invented shape');
        else
            it{end+1} = add('geometry (box)', numel(sc.facets), 'fetched', ...
                'itsg_catalog.csv geom_dims_m');
        end
    end
    att = subsref_default(sc,'attitude',[]);
    if isstruct(att)
        it{end+1} = add('attitude', numel(att.mjd), 'measured', 'ITSG quaternions, this satellite & epoch');
    elseif ischar(att) && strcmpi(att,'ram')
        it{end+1} = add('attitude', 'ram', 'assumed', 'MODELLED ram-pointing, not the real attitude');
    else
        it{end+1} = add('attitude', 'none', 'missing', 'R_bi = eye(3): panel models would give ~0');
    end

    P.items = [it{:}];
    P.sat = SAT;
    cls = {P.items.class};
    P.n = struct('measured', sum(strcmp(cls,'measured')) + sum(strcmp(cls,'measured-ish')), ...
                 'fetched',  sum(strcmp(cls,'fetched')), ...
                 'assumed',  sum(strcmp(cls,'assumed')), ...
                 'missing',  sum(strcmp(cls,'missing')));
end
