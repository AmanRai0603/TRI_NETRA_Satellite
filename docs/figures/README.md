# Figures

- [`flow_design_to_hils.svg`](flow_design_to_hils.svg): TRI-NETRA ADCS from customer case to flight. The 13-step V&V flow runs through design (with the resize loop), verification (dispatch, Monte Carlo, soft OILS, report) and the hardware rungs (OILS, HILS).
- [`architecture_languages.svg`](architecture_languages.svg): who runs what, as a stack of layers. It goes from the pseudocode contract through the C99 and Rust flight software, the HAL, the targets on adcs-link/1, the Rust engine, the Python tools and the MATLAB/Octave twin, down to the outputs.
