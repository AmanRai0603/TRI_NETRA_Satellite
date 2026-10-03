function cfg = forcePresets(name)
%CONFIG.FORCEPRESETS  Named force-model presets built on config.defaultConfig.
%   cfg = config.forcePresets(name)
%     'twobody'     central point mass only
%     'j2'          zonal J2 only, fast
%     'leo_precise' degree-20 gravity, Sun+Moon, cannonball drag+SRP, relativity
%     'leo_full'   full stack: sphharm gravity, 3rd body, panel drag (dria) on
%                   nrlmsise, boxwing SRP, ERP, relativity, solid+ocean tides
%     'gnss_meo'    high-orbit set: degree-12 gravity, 3rd body, SRP, relativity
%   These set toggles/models only; edit epoch/state/spacecraft as needed.
    cfg = config.defaultConfig();
    switch lower(name)
        case 'twobody'
            cfg.forces.gravity.model='twobody';
            cfg.forces.thirdbody.on=false; cfg.forces.drag.on=false; cfg.forces.srp.on=false;
        case 'j2'
            cfg.forces.gravity.model='j2';
            cfg.forces.thirdbody.on=false; cfg.forces.drag.on=false; cfg.forces.srp.on=false;
        case 'leo_precise'
            cfg.gravityField.degree=20;
            cfg.forces.gravity=struct('on',true,'model','sphharm','degree',20,'order',20);
            cfg.forces.thirdbody.on=true; cfg.forces.drag.on=true; cfg.forces.srp.on=true;
            cfg.forces.relativity.on=true;
        case 'leo_full'
            cfg.gravityField.degree=20;
            cfg.forces.gravity=struct('on',true,'model','sphharm','degree',20,'order',20);
            cfg.forces.thirdbody.on=true;
            cfg.forces.drag=struct('on',true,'model','dria','atmos','nrlmsise', ...
                                   'gsi',struct('Tw',300),'corotate',true);
            cfg.forces.srp=struct('on',true,'model','boxwing','eclipse','conical');
            cfg.forces.erp.on=true; cfg.forces.relativity.on=true;
            cfg.forces.solidtides.on=true; cfg.forces.oceantides.on=true;
            % panel geometry for drag+SRP box-wing
            cfg.spacecraft.facets = [];        % fill via dgeom.sat16u / sgeom.sat16u (or your own)
        case 'gnss_meo'
            cfg.r0=[26560000;0;0]; cfg.v0=[0;3870;0]; cfg.tspan=43200;
            cfg.gravityField.degree=12;
            cfg.forces.gravity=struct('on',true,'model','sphharm','degree',12,'order',12);
            cfg.forces.thirdbody.on=true; cfg.forces.drag.on=false;
            cfg.forces.srp.on=true; cfg.forces.relativity.on=true;
        otherwise
            error('config:forcePresets','unknown preset "%s"',name);
    end
end
