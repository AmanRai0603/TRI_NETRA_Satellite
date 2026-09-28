% TIME_FRAMES_XYS06_EXPORT  Export 03_frames_time/+frames/xys06_tables.mat (the
% IAU 2006/2000A X,Y,s series used by frames.eci2ecef_A/B/C) to a compact
% little-endian f64 file for the Rust port (adcs-pop/data/time_frames/xys06.bin).
% Layout (all IEEE-754 binary64, little-endian):
%   [n_xy n_s0 n_s1 n_s2 n_s3 n_s4 0 0]            8 header values
%   xyp (2x6), s_poly (1x6), xy_terms (n_xy x 18),
%   s0..s4 (n_k x 10)                              every matrix ROW-MAJOR
% Run from matlab_sils/pop after setup_paths.
here = fileparts(mfilename('fullpath'));
T = load(fullfile(fileparts(which('frames.eci2ecef_A')), 'xys06_tables.mat'));
out = fullfile(here, '..', 'data', 'time_frames', 'xys06.bin');
fid = fopen(out, 'w', 'ieee-le');
hdr = [size(T.xy_terms,1) size(T.s0,1) size(T.s1,1) size(T.s2,1) size(T.s3,1) size(T.s4,1) 0 0];
fwrite(fid, hdr, 'double');
W = @(M) fwrite(fid, M.', 'double');   % transpose -> column-major write = row-major data
W(T.xyp); W(T.s_poly); W(T.xy_terms); W(T.s0); W(T.s1); W(T.s2); W(T.s3); W(T.s4);
fclose(fid);
fprintf('wrote %s\n', out);
