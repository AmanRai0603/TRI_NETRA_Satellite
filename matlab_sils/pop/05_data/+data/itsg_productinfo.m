function P = itsg_productinfo(product)
%DATA.ITSG_PRODUCTINFO  Public view of the product table (layout, frame, directory).
%   P = data.itsg_productinfo('neutralDensity_1.0')
%     P.name    'neutralDensity'      <- the LAYOUT: columns, frame, sampling
%     P.dir     'neutralDensity_1.0'  <- the DIRECTORY on the server
%
%   Exposed because the version-stripping logic rejected a name the CATALOG had
%   recorded straight off the server, and nothing could test it without a network
%   round-trip and a full validate_OD run. A rule that decides whether data is
%   reachable should be checkable in one line.
    P = data.itsg('__productinfo__', '', product);
end
