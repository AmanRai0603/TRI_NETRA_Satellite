% EX10  One job, several algorithms, several hardware sets.
%   Each algorithm is registered with its slot and hardware needs
%   (data/algorithms); asils.fsw.select picks one per slot and refuses a
%   choice the product cannot fly. A trade flies every candidate on the same
%   seeds and proposes a winner (docs/SELECTION.md).
% Copyright (c) 2026 Agastya. All rights reserved.
startup_asils
P = asils.config('nadir_hold_ais', 'cases/ais_3u.csv', struct('seed', 1));
disp(rmfield(P.fsw.alg, 'caps'))                  % what the AIS bus flies by default
try                                                % a wheel law on a coils-only bus is refused
    asils.config('nadir_hold_ais', 'cases/ais_3u.csv', struct('set', struct('fsw__algorithms__pointing', 'pid')));
catch e, disp(e.message), end
% one run with another magnetic pointing law, one orbit
r = asils.run('nadir_hold_ais', 'cases/ais_3u.csv', 'set', struct('fsw__algorithms__mtq_pointing', 'mtq_smc', 'sim__duration_s', 5740));
asils.metrics.print(r);
% the Standard Code coils-only Sun acquisition (detumble -> spin-up -> Sun-pointing spin)
r = asils.run('sun_spin_ais', 'cases/ais_3u.csv', 'set', struct('fsw__algorithms__sun_spin', 'sunspin_damped'));
asils.metrics.print(r);
% a whole trade (12 runs, ~1 h): asils.trade.run('trade_mtq_pointing_ais')
