function d = eop_dir()
%DATA.EOP_DIR  Cache folder for IERS Earth-orientation files (finals/C04/leap).
%   Pass this to the precise frame builds so their EOP downloads land in the same
%   unified cache as everything else:
%     cfg.frame.build='B'; cfg.frame.data_dir = data.eop_dir();
%   The eci2ecef A/B/C builds fetch finals2000A / EOP 20 C04 into this folder once
%   and reuse them (weekly refresh for the rapid tail).
    d = fullfile(data.root(), 'eop');
    if ~exist(d,'dir'), mkdir(d); end
end
