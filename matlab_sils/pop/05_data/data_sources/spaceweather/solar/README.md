# solar

Solar-flux drivers: F10.7 (OMNI2), native F30 (CLS/LISIRD) and the F30<->F10.7 scale conversions.

Full Input/Process/Output contracts: file headers and the repository-root `CODE_MAP.md`.

| File | Purpose |
|---|---|
| `f30_from_f107.m` | Approximates native F30 from F10.7 (offline fallback only). |
| `f30_to_f107scale.m` | Rescales native F30 onto the F10.7 scale required by the DTM2020 research model. |
| `get_f107.m` | Fetches daily F10.7 with its 81-day centered mean from NASA OMNI2. |
| `get_f30.m` | Fetches native F30 with a measured-data-first fallback chain. |
| `get_f30_cls.m` | Downloads the CLS/CNES Nobeyama 30 cm flux (the index DTM2020 research is built on). |
| `get_omni2.m` | Downloads one full year of NASA OMNI2 hourly data as a timetable. |
