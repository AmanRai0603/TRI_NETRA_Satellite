function [q_body, ok, info] = chain(q_true_body, R_body2head, cat, K, cam)
%ASILS.COMP.STAR_TRACKER.CHAIN  The whole star-tracker chain for one head, the
%   'image' level of asils.devices.star_tracker:
%     render (MODEL SIDE) -> centroid -> identify -> attitude
%   q_true_body   true ECI->body attitude at the exposure (incl. latency)
%   R_body2head   the head's TRUE mounting (misalignment included)
%   The unit reports the body attitude through its NOMINAL mounting, which the
%   caller passes in K.R_head_nominal (so mount errors stay a unit error).
    R_eh = R_body2head*asils.quat.dcm(q_true_body);
    [img, truth] = asils.comp.star_tracker.render(R_eh, cat, cam);
    S = asils.comp.star_tracker.centroid(img, cam);
    ok = false; q_body = q_true_body; info = struct('spots', size(S, 2), 'rendered', size(truth, 2), 'identified', 0);
    if size(S, 2) < 3, return, end
    b = [(S(1,:) - cam.c)/cam.f; (S(2,:) - cam.c)/cam.f; ones(1, size(S, 2))];
    b = b./sqrt(sum(b.^2, 1));
    mag = 6 - 2.5*log10(max(S(3,:), 1)/cam.flux0);
    [id, okid] = asils.comp.star_tracker.identify(b, mag, K, cam);
    info.identified = nnz(id);
    if ~okid, return, end
    [q_eh, ok, id] = asils.comp.star_tracker.attitude(b, id, K, cam.fit_tol_rad);
    info.used = nnz(id);
    if ok
        q_body = asils.quat.fromdcm(K.R_head_nominal'*asils.quat.dcm(q_eh));
    end
end
