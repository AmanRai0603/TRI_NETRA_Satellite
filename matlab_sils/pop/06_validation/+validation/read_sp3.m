function E = read_sp3(fname, satId)
%VALIDATION.READ_SP3  Parse an SP3-c/d precise-orbit file (GPS-derived truth).
%   E = validation.read_sp3(fname[, satId])
%   SP3 is the standard precise-orbit product from GNSS-based POD: ECEF position
%   (km) at fixed epochs, optionally velocity (dm/s). For a LEO satellite the file
%   usually holds ONE object; pass satId to pick one if several are present.
%
%   RETURNS E:
%     E.utc  (Nx6) epoch [Y Mo D H Mi S]     E.mjd (Nx1)
%     E.r_ecef (Nx3) m                        E.v_ecef (Nx3) m/s (NaN if absent)
%     E.satId  the object id used             E.hasVel (bool)
%
%   Units: SP3 position is km -> x1000 m; velocity is dm/s -> x0.1 m/s.
    if nargin<2, satId=''; end
    fid=fopen(fname,'r'); if fid<0, error('validation:read_sp3','cannot open %s',fname); end
    % PREALLOCATE. A 1 Hz Swarm GPSxNAV day is 46807 epochs (~94k records); growing
    % with R(end+1,:) is O(n^2) and takes minutes. Grow geometrically instead.
    CAP = 4096;
    utc=zeros(CAP,6); R=zeros(CAP,3); V=zeros(CAP,3); ids=cell(CAP,1); nRec=0;   % NOTE: 'n' is already used by the epoch parser below
    curEpoch=[]; want='';
    hasV=false; hdrFlag=''; nVrec=0; nPrec=0; firstLine=true;
    while true
        ln=fgetl(fid); if ~ischar(ln), break; end
        if firstLine                                  % '#dV...' = pos+vel, '#dP...' = pos only
            firstLine=false;
            if numel(ln)>=3 && ln(1)=='#', hdrFlag=upper(ln(3)); end
        end
        if isempty(ln), continue; end
        c=ln(1);
        if c=='*'                                   % epoch line
            n=sscanf(ln(2:end),'%f',6).';
            if numel(n)>=6, curEpoch=n(1:6); else, curEpoch=[]; end   % only accept a full epoch
        elseif c=='P'
            if isempty(curEpoch), continue; end        % P before any valid epoch -> skip (robust)
            id=strtrim(ln(2:4));
            if isempty(want), want=pick(id,satId); end
            if strcmp(id,want)
                v=sscanf(ln(5:end),'%f',4).';
                if numel(v)<3, continue; end
                nRec = nRec + 1;
                if nRec > size(R,1)                          % geometric growth
                    m2 = 2*size(R,1);
                    utc(m2,6) = 0;
                    R(m2,3)   = 0;
                    V(m2,3)   = 0;
                    ids{m2}   = '';
                end
                utc(nRec,:)=curEpoch;
                R(nRec,:)=v(1:3)*1000;                    % km -> m
                V(nRec,:)=[NaN NaN NaN];
                ids{nRec}=id;
                nPrec=nPrec+1;
            end
        elseif c=='V'
            if nRec==0, continue; end                    % V with no preceding P -> skip
            id=strtrim(ln(2:4));
            if strcmp(id,want)
                v=sscanf(ln(5:end),'%f',4).';
                if numel(v)<3, continue; end             % malformed V -> leave NaN (FD fills it)
                V(nRec,:)=v(1:3)*0.1;                    % dm/s -> m/s
                hasV=true; nVrec=nVrec+1;
            end
        end
    end
    fclose(fid);
    if isempty(utc)
        error('validation:read_sp3:empty', ...
          ['no epochs parsed from %s. The file may not be plain-text SP3 (still ' ...
           'compressed?) or uses an unexpected layout. Check it opens as text.'], fname);
    end
    utc=utc(1:nRec,:); R=R(1:nRec,:); V=V(1:nRec,:); ids=ids(1:nRec);  % trim preallocation
    % Report exactly what the FILE provides -- the seed must come from the file's
    % own r,v whenever they exist (GPS products normally carry both).
    hasV = hasV && ~any(isnan(V(:)));                    % partial V is NOT usable as-is
    if strcmp(hdrFlag,'V') && nVrec==0
        warning('validation:read_sp3:noVrecords', ...
          ['%s header declares velocity (#dV) but no V records were parsed for %s. ' ...
           'Velocity will be derived from positions.'], fname, want);
    end
    if nVrec>0 && nVrec<nPrec
        warning('validation:read_sp3:partialV', ...
          ['%s has V records for only %d of %d epochs -> velocity will be derived ' ...
           'from positions for consistency.'], fname, nVrec, nPrec);
    end
    E.utc=utc; E.r_ecef=R; E.v_ecef=V; E.satId=want; E.hasVel=hasV;
    E.hdrFlag=hdrFlag; E.nPrec=nPrec; E.nVrec=nVrec;
    E.mjd = arrayfun(@(k) datenum(utc(k,1),utc(k,2),utc(k,3),utc(k,4),utc(k,5),utc(k,6))-678942, (1:size(utc,1))');
end
function w=pick(id,req)
    if nargin>=2 && ~isempty(req), w=req; else, w=id; end
end
