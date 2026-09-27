function [y0, y1] = coverage_years(note)
%VALIDATION.COVERAGE_YEARS  Parse '2002-2017' / '2018-present' -> two years. NaN if not.
%
%   The catalog carries a coverage_note that NOTHING READS. GRACE says "2002-2017",
%   2012 is inside that, and the fetch still 404s -- so the note is METADATA NOBODY
%   VERIFIED, sitting in a column, being believed. Parsing it lets the error say
%   "the catalog CLAIMS this date is covered and the server disagrees", which points
%   at the catalog instead of sending you to re-read the URL builder.
%
%   This is a separate function, not a dispatch branch inside itsg_catalog: a
%   string parser is not a catalog lookup. Two jobs, two names -- the same rule the
%   rest of this audit has been applying. Hiding it behind
%   itsg_catalog('coverage_years',...) would also have needed a varargin the
%   signature does not have, which no static check here would have caught.
    y0 = NaN; y1 = NaN;
    if isempty(note) || ~ischar(note), return, end
    t = regexp(note, '(\d{4})\s*-\s*(\d{4}|present)', 'tokens', 'once');
    if isempty(t), return, end
    y0 = str2double(t{1});
    if strcmpi(t{2},'present')
        c = clock(); y1 = c(1);
    else
        y1 = str2double(t{2});
    end
end
