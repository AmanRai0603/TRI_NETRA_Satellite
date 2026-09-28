% EX11  From a customer's case to our ADCS solution.
%   1 size every actuator option to the case (our coils, fluid loop, N2O RCS;
%     benchmark wheels, CMG, VSCMG)
%   2 fly one mission mode with two options, same start, same seed
%   3 (the full matrix, ~50 runs a case: asils.solution.run + collect + dispatch)
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
Z = asils.sizing.size_all('ais_img_3u');                    % prints demand and every family's budget
for opt = {'fmr+rcs', 'rw+mtq'}                             % our fluid loop + RCS vs benchmark wheels
    S = asils.solution.scenario('ais_img_3u', 'sun_referencing', opt{1});
    r = asils.run(S, 'cases/ais_img_3u.csv', 'set', struct('sim__duration_s', 1800));
end
% the complete solution for a case (takes hours; tools/run_matrix.py --only solutions runs it on N cores):
%   asils.solution.run('ais_img_3u'); Sol = asils.solution.collect('ais_img_3u'); asils.solution.dispatch('ais_img_3u');
