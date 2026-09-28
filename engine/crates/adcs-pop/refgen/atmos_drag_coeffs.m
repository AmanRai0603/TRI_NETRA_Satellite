% ATMOS_DRAG_COEFFS  Export the DTM2020 coefficient tables for the Rust port.
%   Evaluates DTM2020_F107_coeffs_init (operational) and DTM2020_coeffs_init
%   (research) -- the hard-coded MATLAB transcriptions -- and writes every field
%   (tt h he o az2 o2 az t0 tp, 96 terms each) with %.17g, so the doubles the Rust
%   crate parses are bit-identical to the ones Octave holds.
%   Run from matlab_sils/pop after setup_paths:
%     octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/atmos_drag_coeffs.m')"
here = fileparts(mfilename('fullpath'));
outdir = fullfile(here, '..', 'data', 'atmos_drag');
if ~exist(outdir, 'dir'), mkdir(outdir); end
names = {'tt','h','he','o','az2','o2','az','t0','tp'};
sets = {'dtm2020_oper_coeffs.txt',     DTM2020_F107_coeffs_init(), 'DTM2020_F107_coeffs_init (operational, F10.7/Kp)'; ...
        'dtm2020_research_coeffs.txt', DTM2020_coeffs_init(),      'DTM2020_coeffs_init (research, F30/ap60)'};
for s = 1:size(sets,1)
    st = sets{s,2};
    fid = fopen(fullfile(outdir, sets{s,1}), 'w');
    fprintf(fid, '# %s -- exported by refgen/atmos_drag_coeffs.m (%%.17g)\n', sets{s,3});
    fprintf(fid, '# 96 rows (term index 1..96), 9 columns: %s\n', strjoin(names, ' '));
    for i = 1:96
        for j = 1:numel(names)
            v = st.(names{j});
            if j > 1, fprintf(fid, ' '); end
            fprintf(fid, '%.17g', v(i));
        end
        fprintf(fid, '\n');
    end
    fclose(fid);
end
printf('wrote coefficient tables to %s\n', outdir);
