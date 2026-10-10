function [n, mode_ix, act, dump_rcs, feasible, has_slot, tunable, has_obj, objective, has_req, req, f_perf, f_know, f_power, f_prop, f_rate, in_v, tuned, vstart, vval, vperf, vrate, order, p_act, p_feas, p_vstart, p_vval, p_vperf, p_cur, lu_part, lu_scale, lm_before, lm_use, has_scale, scale, fam_gaps, fam_simplicity, fam_mass, fam_power, fam_fmr, fam_use, ek, eo, ep, ea, eb, ec, ef, o_lost, o_feas] = loop_converge(mode_ix, act, dump_rcs, feasible, has_slot, tunable, has_obj, objective, has_req, req, f_perf, f_know, f_power, f_prop, f_rate, in_v, tuned, vstart, vval, vperf, vrate, order, no, nm, p_act, p_feas, p_vstart, p_vval, p_vperf, p_cur, np, lu_part, lu_scale, nlu, lg_has, lg_g0, lg_vb, closed_gyro, closed_heads, closed_lambda, closed_scale_fmr, lam_up, lam_down, st2, frozen, tried_up, tried_down, lm_has, lm_key, lm_old, lm_vb, lm_before, lm_use, has_gyro, gyro, has_sigma, sigma, has_lambda, lambda, star_tracker, has_heads, heads, has_scale, scale, k0_has_gyro, k0_gyro, k0_has_sigma, k0_sigma, k0_has_lambda, k0_lambda, fine, has_sel, nf, fam_gaps, fam_simplicity, fam_mass, fam_power, fam_fmr, fam_use, improvement, scale_min, scale_max, up, down, lambda_min, lambda_max, sigma_min, gyro_min, margin, ek, eo, ep, ea, eb, ec, ef, o_lost, o_feas)
%LOOP_CONVERGE  The knob changes the failures of an iteration call for (pipeline_design's node_converge).
%   The iteration's no options, in the order they were flown: each one's mode (mode_ix, of nm modes), actuator (act),
%   whether it dumps by thrusters (dump_rcs), whether it is feasible, has an algorithm slot (has_slot) of a tuned slot on a
%   tuned actuator (tunable), its objective (has_obj) and requirement (has_req), the classes of its failures (f_perf,
%   f_know, f_power, f_prop, f_rate: a rate-stability metric), whether it flies its slot's every algorithm (in_v) and is
%   tuned (tuned), its violations vval[vstart[i] .. vstart[i + 1]] (vperf: a performance metric's; vrate: a rate-stability
%   metric's), and order: the options by mode and option id.
%   The iteration before the authority moves (prev): its np options, their actuators, feasibility, violations and each
%   one's place among this iteration's (p_cur, -1: none); the parts it moved up (lu_part, in order) and their scales.
%   The history: the gyro move to judge (lg_has, lg_g0, lg_vb), the levers closed, the moves made (lam_up, lam_down, st2),
%   the parts frozen (frozen), tried up or down (tried_up, tried_down, bit p of each), the mass move to judge (lm_has,
%   lm_key, lm_vb; lm_before: the options feasible then; lm_use: those its family could use).
%   The knobs (with has_*: whether set; else the sizing's default), and the knobs the iteration started from (k0_*).
%   The fine class; the solution families (nf): gap counts, simplicity, a mass gap, a power gap, a fluid loop, and bit f
%   of fam_use[i] whether family f can use option i.
%   The registry's parameters: the improvement needed, the scale's bounds and steps, the pump's bounds, the flow sensor's
%   floor, the gyro grade's floor, the tune margin.
%   The events (ek .. ef) and, per option, the mass move's flags out (o_lost: an option it broke; o_feas: feasible in the
%   family that takes a mass lever); how many events.
%   length: 16
%   mode_ix: inout int[*] (passed a whole number)
%   act: inout int[*] (passed a whole number)
%   dump_rcs: inout int[*] (passed a whole number)
%   feasible: inout int[*] (passed a whole number)
%   has_slot: inout int[*] (passed a whole number)
%   tunable: inout int[*] (passed a whole number)
%   has_obj: inout int[*] (passed a whole number)
%   objective: inout real[1][*] (passed a plain number)
%   has_req: inout int[*] (passed a whole number)
%   req: inout real[1][*] (passed a plain number)
%   f_perf: inout int[*] (passed a whole number)
%   f_know: inout int[*] (passed a whole number)
%   f_power: inout int[*] (passed a whole number)
%   f_prop: inout int[*] (passed a whole number)
%   f_rate: inout int[*] (passed a whole number)
%   in_v: inout int[*] (passed a whole number)
%   tuned: inout int[*] (passed a whole number)
%   vstart: inout int[*] (passed a whole number)
%   vval: inout real[1][*] (passed a plain number)
%   vperf: inout int[*] (passed a whole number)
%   vrate: inout int[*] (passed a whole number)
%   order: inout int[*] (passed a whole number)
%   no: int (passed a whole number)
%   nm: int (passed a whole number)
%   p_act: inout int[*] (passed a whole number)
%   p_feas: inout int[*] (passed a whole number)
%   p_vstart: inout int[*] (passed a whole number)
%   p_vval: inout real[1][*] (passed a plain number)
%   p_vperf: inout int[*] (passed a whole number)
%   p_cur: inout int[*] (passed a whole number)
%   np: int (passed a whole number)
%   lu_part: inout int[*] (passed a whole number)
%   lu_scale: inout real[1][*] (passed a plain number)
%   nlu: int (passed a whole number)
%   lg_has: bool (passed true or false)
%   lg_g0: real[1] (passed a plain number)
%   lg_vb: real[1] (passed a plain number)
%   closed_gyro: bool (passed true or false)
%   closed_heads: bool (passed true or false)
%   closed_lambda: bool (passed true or false)
%   closed_scale_fmr: bool (passed true or false)
%   lam_up: bool (passed true or false)
%   lam_down: bool (passed true or false)
%   st2: bool (passed true or false)
%   frozen: int (passed a whole number)
%   tried_up: int (passed a whole number)
%   tried_down: int (passed a whole number)
%   lm_has: bool (passed true or false)
%   lm_key: MassLever (passed a choice, as its option's number)
%   lm_old: real[1] (passed a plain number)
%   lm_vb: real[1] (passed a plain number)
%   lm_before: inout int[*] (passed a whole number)
%   lm_use: inout int[*] (passed a whole number)
%   has_gyro: bool (passed true or false)
%   gyro: real[1] (passed a plain number)
%   has_sigma: bool (passed true or false)
%   sigma: real[1] (passed a plain number)
%   has_lambda: bool (passed true or false)
%   lambda: real[1] (passed a plain number)
%   star_tracker: bool (passed true or false)
%   has_heads: bool (passed true or false)
%   heads: real[1] (passed a plain number)
%   has_scale: inout int[*] (passed a whole number)
%   scale: inout real[1][*] (passed a plain number)
%   k0_has_gyro: bool (passed true or false)
%   k0_gyro: real[1] (passed a plain number)
%   k0_has_sigma: bool (passed true or false)
%   k0_sigma: real[1] (passed a plain number)
%   k0_has_lambda: bool (passed true or false)
%   k0_lambda: real[1] (passed a plain number)
%   fine: bool (passed true or false)
%   has_sel: bool (passed true or false)
%   nf: int (passed a whole number)
%   fam_gaps: inout int[*] (passed a whole number)
%   fam_simplicity: inout real[1][*] (passed a plain number)
%   fam_mass: inout int[*] (passed a whole number)
%   fam_power: inout int[*] (passed a whole number)
%   fam_fmr: inout int[*] (passed a whole number)
%   fam_use: inout int[*] (passed a whole number)
%   improvement: real[1] (passed a plain number)
%   scale_min: real[1] (passed a plain number)
%   scale_max: real[1] (passed a plain number)
%   up: real[1] (passed a plain number)
%   down: real[1] (passed a plain number)
%   lambda_min: real[1] (passed a plain number)
%   lambda_max: real[1] (passed a plain number)
%   sigma_min: real[1] (passed a plain number)
%   gyro_min: real[1] (passed a plain number)
%   margin: real[1] (passed a plain number)
%   ek: inout int[*] (passed a whole number)
%   eo: inout int[*] (passed a whole number)
%   ep: inout int[*] (passed a whole number)
%   ea: inout real[1][*] (passed a plain number)
%   eb: inout real[1][*] (passed a plain number)
%   ec: inout real[1][*] (passed a plain number)
%   ef: inout int[*] (passed a whole number)
%   o_lost: inout int[*] (passed a whole number)
%   o_feas: inout int[*] (passed a whole number)
%   returns n: int (passed a whole number)
%   Generated by tools/pcode.py from the pseudocode (trinetra-pcode/2); do not edit.
    n = 0;
    [d_k_tau, d_lambda, d_heads, d_sigma, d_grade, b0, b1, b2, b3, b4, b5, b6, b7, b8, b9] = asils.models.sizedemand.sizing_knobs();
    improve = (1 - improvement);
    h__61 = no;
    h__62 = (numel(vstart) - 1);
    if h__62 < h__61, h__61 = h__62; end
    cap = h__61;
    n = 0;
    changes = 0;
    if has_gyro
        h__63 = gyro;
    else
        h__63 = d_grade;
    end
    k_gyro = h__63;
    if has_sigma
        h__64 = sigma;
    else
        h__64 = d_sigma;
    end
    k_sigma = h__64;
    if has_lambda
        h__65 = lambda;
    else
        h__65 = d_lambda;
    end
    k_lambda = h__65;
    if has_heads
        h__66 = heads;
    else
        h__66 = d_heads;
    end
    k_heads = h__66;
    k_st = star_tracker;
    if k0_has_gyro
        h__67 = k0_gyro;
    else
        h__67 = d_grade;
    end
    k0g = h__67;
    if k0_has_sigma
        h__68 = k0_sigma;
    else
        h__68 = d_sigma;
    end
    k0s = h__68;
    if k0_has_lambda
        h__69 = k0_lambda;
    else
        h__69 = d_lambda;
    end
    k0l = h__69;
    sc = zeros(7, 1);
    for p = (0):((7) - 1)
        sc((p) + 1) = 1;
        if (((p < numel(has_scale)) && (p < numel(scale))) && (has_scale((p) + 1) == 1))
            sc((p) + 1) = scale((p) + 1);
        end
    end
    closed_g = closed_gyro;
    lup = lam_up;
    ldown = lam_down;
    frz = frozen;
    tried_dn = tried_down;
    nv = 0;
    if ((cap >= 0) && (cap < numel(vstart)))
        nv = vstart((cap) + 1);
    end
    i = ((0):((asils.pc.fmin(cap, numel(o_lost))) - 1)).';
    if ~isempty(i)
        o_lost((i) + 1) = 0;
    end
    i = ((0):((asils.pc.fmin(cap, numel(o_feas))) - 1)).';
    if ~isempty(i)
        o_feas((i) + 1) = 0;
    end
    if lg_has
        [t__56, t__57, t__58] = asils.models.looprules.loop_rate_violation(vval, vrate, nv);
        vn = t__56;
        vval = t__57;
        vrate = t__58;
        if (vn > (improve * lg_vb))
            k_gyro = lg_g0;
            closed_g = true;
            [t__59, t__60, t__61, t__62, t__63, t__64, t__65, t__66] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 0, (-(1)), (-(1)), lg_g0, lg_vb, vn, 0);
            n = t__59;
            ek = t__60;
            eo = t__61;
            ep = t__62;
            ea = t__63;
            eb = t__64;
            ec = t__65;
            ef = t__66;
            changes = (changes + 1);
        end
    end
    h__70 = nlu;
    h__71 = numel(lu_part);
    if h__71 < h__70, h__70 = h__71; end
    for u = (0):((h__70) - 1)
        part = lu_part((u) + 1);
        before = 0;
        now = 0;
        pv = 0;
        h__72 = np;
        h__73 = (numel(p_vstart) - 1);
        if h__73 < h__72, h__72 = h__73; end
        for j = (0):((h__72) - 1)
            if ((asils.models.looprules.loop_part(p_act((j) + 1)) == part) && (p_feas((j) + 1) == 0))
                [t__67, t__68, t__69] = asils.models.looprules.loop_option_violation(p_vval, p_vperf, p_vstart((j) + 1), p_vstart(((j + 1)) + 1), true);
                pv = t__67;
                p_vval = t__68;
                p_vperf = t__69;
                before = (before + pv);
                c = p_cur((j) + 1);
                if ((c >= 0) && (c < cap))
                    [t__70, t__71, t__72] = asils.models.looprules.loop_option_violation(vval, vperf, vstart((c) + 1), vstart(((c + 1)) + 1), true);
                    pv = t__70;
                    vval = t__71;
                    vperf = t__72;
                end
                now = (now + pv);
            end
        end
        if ((before > 0) && (now > (improve * before)))
            if ((part >= 0) && (part < 7))
                sc((part) + 1) = lu_scale((u) + 1);
                frz = bitor(frz, bitshift(1, part));
            end
            [t__73, t__74, t__75, t__76, t__77, t__78, t__79, t__80] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 1, (-(1)), part, lu_scale((u) + 1), before, now, 0);
            n = t__73;
            ek = t__74;
            eo = t__75;
            ep = t__76;
            ea = t__77;
            eb = t__78;
            ec = t__79;
            ef = t__80;
            changes = (changes + 1);
        end
    end
    want = zeros(7, 1);
    for jj = (0):((cap) - 1)
        i = order((jj) + 1);
        if ((i >= 0) && (i < cap))
            pt = asils.models.looprules.loop_part(act((i) + 1));
            if (feasible((i) + 1) == 1)
                if (((((tunable((i) + 1) == 1) && (has_req((i) + 1) == 1)) && (req((i) + 1) > 0)) && (has_obj((i) + 1) == 1)) && (objective((i) + 1) > (margin * req((i) + 1))))
                    if (in_v((i) + 1) == 0)
                        in_v((i) + 1) = 1;
                        [t__81, t__82, t__83, t__84, t__85, t__86, t__87, t__88] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 2, i, (-(1)), objective((i) + 1), req((i) + 1), 0, 0);
                        n = t__81;
                        ek = t__82;
                        eo = t__83;
                        ep = t__84;
                        ea = t__85;
                        eb = t__86;
                        ec = t__87;
                        ef = t__88;
                        changes = (changes + 1);
                    elseif (tuned((i) + 1) == 0)
                        tuned((i) + 1) = 1;
                        [t__89, t__90, t__91, t__92, t__93, t__94, t__95, t__96] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 3, i, (-(1)), objective((i) + 1), req((i) + 1), 0, 0);
                        n = t__89;
                        ek = t__90;
                        eo = t__91;
                        ep = t__92;
                        ea = t__93;
                        eb = t__94;
                        ec = t__95;
                        ef = t__96;
                        changes = (changes + 1);
                    end
                end
            else
                if (f_perf((i) + 1) == 1)
                    if ((has_slot((i) + 1) == 1) && (in_v((i) + 1) == 0))
                        in_v((i) + 1) = 1;
                        [t__97, t__98, t__99, t__100, t__101, t__102, t__103, t__104] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 4, i, (-(1)), 0, 0, 0, 0);
                        n = t__97;
                        ek = t__98;
                        eo = t__99;
                        ep = t__100;
                        ea = t__101;
                        eb = t__102;
                        ec = t__103;
                        ef = t__104;
                        changes = (changes + 1);
                    elseif ((tunable((i) + 1) == 1) && (tuned((i) + 1) == 0))
                        tuned((i) + 1) = 1;
                        [t__105, t__106, t__107, t__108, t__109, t__110, t__111, t__112] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 5, i, (-(1)), 0, 0, 0, 0);
                        n = t__105;
                        ek = t__106;
                        eo = t__107;
                        ep = t__108;
                        ea = t__109;
                        eb = t__110;
                        ec = t__111;
                        ef = t__112;
                        changes = (changes + 1);
                    elseif (f_power((i) + 1) == 1)
                        [t__113, t__114, t__115, t__116, t__117, t__118, t__119, t__120] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 6, i, (-(1)), 0, 0, 0, 0);
                        n = t__113;
                        ek = t__114;
                        eo = t__115;
                        ep = t__116;
                        ea = t__117;
                        eb = t__118;
                        ec = t__119;
                        ef = t__120;
                    elseif (((fine && (f_rate((i) + 1) == 1)) && (k_gyro > (gyro_min * 1.01))) && (~closed_g))
                        if (k_gyro == k0g)
                            g0 = k_gyro;
                            h__74 = gyro_min;
                            h__75 = asils.models.looprules.loop_round3((g0 * 0.3));
                            if h__75 > h__74, h__74 = h__75; end
                            k_gyro = h__74;
                            [t__121, t__122, t__123] = asils.models.looprules.loop_rate_violation(vval, vrate, nv);
                            rv = t__121;
                            vval = t__122;
                            vrate = t__123;
                            [t__124, t__125, t__126, t__127, t__128, t__129, t__130, t__131] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 7, i, (-(1)), g0, k_gyro, rv, 0);
                            n = t__124;
                            ek = t__125;
                            eo = t__126;
                            ep = t__127;
                            ea = t__128;
                            eb = t__129;
                            ec = t__130;
                            ef = t__131;
                            changes = (changes + 1);
                        end
                    elseif ((act((i) + 1) == fix(4)) && (k_sigma > (sigma_min * 1.01)))
                        sg = k_sigma;
                        if (sg == k0s)
                            h__76 = sigma_min;
                            h__77 = (sg / 4);
                            if h__77 > h__76, h__76 = h__77; end
                            k_sigma = h__76;
                            [t__132, t__133, t__134, t__135, t__136, t__137, t__138, t__139] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 8, i, (-(1)), sg, k_sigma, 0, 0);
                            n = t__132;
                            ek = t__133;
                            eo = t__134;
                            ep = t__135;
                            ea = t__136;
                            eb = t__137;
                            ec = t__138;
                            ef = t__139;
                            changes = (changes + 1);
                        end
                    elseif ((pt >= 0) && (pt < 7))
                        if (want((pt) + 1) == 0)
                            want((pt) + 1) = 1;
                        end
                    end
                end
                if (f_know((i) + 1) == 1)
                    if ((~fine) && (~k_st))
                        k_st = true;
                        [t__140, t__141, t__142, t__143, t__144, t__145, t__146, t__147] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 9, i, (-(1)), 0, 0, 0, 0);
                        n = t__140;
                        ek = t__141;
                        eo = t__142;
                        ep = t__143;
                        ea = t__144;
                        eb = t__145;
                        ec = t__146;
                        ef = t__147;
                        changes = (changes + 1);
                    else
                        [t__148, t__149, t__150, t__151, t__152, t__153, t__154, t__155] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 10, i, (-(1)), 0, 0, 0, 0);
                        n = t__148;
                        ek = t__149;
                        eo = t__150;
                        ep = t__151;
                        ea = t__152;
                        eb = t__153;
                        ec = t__154;
                        ef = t__155;
                    end
                end
                if (f_power((i) + 1) == 1)
                    lam = k_lambda;
                    fmr = (act((i) + 1) == fix(4));
                    if (fmr && (k_lambda ~= k0l))
                        lam = k_lambda;
                    elseif (fmr && (dump_rcs((i) + 1) == 1))
                        [t__156, t__157, t__158, t__159, t__160, t__161, t__162, t__163] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 11, i, (-(1)), 0, 0, 0, 0);
                        n = t__156;
                        ek = t__157;
                        eo = t__158;
                        ep = t__159;
                        ea = t__160;
                        eb = t__161;
                        ec = t__162;
                        ef = t__163;
                    elseif ((fmr && (lam < lambda_max)) && (~ldown))
                        h__78 = lambda_max;
                        h__79 = (lam * 3);
                        if h__79 < h__78, h__78 = h__79; end
                        k_lambda = h__78;
                        lup = true;
                        [t__164, t__165, t__166, t__167, t__168, t__169, t__170, t__171] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 12, i, (-(1)), lam, k_lambda, 0, 0);
                        n = t__164;
                        ek = t__165;
                        eo = t__166;
                        ep = t__167;
                        ea = t__168;
                        eb = t__169;
                        ec = t__170;
                        ef = t__171;
                        changes = (changes + 1);
                    elseif fmr
                        if ldown
                            h__80 = 1;
                        else
                            h__80 = 0;
                        end
                        [t__172, t__173, t__174, t__175, t__176, t__177, t__178, t__179] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 13, i, (-(1)), lam, 0, 0, h__80);
                        n = t__172;
                        ek = t__173;
                        eo = t__174;
                        ep = t__175;
                        ea = t__176;
                        eb = t__177;
                        ec = t__178;
                        ef = t__179;
                    elseif ((f_perf((i) + 1) == 0) && (pt == fix(3)))
                        if (want((pt) + 1) == 0)
                            want((pt) + 1) = 2;
                        end
                    elseif (f_perf((i) + 1) == 0)
                        [t__180, t__181, t__182, t__183, t__184, t__185, t__186, t__187] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 14, i, pt, 0, 0, 0, 0);
                        n = t__180;
                        ek = t__181;
                        eo = t__182;
                        ep = t__183;
                        ea = t__184;
                        eb = t__185;
                        ec = t__186;
                        ef = t__187;
                    end
                end
                if (f_prop((i) + 1) == 1)
                    want((fix(4)) + 1) = 1;
                end
            end
        end
    end
    closed_h = closed_heads;
    closed_l = closed_lambda;
    closed_f = closed_scale_fmr;
    if lm_has
        lost = 0;
        h__81 = cap;
        h__82 = numel(o_lost);
        if h__82 < h__81, h__81 = h__82; end
        for i = (0):((h__81) - 1)
            if (((lm_before((i) + 1) == 1) && (lm_use((i) + 1) == 1)) && (feasible((i) + 1) == 0))
                o_lost((i) + 1) = 1;
                lost = (lost + 1);
            end
        end
        [t__188, t__189, t__190, t__191, t__192, t__193, t__194] = asils.models.looprules.loop_family_violation(mode_ix, feasible, lm_use, 0, vstart, vval, vperf, cap, nm);
        v_now = t__188;
        mode_ix = t__189;
        feasible = t__190;
        lm_use = t__191;
        vstart = t__192;
        vval = t__193;
        vperf = t__194;
        worse = (v_now > (((2 - improve) * lm_vb) + 1e-9));
        if ((lost > 0) || worse)
            if (lm_key == 0)
                closed_g = true;
                k_gyro = lm_old;
            elseif (lm_key == 1)
                closed_h = true;
                k_heads = lm_old;
            elseif (lm_key == 2)
                closed_l = true;
                k_lambda = lm_old;
            else
                closed_f = true;
                sc((fix(1)) + 1) = lm_old;
            end
            if worse
                h__83 = 1;
            else
                h__83 = 0;
            end
            [t__195, t__196, t__197, t__198, t__199, t__200, t__201, t__202] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 15, (-(1)), (-(1)), lm_vb, v_now, 0, h__83);
            n = t__195;
            ek = t__196;
            eo = t__197;
            ep = t__198;
            ea = t__199;
            eb = t__200;
            ec = t__201;
            ef = t__202;
            changes = (changes + 1);
        end
    end
    if ((has_sel && (changes == 0)) && (nf > 0))
        f = 0;
        h__84 = nf;
        h__85 = numel(fam_gaps);
        if h__85 < h__84, h__84 = h__85; end
        for g = (1):((h__84) - 1)
            if ((fam_gaps((g) + 1) < fam_gaps((f) + 1)) || ((fam_gaps((g) + 1) == fam_gaps((f) + 1)) && (fam_simplicity((g) + 1) < fam_simplicity((f) + 1))))
                f = g;
            end
        end
        if (fam_mass((f) + 1) == 1)
            h__86 = cap;
            h__87 = numel(o_feas);
            if h__87 < h__86, h__86 = h__87; end
            for i = (0):((h__86) - 1)
                if ((bitand(bitshift(fam_use((i) + 1), -(f)), 1) == 1) && (feasible((i) + 1) == 1))
                    o_feas((i) + 1) = 1;
                end
            end
            [t__203, t__204, t__205, t__206, t__207, t__208, t__209] = asils.models.looprules.loop_family_violation(mode_ix, feasible, fam_use, f, vstart, vval, vperf, cap, nm);
            vb = t__203;
            mode_ix = t__204;
            feasible = t__205;
            fam_use = t__206;
            vstart = t__207;
            vval = t__208;
            vperf = t__209;
            if ((fine && (k_gyro < 0.999)) && (~closed_g))
                g0 = k_gyro;
                h__88 = 1;
                h__89 = asils.models.looprules.loop_round3((g0 * 3));
                if h__89 < h__88, h__88 = h__89; end
                k_gyro = h__88;
                [t__210, t__211, t__212, t__213, t__214, t__215, t__216, t__217] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 16, (-(1)), f, g0, k_gyro, vb, 0);
                n = t__210;
                ek = t__211;
                eo = t__212;
                ep = t__213;
                ea = t__214;
                eb = t__215;
                ec = t__216;
                ef = t__217;
            elseif (((fine && (k_heads == 2)) && (~st2)) && (~closed_h))
                k_heads = 1;
                [t__218, t__219, t__220, t__221, t__222, t__223, t__224, t__225] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 17, (-(1)), f, 2, 1, vb, 0);
                n = t__218;
                ek = t__219;
                eo = t__220;
                ep = t__221;
                ea = t__222;
                eb = t__223;
                ec = t__224;
                ef = t__225;
            elseif (((((fam_fmr((f) + 1) == 1) && (fam_power((f) + 1) == 0)) && (k_lambda > lambda_min)) && (~lup)) && (~closed_l))
                lam = k_lambda;
                h__90 = lambda_min;
                h__91 = (lam / 3);
                if h__91 > h__90, h__90 = h__91; end
                k_lambda = h__90;
                ldown = true;
                [t__226, t__227, t__228, t__229, t__230, t__231, t__232, t__233] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 18, (-(1)), f, lam, k_lambda, vb, 0);
                n = t__226;
                ek = t__227;
                eo = t__228;
                ep = t__229;
                ea = t__230;
                eb = t__231;
                ec = t__232;
                ef = t__233;
            elseif (((fam_fmr((f) + 1) == 1) && (~closed_f)) && (sc((fix(1)) + 1) > scale_min))
                s0 = sc((fix(1)) + 1);
                h__92 = scale_min;
                h__93 = (s0 * down);
                if h__93 > h__92, h__92 = h__93; end
                sc((fix(1)) + 1) = h__92;
                tried_dn = bitor(tried_dn, bitshift(1, fix(1)));
                [t__234, t__235, t__236, t__237, t__238, t__239, t__240, t__241] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 19, (-(1)), f, s0, sc((fix(1)) + 1), vb, 0);
                n = t__234;
                ek = t__235;
                eo = t__236;
                ep = t__237;
                ea = t__238;
                eb = t__239;
                ec = t__240;
                ef = t__241;
            else
                [t__242, t__243, t__244, t__245, t__246, t__247, t__248, t__249] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 20, (-(1)), f, 0, 0, vb, 0);
                n = t__242;
                ek = t__243;
                eo = t__244;
                ep = t__245;
                ea = t__246;
                eb = t__247;
                ec = t__248;
                ef = t__249;
            end
        end
    end
    t__250 = ((0):((7) - 1)).';
    for p = t__250(logical((want((t__250) + 1) ~= 0))).'
        s = sc((p) + 1);
        if (bitand(bitshift(frz, -(p)), 1) == 1)
            [t__251, t__252, t__253, t__254, t__255, t__256, t__257, t__258] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 21, (-(1)), p, s, 0, 0, 0);
            n = t__251;
            ek = t__252;
            eo = t__253;
            ep = t__254;
            ea = t__255;
            eb = t__256;
            ec = t__257;
            ef = t__258;
        elseif ((want((p) + 1) == 1) && (s >= scale_max))
            [t__259, t__260, t__261, t__262, t__263, t__264, t__265, t__266] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 22, (-(1)), p, s, 0, 0, 0);
            n = t__259;
            ek = t__260;
            eo = t__261;
            ep = t__262;
            ea = t__263;
            eb = t__264;
            ec = t__265;
            ef = t__266;
        elseif ((want((p) + 1) == 1) && (bitand(bitshift(tried_dn, -(p)), 1) == 1))
            [t__267, t__268, t__269, t__270, t__271, t__272, t__273, t__274] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 23, (-(1)), p, s, 0, 0, 0);
            n = t__267;
            ek = t__268;
            eo = t__269;
            ep = t__270;
            ea = t__271;
            eb = t__272;
            ec = t__273;
            ef = t__274;
        elseif ((want((p) + 1) == 2) && (s <= scale_min))
            [t__275, t__276, t__277, t__278, t__279, t__280, t__281, t__282] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 24, (-(1)), p, s, 0, 0, 0);
            n = t__275;
            ek = t__276;
            eo = t__277;
            ep = t__278;
            ea = t__279;
            eb = t__280;
            ec = t__281;
            ef = t__282;
        elseif ((want((p) + 1) == 2) && (bitand(bitshift(tried_up, -(p)), 1) == 1))
            [t__283, t__284, t__285, t__286, t__287, t__288, t__289, t__290] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 25, (-(1)), p, s, 0, 0, 0);
            n = t__283;
            ek = t__284;
            eo = t__285;
            ep = t__286;
            ea = t__287;
            eb = t__288;
            ec = t__289;
            ef = t__290;
        elseif (want((p) + 1) == 1)
            h__94 = scale_max;
            h__95 = (s * up);
            if h__95 < h__94, h__94 = h__95; end
            sc((p) + 1) = h__94;
            [t__291, t__292, t__293, t__294, t__295, t__296, t__297, t__298] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 26, (-(1)), p, s, sc((p) + 1), 0, 1);
            n = t__291;
            ek = t__292;
            eo = t__293;
            ep = t__294;
            ea = t__295;
            eb = t__296;
            ec = t__297;
            ef = t__298;
        else
            h__96 = scale_min;
            h__97 = (s * down);
            if h__97 > h__96, h__96 = h__97; end
            sc((p) + 1) = h__96;
            [t__299, t__300, t__301, t__302, t__303, t__304, t__305, t__306] = asils.models.looprules.loop_event(ek, eo, ep, ea, eb, ec, ef, n, 26, (-(1)), p, s, sc((p) + 1), 0, 0);
            n = t__299;
            ek = t__300;
            eo = t__301;
            ep = t__302;
            ea = t__303;
            eb = t__304;
            ec = t__305;
            ef = t__306;
        end
    end
end
