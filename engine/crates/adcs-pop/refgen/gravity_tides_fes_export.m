% refgen/gravity_tides_fes_export.m -- export matlab_sils/pop/force_data/fes2004_deg10.mat
% (the table oceantides.fromModel consumes) to the compact little-endian binary the
% Rust crate embeds: ../data/gravity_tides/fes2004_deg10.bin
%
%   bytes  0..7   magic "POPFES01"
%          8..11  u32 N (rows)          12..15 u32 L (note length)
%          16..   L bytes note (ASCII, the .mat 'note' string)
%          then   N*6 int8  doodson multipliers, row-major [tau s h p N' ps]
%                 N   uint8 degree n,  N uint8 order m
%                 N   f64 Cp, N f64 Sp, N f64 Cm, N f64 Sm   (SI, exactly the .mat doubles)
%
% Run from matlab_sils/pop:
%   octave-cli --no-gui -q --eval "setup_paths; run('<abs>/refgen/gravity_tides_fes_export.m')"
% Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
here = fileparts(mfilename('fullpath'));
if isempty(here), here = pwd; end
pp = strsplit(path, pathsep);          % the POP root = parent of 01_core on the path
popRoot = fileparts(pp{find(~cellfun(@isempty, regexp(pp, '[/\\]01_core$')), 1)});
S = load(fullfile(popRoot, 'force_data', 'fes2004_deg10.mat'));
D = double(S.doodson); n = double(S.n(:)); m = double(S.m(:));
N = numel(n);
assert(isequal(size(D), [N 6]) && all(D(:) == round(D(:))) && all(abs(D(:)) < 128));
assert(all(n >= 0 & n < 256 & m >= 0 & m <= n));
outDir = fullfile(here, '..', 'data', 'gravity_tides');
if ~exist(outDir, 'dir'), mkdir(outDir); end
fn = fullfile(outDir, 'fes2004_deg10.bin');
fid = fopen(fn, 'w', 'ieee-le');
fwrite(fid, 'POPFES01', 'char');
fwrite(fid, N, 'uint32');
fwrite(fid, numel(S.note), 'uint32');
fwrite(fid, S.note, 'char');
fwrite(fid, reshape(D.', 1, []), 'int8');
fwrite(fid, n, 'uint8');
fwrite(fid, m, 'uint8');
fwrite(fid, S.Cp(:), 'float64');
fwrite(fid, S.Sp(:), 'float64');
fwrite(fid, S.Cm(:), 'float64');
fwrite(fid, S.Sm(:), 'float64');
fclose(fid);
% round-trip check
fid = fopen(fn, 'r', 'ieee-le');
mg = char(fread(fid, 8, 'char').'); N2 = fread(fid, 1, 'uint32'); L = fread(fid, 1, 'uint32');
nt = char(fread(fid, L, 'char').');
D2 = reshape(fread(fid, N2*6, 'int8'), 6, []).';
n2 = fread(fid, N2, 'uint8'); m2 = fread(fid, N2, 'uint8');
Cp = fread(fid, N2, 'float64'); Sp = fread(fid, N2, 'float64');
Cm = fread(fid, N2, 'float64'); Sm = fread(fid, N2, 'float64');
fclose(fid);
assert(strcmp(mg, 'POPFES01') && N2 == N && strcmp(nt, S.note));
assert(isequal(D2, D) && isequal(n2, n) && isequal(m2, m));
assert(isequal(Cp, S.Cp(:)) && isequal(Sp, S.Sp(:)) && isequal(Cm, S.Cm(:)) && isequal(Sm, S.Sm(:)));
printf('wrote %s (%d rows, round-trip exact)\n', fn, N);
