function L = leapTable()
%LEAPTABLE  Leap-second table: [year month TAI-UTC(seconds)] at 0h UTC of date.
%   *** UPDATE THIS when IERS announces a new leap second (Bulletin C). ***
%   Current value TAI-UTC = 37 s since 2017-01-01 (none since; none scheduled).
    L = [1972 1 10; 1972 7 11; 1973 1 12; 1974 1 13; 1975 1 14; 1976 1 15;
         1977 1 16; 1978 1 17; 1979 1 18; 1980 1 19; 1981 7 20; 1982 7 21;
         1983 7 22; 1985 7 23; 1988 1 24; 1990 1 25; 1991 1 26; 1992 7 27;
         1993 7 28; 1994 7 29; 1996 1 30; 1997 7 31; 1999 1 32; 2006 1 33;
         2009 1 34; 2012 7 35; 2015 7 36; 2017 1 37];
end
