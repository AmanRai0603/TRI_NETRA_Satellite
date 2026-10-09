function p = part_(caseId, tag, kind, name, made)
%ASILS.SIZING.PART_  A sized part's head (the engine's adcs-design part): SZ-<case>-<tag>.
%   Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
    p = struct('part_number', sprintf('SZ-%s-%s', caseId, tag), 'kind', kind, 'name', sprintf('%s — %s', name, caseId), ...
        'status', 'sized', 'source', 'asils.sizing (the design''s laws)', 'made', made, 'descriptor_version', 1);
end
