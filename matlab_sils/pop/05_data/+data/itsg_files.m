function files = itsg_files(satDir, product, year)
%DATA.ITSG_FILES  List the actual filenames in one product/year directory.
%   files = data.itsg_files('CHAMP','neutralDensity_1.0',2007)
%
%   The directory name and the FILE name are not the same string. The product lives
%   in  .../CHAMP/neutralDensity_1.0/2007/  but the file inside is not necessarily
%   CHAMP_neutralDensity_1.0_2007-01-01.txt.gz -- the version may not repeat, the
%   separator may differ, the date format may differ.
%
%   Building a filename from a directory name is a GUESS about someone else's naming
%   convention. We already made that mistake once and reported it as "the data does
%   not exist". So: list the directory, match the date, use what is actually there.
    files = {};
    base = 'https://ftp.tugraz.at/pub/ITSG/satelliteOrbitProducts/operational/';
    url  = sprintf('%s%s/%s/%d/', base, satDir, product, year);
    try
        opt  = weboptions('ContentType','text', 'Timeout',30);
        html = webread(url, opt);
        if ~ischar(html), html = char(html(:).'); end
        d = regexp(html, 'href="([^"/?][^"]*\.(?:gz|txt|dat))"', 'tokens');
        for k = 1:numel(d), files{end+1} = d{k}{1}; end %#ok<AGROW>
        files = unique(files);
    catch
        % LEGITIMATE swallow: default set before the try; caller announces the miss.
    end
end
