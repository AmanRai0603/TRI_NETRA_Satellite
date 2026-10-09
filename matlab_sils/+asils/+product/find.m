function f = find(R, kind, id)
%ASILS.PRODUCT.FIND  A catalogue record (data/<kind>/<id>.json); $ADCS_SIZED_DIR/<kind>/<id>.json first when set (the design loop's
%   current iteration); else a case-sized one (store/sized/<case>/<kind>/<id>.json), as the engine's product::find.
%   f = asils.product.find(asils.util.root(), 'parts', 'TRN-GYRO-P1')
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    sd = getenv('ADCS_SIZED_DIR');
    if ~isempty(sd)
        g = fullfile(sd, kind, [id '.json']);
        if exist(g, 'file') == 2, f = g; return, end
    end
    f = fullfile(R, 'data', kind, [id '.json']);
    if exist(f, 'file') == 2, return, end
    d = dir(fullfile(R, 'store', 'sized'));
    names = sort({d([d.isdir]).name});
    for i = 1:numel(names)
        g = fullfile(R, 'store', 'sized', names{i}, kind, [id '.json']);
        if exist(g, 'file') == 2, f = g; return, end
    end
    error('asils:product:missing', 'no %s record %s (data/%s or store/sized/*/%s)', kind(1:end-1), id, kind, kind);
end
