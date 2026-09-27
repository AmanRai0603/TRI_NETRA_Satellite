function [a, comp] = accel(rSat, rSun, CrAoM, model, varargin)
%ERP.ACCEL  Earth radiation pressure with switchable model.
%   model : 'knocke' (default) | 'simple' | 'ceres' | 'boxwing'
%
%   knocke/simple/ceres are CANNONBALL models: they collapse the spacecraft to the
%   single number CrAoM = Cr*A/m and CANNOT see attitude. 'boxwing' applies each
%   Earth element's beam to the facets instead, so A and the optical response follow
%   the attitude -- the ERP counterpart of srp 'boxwing' and the panel drag models.
%
%   boxwing takes a different argument list, because CrAoM is meaningless to it:
%       erp.accel(rSat, rSun, [], 'boxwing', R_b2i, sc, doy)
%   Passing CrAoM to boxwing is a category error and errors below rather than being
%   silently ignored.
    if nargin<4||isempty(model), model='knocke'; end
    switch lower(model)
        case 'knocke', [a,comp]=erp.knocke(rSat,rSun,CrAoM,varargin{:});
        case 'simple', [a,comp]=erp.simple(rSat,rSun,CrAoM,varargin{:});
        case 'ceres',  [a,comp]=erp.ceres(rSat,rSun,CrAoM,varargin{:});
        case 'boxwing'
            if numel(varargin) < 2
                error('erp:accel:boxwingArgs', ...
                  ['erp ''boxwing'' needs the attitude and the geometry: ' ...
                   'erp.accel(rSat,rSun,[],''boxwing'',R_b2i,sc[,doy]). It does not ' ...
                   'use CrAoM -- that is the point of it.']);
            end
            [a,comp]=erp.boxwing(rSat,rSun,varargin{1},varargin{2},varargin{3:end});
        otherwise
            error('erp:accel','unknown model "%s". Known: knocke, simple, ceres (all cannonball), boxwing.',model);
    end
end
