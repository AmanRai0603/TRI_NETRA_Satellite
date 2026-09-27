function outdir = save_results(tag)
%SAVE_RESULTS  Make (and return) a timestamped results folder under data.root.
%   outdir = save_results('orbit_GOCE_2013-06-01')
%   Figures (.png), tables (.csv) and data (.mat) from the comparison examples are
%   written here so every run is reproducible and auditable.
    root = fullfile(data.root(),'results', tag);
    if ~exist(root,'dir'), mkdir(root); end
    outdir = root;
end
