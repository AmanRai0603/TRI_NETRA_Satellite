function s = matrix_cell(code)
%VALIDATION.MATRIX_CELL  Render one matrix cell. A file, not a script local.
    switch code
        case 1, s = 'y';
        case 0, s = '.';
        otherwise, s = 'X';
    end
end
