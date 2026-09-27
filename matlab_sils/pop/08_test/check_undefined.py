#!/usr/bin/env python3
"""
check_undefined.py -- find the class of bug that crashed validate_OD at line 751.

WHAT IT LOOKS FOR
    An identifier that is USED but never ASSIGNED anywhere in its file, and is not a
    function, a package, or a builtin. That is `DO_PLOTS` when the toggle is `PLOTS`:
    a name I invented that nothing defines. MATLAB does not catch it until the line
    executes -- which, in validate_OD, was after a 20 s propagation and every
    download.

WHY STATICALLY AND NOT BY RUNNING
    Most of these paths cannot run here: no MATLAB, no network, no Aerospace Toolbox.
    A bug that only appears on the user's machine after 20 s of work is exactly the
    one to catch by reading rather than executing.

HOW IT AVOIDS LYING TO YOU
    Comments and string literals are stripped first -- otherwise every word inside an
    fprintf becomes a phantom "undefined variable" and the report becomes noise you
    learn to ignore, which is worse than no report.
    Function CALLS are excluded by collecting every .m basename in the tree plus a
    builtin list. Struct FIELDS (s.foo) are excluded: `foo` there is not a variable.
"""
import re, sys, os
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else '.')

BUILTIN = set("""
drawnow clf cla shg refresh uistack
abs acos acosd all any asin asind atan atan2 atan2d ceil cell cellfun class
containers cos cosd cross cumsum datenum datestr datetime deal deg2rad det diag
diff disp dot double eps error exist eye false fclose feval fieldnames figure find
fix fliplr flipud floor fopen fprintf fread func2str fwrite gca gcf get getfield
histc hold horzcat hypot inf inputname int2str integral interp1 isa iscell ischar
isempty isequal isfield isfinite isinf islogical isnan isnumeric isreal isrow
isscalar isstruct isvector length linspace load log log10 log2 logical lower
mat2str max mean min mod nan nargin nargout ndims nnz norm num2str numel ones
pi pinv plot polyfit polyval printf prod rad2deg rand randn real regexp regexprep
rem repmat reshape rmfield round save setfield sign sin sind single size sort
sprintf sqrt squeeze str2double str2num strcat strcmp strcmpi strfind strjoin
strrep strsplit strtrim struct subplot sum swapcase tan tand text tic toc trace
transpose true try typecast uint8 unique upper var vertcat warning while xlabel
ylabel zlabel zeros title legend grid axis xlim ylim zlim semilogy semilogx loglog
bar barh area scatter surf mesh contour colorbar colormap subsref numel end
mkdir exist fullfile fileparts filesep dir delete copyfile movefile
strvcat blanks deblank fliplr circshift kron trapz cumtrapz gradient del2
nchoosek factorial primes gcd lcm nthroot expm logm sqrtm rank null orth
lu qr chol svd eig schur hess balance cond normest
ode45 ode89 ode113 ode15s odeset deval
now clock cputime etime pause input keyboard
bitand bitor bitxor bitshift dec2bin bin2dec dec2hex hex2dec
regexpi validatestring assert inputParser
graphics_toolkit print saveas exportgraphics sgtitle rectangle
nan NaN Inf inf eps pi ans varargin varargout
websave webread weboptions urlread urlwrite
gunzip untar unzip gzip tar zip
containers.Map
matlab java javax org com
box clear grid hold axis figure clf shading rotate3d datetick
set vecnorm plot3 arrayfun datevec median minutes yline xline height isdatetime
num2cell cellstr char string fliplr rot90 tril triu eye accumarray bsxfun
cat colon end ndgrid meshgrid permute ipermute shiftdim sub2ind ind2sub
nanmean nanstd nansum std rms mode range iqr prctile quantile cov corrcoef
sortrows issorted ismember setdiff intersect union unique flip
datestr datenum weeknum calendar eomday is_leap_year addtodate
fix rem idivide floor ceil round nthargout print_usage inputname
matlabroot version ver license exist which type lookfor help doc
sphere cylinder ellipsoid patch surface line light material shading
alpha camlight lighting view rotate3d zoom pan datacursormode
annotation uicontrol uipanel uimenu uitoolbar dialog msgbox
tightPosition sgtitle subtitle nexttile tiledlayout
isvarname genvarname matlab.lang.makeValidName
func2str str2func nargchk narginchk nargoutchk validateattributes
onCleanup lastwarn lasterr rethrow MException
containers keys values isKey remove
table array2table struct2table table2struct readtable writetable
timetable retime synchronize
categorical categories iscategorical
fft ifft fft2 ifft2 fftshift conv conv2 filter deconv
interp2 interp3 interpn griddata scatteredInterpolant
fminsearch fminbnd fzero fsolve lsqnonlin quad quadgk
cellfun structfun containers
weboptions webwrite jsondecode jsonencode
regexptranslate strtok textscan fgetl fgets fscanf sscanf
fseek ftell frewind feof ferror
computer ispc isunix ismac getenv setenv system unix dos
rng randi randperm
warning_ids nargin nargout
mfilename clc clf close home tempname tempdir pwd cd ls rmdir isdir isfolder isfile
namelengthmax computer memory profile diary echo more format
bitcmp flintmax realmax realmin intmax intmin
deal ifelse merge tic toc rethrow lasterror
substr index rindex strjust ostrsplit strread
nthargout columns rows numfields isargout
puts fputs fdisp fflush stdout stderr stdin
usleep sleep time mktime localtime gmtime strftime strptime asctime ctime
octave_config_info exist_in_path file_in_loadpath
sizeof typecast cast idivide colon linspace logspace
end_try_catch print_empty_dimensions
gammaln beta betaln erf erfc erfinv erfcinv gamma factorial
sinh cosh tanh asinh acosh atanh sec csc cot secd cscd cotd
nthroot cbrt exp expm1 log1p
cummax cummin
speye sparse full spalloc nzmax spones spfun
issparse isbanded isdiag istriu istril ishermitian issymmetric
inputParser addRequired addOptional addParameter parse

""".split())

KEYWORDS = set("""function end if elseif else for while switch case otherwise break continue
return try catch global persistent do until unwind_protect unwind_protect_cleanup
endfunction endif endfor endwhile endswitch end_try_catch parfor spmd classdef
properties methods events enumeration arguments""".split())

def strip_noise(src: str) -> str:
    """Remove block comments, line comments, and string literals."""
    out_lines, in_block = [], False
    for line in src.split('\n'):
        st = line.strip()
        if st.startswith('%{'):
            in_block = True; out_lines.append(''); continue
        if st.startswith('%}'):
            in_block = False; out_lines.append(''); continue
        if in_block:
            out_lines.append(''); continue
        # strings before comments: a % inside '...' is not a comment
        res, i, n = [], 0, len(line)
        while i < n:
            c = line[i]
            if c == "'":
                # Transpose vs string. After an identifier or a closing bracket a
                # quote is TRANSPOSE (x'); otherwise it opens a STRING.
                # The trap: `case 'twobody'` -- "case" ends in a letter, so a naive
                # isalnum() test calls that quote a transpose, leaves the string
                # contents in the code, and every case label becomes a phantom
                # undefined variable. That is how a checker earns its way into being
                # ignored. Keywords are therefore excluded explicitly.
                # A quote is TRANSPOSE only if it touches what precedes it with no
                # space: A(1)' is transpose, but [base f(x) 'lit'] is a STRING --
                # inside brackets a space separates elements. Missing that left
                # '&FORMAT=TLE' unstripped and FORMAT/TLE became phantom variables.
                raw_prev = ''.join(res)
                touching = bool(raw_prev) and not raw_prev[-1].isspace()
                prev = raw_prev.rstrip()
                tok = re.search(r'([A-Za-z_]\w*)$', prev)
                is_kw = tok and tok.group(1) in KEYWORDS
                if touching and prev and not is_kw and (prev[-1].isalnum() or prev[-1] in ")]}._'"):
                    res.append(c); i += 1; continue
                j = i + 1
                while j < n:
                    if line[j] == "'":
                        if j + 1 < n and line[j+1] == "'":
                            j += 2; continue
                        break
                    j += 1
                res.append("''"); i = j + 1; continue
            if c == '"':
                j = line.find('"', i + 1)
                if j < 0: j = n
                res.append('""'); i = j + 1; continue
            if c == '%':
                break
            res.append(c); i += 1
        out_lines.append(''.join(res))
    return '\n'.join(out_lines)

# every function the tree defines (basename) + every package name
FUNCS, PKGS = set(), set()
for p in ROOT.rglob('*.m'):
    FUNCS.add(p.stem)
    for part in p.parts:
        if part.startswith('+'):
            PKGS.add(part[1:])

ID = r'[A-Za-z_]\w*'

# COMMAND SYNTAX: `box on` is box('on'), `axis off` is axis('off'). Both words look
# like identifiers and neither is a variable. This is the last big false-alarm class.
CMD_WORDS = set("""on off equal tight square auto manual normal reverse none all
image xy ij fill vis3d hold clc close force -force -r -depsc -dpng -v7.3 -mat
default remove global local
clear clc close all functions variables""".split())

def assigned_names(code: str, is_func: bool, header: str) -> set:
    a = set()
    # join line continuations first: `[t, ...\n y] = ode45(...)` is one statement,
    # and scanning it line-by-line loses the outputs after the '...'
    code = re.sub(r'\.\.\.\s*\n\s*', ' ', code)
    # x = ..., x(i) = ..., x.f = ..., x{i} = ...
    for m in re.finditer(rf'(?:^|;|,|\)|\s)({ID})\s*(?:\([^)]*\)|\{{[^}}]*\}}|\.\w+)*\s*=(?!=)', code, re.M):
        a.add(m.group(1))
    # [a, b] = f(...)
    for m in re.finditer(r'\[([^\]\[]*)\]\s*=(?!=)', code):
        for w in m.group(1).split(','):
            w = w.strip()
            mm = re.match(rf'^({ID})', w)
            if mm: a.add(mm.group(1))
    # for x = ...
    for m in re.finditer(rf'\bfor\s+({ID})\s*=', code):
        a.add(m.group(1))
    # catch ME / catch(ME): the exception variable IS assigned by the catch
    for m in re.finditer(rf'\bcatch\s*\(?\s*({ID})', code):
        a.add(m.group(1))
    # ANONYMOUS FUNCTION PARAMETERS: `add = @(n,A) struct('n',n,'A',A)` -- A is a
    # parameter, not an undefined variable. Missing this flagged buildBox.m and
    # spaceweather.m, which are both correct.
    for m in re.finditer(r'@\s*\(([^)]*)\)', code):
        for w in m.group(1).split(','):
            w = w.strip()
            if re.fullmatch(ID, w): a.add(w)
    # FUNCTION HANDLES: `solver = @ode78` refers to a function, not a variable.
    for m in re.finditer(rf'@\s*({ID})', code):
        a.add(m.group(1))
    # global / persistent
    for m in re.finditer(rf'\b(?:global|persistent)\s+([\w\s]+)', code):
        a.update(m.group(1).split())
    # function params + outputs
    if is_func:
        mh = re.match(rf'\s*function\s*(?:\[([^\]]*)\]|({ID}))?\s*=?\s*({ID})\s*\(([^)]*)\)', header)
        if mh:
            if mh.group(1): a.update(w.strip() for w in mh.group(1).split(',') if w.strip())
            if mh.group(2): a.add(mh.group(2).strip())
            if mh.group(4): a.update(w.strip() for w in mh.group(4).split(',') if w.strip())
    return a

def used_names(code: str) -> set:
    u = set()
    # drop command-syntax argument words before scanning
    # `grid on; box on;` puts two commands on one line, so anchor on ^ OR ';'
    # rather than the whole line -- the whole-line version missed every one of them.
    cmd = '|'.join(re.escape(w) for w in CMD_WORDS)
    code = re.sub(r'(^|;|,)\s*([A-Za-z_]\w*)\s+(' + cmd + r')\s*(?=;|,|$)',
                  r'\1 \2', code, flags=re.M)
    for m in re.finditer(rf'\b({ID})\b', code):
        name = m.group(1)
        s, e = m.start(), m.end()
        # struct field access: preceded by '.'
        before = code[:s].rstrip()
        if before.endswith('.') and not before.endswith('..'):
            continue
        # field in a struct() call or a name followed by '.' is fine to keep
        u.add(name)
    return u

problems = []
for p in sorted(ROOT.rglob('*.m')):
    src = p.read_text(errors='replace')
    code = strip_noise(src)
    if not code.strip():
        continue
    # split into function units; a script has no leading 'function'
    first_fn = re.search(r'^\s*function\b', code, re.M)
    is_script = not (first_fn and code[:first_fn.start()].strip() == '')

    units = []
    if is_script:
        cut = first_fn.start() if first_fn else len(code)
        units.append(('<script>', code[:cut], False, ''))
        rest = code[cut:]
    else:
        rest = code
    for m in re.finditer(r'^\s*function\b.*$', rest, re.M):
        start = m.start()
        nxt = re.search(r'^\s*function\b', rest[m.end():], re.M)
        stop = m.end() + (nxt.start() if nxt else len(rest) - m.end())
        name_m = re.search(rf'({ID})\s*\(', m.group(0))
        units.append((name_m.group(1) if name_m else '?', rest[start:stop], True, m.group(0)))

    local_fns = set()
    for m in re.finditer(rf'^\s*function\b[^\n]*?({ID})\s*\(', code, re.M):
        local_fns.add(m.group(1))
    for m in re.finditer(rf'^\s*function\s+(?:\[[^\]]*\]|{ID})\s*=\s*({ID})', code, re.M):
        local_fns.add(m.group(1))

    # NESTED FUNCTIONS capture the enclosing scope: `function u=U2(x)` written
    # INSIDE accelFromDeg2 can read mu/Re/dcs from its parent. Treating each unit
    # as isolated flagged all three, which are correct. Octave/MATLAB distinguish
    # nested (indented, inside the parent's body, parent not yet ended) from local
    # (top-level, after the parent's end) -- a cheap proxy is whether the file's
    # units are separated by a top-level `end`. Rather than guess, give every unit
    # the union of names assigned anywhere in the FILE as a fallback pool: this
    # trades a few missed detections for no false alarms, and a checker that cries
    # wolf gets switched off.
    file_assigned = assigned_names(code, False, '')
    # ...including every function header's parameters and outputs. A nested function
    # reads its parent's params (accelFromDeg2's U2 uses dcs/mu/Re), and skipping the
    # headers here flagged all three as undefined.
    for hm in re.finditer(r'^\s*function\b[^\n]*$', code, re.M):
        file_assigned |= assigned_names('', True, hm.group(0))

    for uname, ucode, is_fn, header in units:
        A = assigned_names(ucode, is_fn, header)
        A |= local_fns
        A |= file_assigned
        U = used_names(ucode)
        for name in sorted(U - A - KEYWORDS - BUILTIN - FUNCS - PKGS):
            # a name immediately followed by '(' that we do not know is likely a
            # function we failed to index -- report separately, lower confidence
            called = re.search(rf'\b{re.escape(name)}\s*\(', ucode) is not None
            problems.append((str(p.relative_to(ROOT)), uname, name, 'call?' if called else 'VAR'))

var_problems = [x for x in problems if x[3] == 'VAR']
call_problems = [x for x in problems if x[3] == 'call?']

# ============================================================================
#  STRUCT FIELDS: the R.rdo.rms3d class.
#
#  `R.rdo.rms3d` where rtn_stats returns `pos3D`. `R.acc.ratio` where od_metrics
#  builds R.acc WITHOUT a ratio field and computes it inline. Both crash at the
#  line, both survived review, and both are the DO_PLOTS bug wearing a dot.
#
#  This cannot be done perfectly without type inference -- a struct's shape depends
#  on which branch ran. So the test is deliberately narrow and therefore trustworthy:
#  for a variable whose fields are set ONLY via struct('a',..,'b',..) or x.f = ...
#  IN THE SAME FILE, reading a field that never appears in that set is a bug. Where
#  a struct crosses a file boundary (R comes from od_metrics into show_provenance),
#  the fields are collected from EVERY file, which is a superset and so only reports
#  a name that exists NOWHERE. That is exactly the invented-name case.
# ============================================================================
all_fields = {}      # varname -> set of fields assigned anywhere
field_reads = []     # (file, var, field)

for p_ in sorted(ROOT.rglob('*.m')):
    c_ = strip_noise(p_.read_text(errors='replace'))
    c_ = re.sub(r'\.\.\.\s*\n\s*', ' ', c_)
    # x.f = ...   and   x.a.f = ...
    for m in re.finditer(rf'\b({ID})((?:\.{ID})+)\s*(?:\([^)]*\))?\s*=(?!=)', c_):
        base, chain = m.group(1), m.group(2)
        for f in chain.strip('.').split('.'):
            all_fields.setdefault(base, set()).add(f)
        # nested: R.rdo = ... makes 'rdo' a field of R
    # x = struct('a',..,'b',..)
    for m in re.finditer(rf'\b({ID})\s*(?:\.({ID}))?\s*=\s*struct\s*\(([^;]*)\)', c_):
        base = m.group(1)
        holder = m.group(2) or base
        for fm in re.finditer(r"''", m.group(3)):
            pass
        # struct() args were stripped to '' -- recover names from the RAW source
    # setfield / dynamic fields are not tracked: too dynamic to judge

# struct('name',...) survives stripping only in the raw text, so re-scan raw
for p_ in sorted(ROOT.rglob('*.m')):
    raw = p_.read_text(errors='replace')
    raw = re.sub(r'^\s*%.*$', '', raw, flags=re.M)
    raw = re.sub(r'\.\.\.\s*\n\s*', ' ', raw)
    for m in re.finditer(rf'\b({ID})(?:\.({ID}))?\s*=\s*struct\s*\(', raw):
        base = m.group(1); sub = m.group(2)
        j = raw.index('(', m.end()-1); depth = 0; k = j
        while k < len(raw):
            if raw[k] == '(': depth += 1
            elif raw[k] == ')':
                depth -= 1
                if depth == 0: break
            k += 1
        args = raw[j+1:k]
        names = re.findall(r"'([A-Za-z_]\w*)'\s*,", args)
        holder = sub if sub else base
        if sub:
            all_fields.setdefault(base, set()).add(sub)
        all_fields.setdefault(holder, set()).update(names)

for p_ in sorted(ROOT.rglob('*.m')):
    c_ = strip_noise(p_.read_text(errors='replace'))
    c_ = re.sub(r'\.\.\.\s*\n\s*', ' ', c_)
    for m in re.finditer(rf'\b({ID})\.({ID})\.({ID})\b(?!\s*=(?!=))', c_):
        base, mid, leaf = m.groups()
        if base in all_fields and mid in all_fields.get(base, set()):
            known = all_fields.get(mid, set())
            if known and leaf not in known:
                field_reads.append((str(p_.relative_to(ROOT)), f'{base}.{mid}', leaf, sorted(known)))

print(f"scanned {len(list(ROOT.rglob('*.m')))} files\n")
print("=" * 78)
print("USED BUT NEVER ASSIGNED  (the DO_PLOTS class -- crashes at runtime)")
print("=" * 78)
if not var_problems:
    print("  (none)")
for f, u, n, _ in var_problems:
    print(f"  {f}  [{u}]  ->  {n}")
print()
print("=" * 78)
print("UNKNOWN NAME FOLLOWED BY '('  (a function that may not exist)")
print("=" * 78)
if not call_problems:
    print("  (none)")
for f, u, n, _ in call_problems:
    print(f"  {f}  [{u}]  ->  {n}(...)")
# ============================================================================
#  ONE NAME, TWO TYPES: the `P = R.rdo.dv_rtn` class.
#
#  show_OD did `P = defaults(PLOTS)` (a struct) and later `P = R.rdo.dv_rtn` (a
#  matrix). Eighteen lines on, `P.growth` died with "Dot indexing is not supported"
#  -- in a figure that had nothing to do with the edit.
#
#  My first attempt at this check flagged `fk = facets(k)`, which is a STRUCT ARRAY
#  index and perfectly correct -- and it would NOT have caught the real bug, because
#  `R.rdo.dv_rtn` has no paren after the identifier. Wrong on both ends.
#
#  The reliable signal needs no type inference: within one function, is the name used
#  BOTH as `x.field` AND as `x(:,1)` / `x(i,j)` -- struct access and numeric indexing?
#  Those cannot both be right for one variable. `facets(k)` is not flagged because
#  `facets` is never written `facets.field`.
# ============================================================================
clobbers = []
NUMIDX = None

for p_ in sorted(ROOT.rglob('*.m')):
    code = strip_noise(p_.read_text(errors='replace'))
    code = re.sub(r'\.\.\.\s*\n\s*', ' ', code)
    units = []
    fm = list(re.finditer(r'^\s*function\b.*$', code, re.M))
    if not fm:
        units = [('<script>', code)]
    else:
        if code[:fm[0].start()].strip():
            units.append(('<script>', code[:fm[0].start()]))
        for i, m in enumerate(fm):
            stop = fm[i+1].start() if i+1 < len(fm) else len(code)
            nm = re.search(rf'({ID})\s*\(', m.group(0))
            units.append((nm.group(1) if nm else '?', code[m.start():stop]))
    for uname, ucode in units:
        dotted = set(re.findall(rf'\b({ID})\.{ID}', ucode))
        for name in dotted:
            if name in PKGS or name in FUNCS or name in BUILTIN:
                continue
            # numeric indexing: x(:,1)  x(1,2)  x(k,:)  x(end,1) -- a COMMA or a colon
            # inside the parens. x(k) alone is struct-array indexing and is fine.
            numidx = re.search(rf'\b{re.escape(name)}\s*\(\s*(?::|end\b|\d)[^)]*[,:][^)]*\)', ucode)
            if numidx:
                assigns = re.findall(rf'^\s*{re.escape(name)}\s*=\s*(.+?);?\s*$', ucode, re.M)
                clobbers.append((str(p_.relative_to(ROOT)), uname, name,
                                 numidx.group(0)[:30], assigns[-1][:40] if assigns else '?'))

print()
print("=" * 78)
print("ONE NAME, TWO TYPES  (used as x.field AND indexed as x(:,1))")
print("=" * 78)
if not clobbers:
    print("  (none)")
seenc = set()
for f, u, n, idx, asg in clobbers:
    k = (f, u, n)
    if k in seenc: continue
    seenc.add(k)
    print(f"  {f}  [{u}]")
    print(f"     {n}.<field>  AND  {idx}   (last assigned: {n} = {asg})")

print()
print("=" * 78)
print("STRUCT FIELD READ BUT NEVER ASSIGNED  (the R.rdo.rms3d class)")
print("ADVISORY ONLY -- this collects fields by variable NAME across files, so an")
print("`sc` in one file is conflated with an `sc` in another. Doing it properly needs")
print("type inference. Treat a hit as 'go look', never as 'this is a bug'. The real")
print("defence is on the READER side: never assume a struct's shape (see")
print("show_provenance.m, which now checks isfield before every access).")
print("=" * 78)
if not field_reads:
    print("  (none)")
seen = set()
for f, var, leaf, known in field_reads:
    key = (var, leaf)
    if key in seen: continue
    seen.add(key)
    print(f"  {f}")
    print(f"     {var}.{leaf}  ->  never assigned. Known fields: {', '.join(known[:9])}")
print(f"\n{len(var_problems)} variable problem(s), {len(call_problems)} call problem(s), {len(seenc)} type-clash problem(s), {len(seen)} field problem(s)")
