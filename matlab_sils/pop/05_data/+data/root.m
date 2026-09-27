function p = root(cmd, val)
%DATA.ROOT  Get or set the single data-cache directory for all fetchers.
%   p = data.root()                 % current cache root (created if missing)
%   data.root('set', '/path/cache') % point every fetcher at your folder
%   data.root('reset')              % back to the default (<toolbox>/data_cache)
%
%   Every fetcher (EOP, space weather, gravity .gfc, TLE) stores under here in a
%   category subfolder, so all downloaded data lives in ONE place you control and
%   is reused on later runs (downloaded once, then read from disk).  No absolute
%   path is hardcoded: the default is derived from this file's location.
    persistent DIR
    if isempty(DIR)
        % this file: <root>/05_data/+data/root.m  -> up three to the toolbox root
        here = fileparts(fileparts(fileparts(mfilename('fullpath'))));
        DIR  = fullfile(here, 'data_cache');
    end
    if nargin>=1
        switch lower(cmd)
            case 'set',   DIR = val;
            case 'reset', here = fileparts(fileparts(fileparts(mfilename('fullpath'))));
                          DIR = fullfile(here,'data_cache');
            case 'get'
            otherwise, error('data:root','unknown command "%s"',cmd);
        end
    end
    if ~exist(DIR,'dir'), mkdir(DIR); end
    p = DIR;
end
