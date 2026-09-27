function gfcPath = gravity(model, opts)
%DATA.GRAVITY  Ensure an ICGEM gravity field (.gfc) is cached; return its path.
%   gfcPath = data.gravity(model[, opts])
%     model : known name ('EGM2008','EIGEN-6C4','GOCO06s','GGM05C', ...) OR a
%             direct .gfc URL OR a local file path (returned as-is if it exists).
%     opts  : .url (explicit ICGEM download URL, overrides the registry)
%             .filename (cache name)  .force  .verbose
%
%   ANSWER TO "do I need to keep the .gfc file?":
%   No -- give a known model name (or its ICGEM URL) and this downloads it ONCE
%   into <data.root>/gravity/ and reuses it afterwards. If you are OFFLINE, drop
%   the .gfc into that folder (or anywhere) and pass its path; nothing is fetched.
%   ICGEM download URLs sometimes embed a version hash, so if a name is not in the
%   small registry below, pass opts.url with the exact ICGEM link (Right-click ->
%   copy link on https://icgem.gfz-potsdam.de/tom_longtime).
%
%   The returned path goes straight into cfg.gravityField.field; grav.loadGFC
%   parses it.
    if nargin<2, opts=struct(); end
    % already a usable path?
    if exist(model,'file')==2, gfcPath = model; return; end
    % explicit URL?
    isRegistry = false;
    if ~isempty(strfind(lower(model),'http'))
        url = {model};  fname = getf(opts,'filename', urlLeaf(model));
    elseif isfield(opts,'url') && ~isempty(opts.url)
        url = cellify(opts.url); fname = getf(opts,'filename',[upper(model) '.gfc']);
    else
        [url, fname] = registry(model); isRegistry = true;
    end
    try
        gfcPath = data.ensure('gravity', fname, url, Inf, opts);
    catch e
        if isRegistry
            % ICGEM version hashes change and can 404. Give an ACTIONABLE path
            % instead of a raw download error.
            cd_ = fullfile(data.root(),'gravity');
            error('data:gravity:download', ['could not auto-download %s (%s).\n' ...
              'ICGEM download hashes change over time. Options:\n' ...
              '  1) Use EGM2008 (cfg.gravityField.field=''EGM2008''): degree 2190, works,\n' ...
              '     and for ORBIT propagation it is equivalent to EIGEN-6C4 (<< um/s^2 diff).\n' ...
              '  2) Download %s from https://icgem.gfz.de/tom_longtime (right-click the\n' ...
              '     ''gfc'' link -> Save link as) and place it at:\n       %s\n' ...
              '  3) Pass the exact link:  cfg.gravityField.field = ''<the .gfc URL>''.'], ...
              fname, regexprep(e.message,'\n.*',''), fname, fullfile(cd_,fname));
        else
            rethrow(e);
        end
    end
end

function [url, fname] = registry(model)
% Minimal registry of stable direct ICGEM links. Extend as needed; if a model is
% missing, pass opts.url with the ICGEM link instead of guessing.
    m = upper(strrep(model,' ',''));
    % Each model lists BOTH the new (gfz.de) and legacy (gfz-potsdam.de) hosts as
    % candidates; data.ensure tries them in order. Hashes are ICGEM version tags.
    R = { ...
      'EGM2008', {'https://icgem.gfz.de/getmodel/gfc/c50128797a9cb62e936337c890e4425f03f0461d7329b09a8cc8561504465340/EGM2008.gfc', ...
                  'https://icgem.gfz-potsdam.de/getmodel/gfc/c50128797a9cb62e936337c890e4425f03f0461d7329b09a8cc8561504465340/EGM2008.gfc'}, 'EGM2008.gfc'; ...
      'EIGEN-6C4', {'https://icgem.gfz.de/getmodel/gfc/7fd8fe44aa1518cd79ca84300aef4b41ddb2364aef9e82b7cdaabdb3aa0a3c02/EIGEN-6C4.gfc', ...
                    'https://icgem.gfz-potsdam.de/getmodel/gfc/7fd8fe44aa1518cd79ca84300aef4b41ddb2364aef9e82b7cdaabdb3aa0a3c02/EIGEN-6C4.gfc'}, 'EIGEN-6C4.gfc'; ...
      'GOCO06S', {'https://icgem.gfz.de/getmodel/gfc/32ec2884630a02670476f752d2a2062a76d16aabee40e28f37c3892a5b2b6ac9/GOCO06s.gfc', ...
                  'https://icgem.gfz-potsdam.de/getmodel/gfc/32ec2884630a02670476f752d2a2062a76d16aabee40e28f37c3892a5b2b6ac9/GOCO06s.gfc'}, 'GOCO06s.gfc' };
    k = find(strcmpi(R(:,1), m),1);
    if isempty(k)
        error('data:gravity:unknownModel', ...
          ['unknown gravity model "%s". Pass a .gfc path, a direct URL, or ' ...
           'opts.url with the ICGEM link. Known: %s'], model, strjoin(R(:,1)',', '));
    end
    url = R{k,2}; fname = R{k,3};
end

function s=urlLeaf(u), p=strsplit(u,'/'); s=p{end}; if isempty(strfind(s,'.gfc')), s=[s '.gfc']; end, end
function c=cellify(x), if ischar(x), c={x}; else, c=x; end, end
function val=getf(s,f,d), if isfield(s,f)&&~isempty(s.(f)), val=s.(f); else, val=d; end, end
