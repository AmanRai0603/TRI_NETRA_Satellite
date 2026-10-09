function S = stated()
%ASILS.UTIL.STATED  The design's stated values the twin reads (data/stated.json, adcs-stated/2, written from the design's
%   stated nodes by tools/design_build.py), as the engine's adcs-sim stated.rs reads them: a node's number, list, flag,
%   whole number or x/y/z triple, each refused by name when the design states none. Read once a session.
%   S.get(node), S.list(node, n), S.flag(node), S.whole(node, lo, hi), S.v3(prefix)
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    persistent V F
    f = fullfile(asils.util.root(), 'data', 'stated.json');
    if isempty(V) || ~strcmp(F, f)
        j = asils.util.readjson(f);
        assert(strcmp(asils.util.getf(j, 'schema', ''), 'adcs-stated/2'), 'asils:stated:schema', '%s: not an adcs-stated/2 file', f);
        V = j.value; F = f;
    end
    S.get = @(node) get_(V, f, node);
    S.list = @(node, n) list_(V, f, node, n);
    S.flag = @(node) flag_(V, f, node);
    S.whole = @(node, lo, hi) whole_(V, f, node, lo, hi);
    S.v3 = @(prefix) [get_(V, f, [prefix '_x']); get_(V, f, [prefix '_y']); get_(V, f, [prefix '_z'])];
end

function x = get_(V, f, node)
    assert(isfield(V, node), 'asils:stated:missing', 'the design states no value for %s (%s), which the twin needs', node, f);
    x = V.(node);
    assert(isnumeric(x) && isscalar(x), 'asils:stated:list', '%s states a list (%s), where the twin needs a number', node, f);
end
function x = list_(V, f, node, n)
    assert(isfield(V, node), 'asils:stated:missing', 'the design states no value for %s (%s), which the twin needs', node, f);
    x = V.(node)(:);
    assert(numel(x) == n, 'asils:stated:list', '%s states %d number(s) (%s), where the twin needs a list of %d', node, numel(x), f, n);
end
function b = flag_(V, f, node)
    x = get_(V, f, node);
    assert(x == 0 || x == 1, 'asils:stated:flag', '%s = %g (%s): 1 (yes) or 0 (no)', node, x, f);
    b = x == 1;
end
function x = whole_(V, f, node, lo, hi)
    x = get_(V, f, node);
    assert(x == round(x) && x >= lo && x <= hi, 'asils:stated:whole', '%s = %g (%s): a whole number from %d to %d', node, x, f, lo, hi);
end
