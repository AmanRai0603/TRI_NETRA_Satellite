function J = jobs(D)
%ASILS.TRADE.JOBS  The (candidate, seed) jobs of a trade, in a fixed order.
    J = struct('ci', {}, 'cand', {}, 'scenario', {}, 'seed', {});
    for i = 1:numel(D.candidates)
        c = D.candidates{i};
        sc = asils.util.getf(c, 'scenario', asils.util.getf(D, 'scenario', ''));
        for s = D.seeds(:)'
            J(end+1) = struct('ci', i, 'cand', c.id, 'scenario', sc, 'seed', s); %#ok<AGROW>
        end
    end
end
