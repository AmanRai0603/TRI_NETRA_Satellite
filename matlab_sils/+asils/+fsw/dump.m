function m = dump(h_w, Aw, h_target, B, k, m_max)
%ASILS.FSW.DUMP  Magnetic momentum dumping (cross-product law), ported idea of
%   Standard Code ctrl.rwDump: m = k (dh x B)/|B|^2, dh = Aw h_w - h_target.
%   Direction-preserving saturation.
    dh = Aw*h_w - h_target;
    m = k*asils.util.cross3(dh, B)/(B'*B);
    a = min(1, min(m_max ./ max(abs(m), 1e-30)));
    m = a*m;
end
