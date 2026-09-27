function names = itsg_products(satDir)
%DATA.ITSG_PRODUCTS  List the product directories the ITSG server has for a satellite.
%   names = data.itsg_products('GRACE-1')
%     -> {'attitude','kinematicOrbit','neutralDensity_1.0','reducedDynamicOrbit',...}
%
%   ---------------------------------------------------------------------------
%   WHY THIS EXISTS
%   ---------------------------------------------------------------------------
%   We were GUESSING product names and reporting the failure of the guess as a fact
%   about the world: "neutralDensity: not published for this satellite/epoch", while
%   itsg_catalog.csv said has_density = yes for the same satellite.
%
%   The real name is  neutralDensity_1.0  --  a VERSION SUFFIX. No amount of
%   guessing was going to produce that, which is the whole argument against guessing:
%   the failure mode is not "we get it wrong sometimes", it is "we cannot get it
%   right and we report our failure as the server's".
%
%   ---------------------------------------------------------------------------
%   ContentType 'text' IS LOAD-BEARING
%   ---------------------------------------------------------------------------
%   Without it, webread sniffs the response and can throw or return something that is
%   not a char for a directory listing -- and this whole function silently returned
%   {} on the user's machine ("could not list the server; falling back to GUESSED
%   names"). The fallback then produced exactly the 404s the listing was added to
%   avoid, so the feature looked implemented and did nothing.
    names = {};
    base = 'https://ftp.tugraz.at/pub/ITSG/satelliteOrbitProducts/operational/';
    try
        opt  = weboptions('ContentType','text', 'Timeout',30);
        html = webread([base satDir '/'], opt);
        if ~ischar(html), html = char(html(:).'); end
        % dots and version digits are part of the name: neutralDensity_1.0
        d = regexp(html, 'href="([A-Za-z_0-9\.\-]+?)/"', 'tokens');
        for k = 1:numel(d)
            n = d{k}{1};
            if any(strcmpi(n, {'..','.'})), continue, end
            if ~isempty(regexp(n, '^(19|20)\d{2}$', 'once')), continue, end  % year dirs
            names{end+1} = n; %#ok<AGROW>
        end
        names = unique(names);
    catch
        % LEGITIMATE swallow: default ({}) set before the try, and the caller checks
        % it and ANNOUNCES the fallback. Offline is a normal state for this function.
    end
end
