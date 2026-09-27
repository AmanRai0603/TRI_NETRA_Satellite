function T = reads(cfg, verbose)
%SAT.READS  Which spacecraft fields will the CURRENT toggles actually consult?
%   T = sat.reads(cfg)            prints the table, returns it
%   T = sat.reads(cfg, false)     silent
%
%   ---------------------------------------------------------------------------
%   THE QUESTION THIS ANSWERS
%   ---------------------------------------------------------------------------
%   You set Cd = 3.0 and select drag model 'dria'. Is your Cd used? No -- DRIA
%   computes Cd(t) from the gas-surface physics and never looks at sc.Cd. You set
%   Cr = 1.3 and srp 'boxwing'. Is your Cr used? No -- box-wing uses the per-facet
%   optics. You set an attitude and erp 'knocke'. Does ERP see it? No -- there is
%   no ERP box-wing in this tree at all.
%
%   None of that is wrong, but ALL of it is invisible, and a value that is printed
%   in the run report while being ignored by the physics is exactly how CHAMP flew
%   at Cd 2.2 for as long as it did while the report said 3.0. So: one canonical
%   field per quantity (sat.spacecraft), and this function to say which of them the
%   toggles you chose will actually read.
%
%   A field marked (unused) is not an error. It means the model you selected
%   derives that quantity instead of accepting it -- which is usually the reason
%   you selected it.
    if nargin < 2, verbose = true; end
    F  = getf(cfg,'forces',struct());
    sc = getf(cfg,'spacecraft',struct());

    dragOn  = onOf(F,'drag');    dragM  = lower(getf(getf(F,'drag',struct()),'model','cannonball'));
    srpOn   = onOf(F,'srp');     srpM   = lower(getf(getf(F,'srp',struct()),'model','cannonball'));
    erpOn   = onOf(F,'erp');     erpM   = lower(getf(getf(F,'erp',struct()),'model','knocke'));
    att     = getf(sc,'attitude',[]);
    hasAtt  = ~isempty(att);
    hasFac  = ~isempty(getf(sc,'facets',[]));

    R = {};   % force | model | reads | derives | ignores
    if dragOn
        if strcmp(dragM,'cannonball')
            R(end+1,:) = {'drag', dragM, 'mass, Aref, Cd', '-', 'facets, attitude'};
        else
            R(end+1,:) = {'drag', dragM, 'mass, facets, attitude', 'Cd(t), A(t)', 'Aref, Cd'};
        end
    end
    if srpOn
        if strcmp(srpM,'cannonball')
            R(end+1,:) = {'srp', srpM, 'mass, Aref, Cr', '-', 'facets, attitude'};
        else
            R(end+1,:) = {'srp', srpM, 'mass, facets, attitude', 'A(t), optics', 'Aref, Cr'};
        end
    end
    if erpOn
        if strcmp(erpM,'boxwing')
            R(end+1,:) = {'erp', erpM, 'mass, facets, attitude', 'A(t), optics', 'Aref, Cr'};
        else
            R(end+1,:) = {'erp', erpM, 'mass, Aref, Cr', '-', 'facets, attitude (cannonball)'};
        end
    end
    T = R;

    if ~verbose, return, end
    fprintf('\n---- what the CURRENT toggles will actually read from spacecraft ----\n');
    fprintf('  %-6s | %-11s | %-24s | %-13s | %s\n','force','model','READS','DERIVES','IGNORES');
    fprintf('  %s\n', repmat('-',1,92));
    for i = 1:size(R,1)
        fprintf('  %-6s | %-11s | %-24s | %-13s | %s\n', R{i,1}, R{i,2}, R{i,3}, R{i,4}, R{i,5});
    end

    % the traps, named
    fprintf('\n');
    if dragOn && ~strcmp(dragM,'cannonball') && ~hasFac
        fprintf('  [!] drag ''%s'' needs sc.facets and none are set -- forces/drag synthesises a\n', dragM);
        fprintf('      single +x plate of area Aref. That is a choice being made for you.\n');
    end
    if dragOn && ~strcmp(dragM,'cannonball') && ~hasAtt
        fprintf('  [!] drag ''%s'' with NO attitude: R_bi stays eye(3), so the body axes sit on\n', dragM);
        fprintf('      the INERTIAL axes and a +x plate faces inertial +x, not the flow. The\n');
        fprintf('      drag then collapses toward ZERO. Set attitude ''ram'' or ''measured''.\n');
    end
    if srpOn && strcmp(srpM,'boxwing') && hasFac && numel(sc.facets) == 1
        fprintf('  [!] srp ''boxwing'' with ONE facet: a single plate that faces the flow does\n');
        fprintf('      not face the Sun, so SRP -> 0. Correct geometry, useless model.\n');
    end
    if dragOn && ~strcmp(dragM,'cannonball') && ~isempty(getf(sc,'Cd',[]))
        fprintf('  (i) sc.Cd = %g is set but ''%s'' derives Cd(t) instead. Not an error -- that\n', sc.Cd, dragM);
        fprintf('      is why you chose it. It IS why the run report''s Cd can mislead.\n');
    end
    if erpOn && hasAtt && ~strcmp(erpM,'boxwing')
        fprintf('  (i) an attitude is set, but erp ''%s'' is CrAoM-only and cannot use it.\n', erpM);
        fprintf('      erp ''boxwing'' can -- it applies each Earth element''s beam to the facets.\n');
    end
    if erpOn && strcmp(erpM,'boxwing') && ~hasFac
        fprintf('  [!] erp ''boxwing'' needs sc.facets and none are set -- it will error.\n');
    end
end

function tf = onOf(F,nm)
    tf = isfield(F,nm) && isfield(F.(nm),'on') && F.(nm).on;
end
function v = getf(s,f,d)
    if isstruct(s) && isfield(s,f) && ~isempty(s.(f)), v = s.(f); else, v = d; end
end
