function o = adopt(opts, t0, tf)
%INTEG.ADOPT  Fill defaults for adaptive-integrator option structs.
    if nargin<1||isempty(opts), opts=struct(); end
    span=abs(tf-t0);
    def=struct('rtol',1e-9,'atol',1e-12,'hmax',span/2,'hmin',1e-6, ...
               'h0',min(span/100,10),'facmin',0.2,'facmax',5,'maxsteps',2e6);
    fn=fieldnames(def);
    for i=1:numel(fn)
        if ~isfield(opts,fn{i})||isempty(opts.(fn{i})), o.(fn{i})=def.(fn{i});
        else, o.(fn{i})=opts.(fn{i}); end
    end
    o.h0=min(o.h0,o.hmax);
end
