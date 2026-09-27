function write_csv(path, headers, M)
%VALIDATION.WRITE_CSV  Numeric matrix + header row to CSV. A FILE, not a script local
%   (MATLAB hoists a script's locals; Octave does not -- see CODEBASE_AUDIT.md).
%   Plain fprintf rather than writematrix/csvwrite: those differ between MATLAB and
%   Octave in header handling, and this has to run in both.
    fid = fopen(path,'w');
    if fid < 0, error('validation:write_csv','cannot write %s', path); end
    fprintf(fid, '%s', headers{1});
    for i=2:numel(headers), fprintf(fid, ',%s', headers{i}); end
    fprintf(fid, '\n');
    for r = 1:size(M,1)
        fprintf(fid, '%.10g', M(r,1));
        for c = 2:size(M,2), fprintf(fid, ',%.10g', M(r,c)); end
        fprintf(fid, '\n');
    end
    fclose(fid);
end
