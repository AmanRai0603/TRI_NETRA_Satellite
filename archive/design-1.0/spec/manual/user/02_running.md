# Running
<!-- kind: how-to; depth: read -->

**In one line:** open your case, choose a product, a scenario and a campaign, and run; the result is saved as one file beside your case, and nothing you run changes the software.

## Run in the web app or the workbench

1. **Case → Open** your CSV. The case report shows what it says and what it can run. If the format is wrong it names the line and the key, and nothing runs.
2. **Run**, and choose:
   - the **product**: a catalogue product, or let the solver choose;
   - the **scenario**: one of the release's, such as `detumble_3u` or `nadir_hold_3u`; the case values it reads are shown beside it;
   - the **campaign**: nominal, Monte Carlo, edge, sweep or fault, and the number of runs.
3. Watch it live: the 3D attitude, the time plots, the mode timeline and the requirement gauges.
4. When it ends, the result is saved beside your case and opens. Read its first sentence: how many requirements pass, on which engine, at what credibility.

## Find a product with the solver

1. Open your case and choose **Solve**.
2. The solver tries every offered product against your case, tunes each in SILS, and shortlists those that meet every requirement you wrote, with their margins.
3. If none does, it says so, with the gap per requirement. That is an answer, not a failure.

## Design new candidates (design team)

1. Choose a design job of the release, such as `cubesat_3u_ais`, and run **Design**.
2. Every combination that passes is saved as a **candidate** in your store (`store/candidates/<id>.toml`), with a result document for the job.
3. A candidate is not in the catalogue. To put one there, send a node form with "Something else", attaching the candidate file and its design result ([Asking for changes](04_asking_for_changes.md)).

## Run from a terminal

<!-- since P4 -->
```
adcs case check mycase.csv
adcs case import mycase.csv
adcs sim run detumble_3u --case mycase --save
adcs sim campaign inertial_hold_mc500 --case mycase --save
adcs solve mycase
adcs design cubesat_3u_ais
adcs result list --case mycase
```

## Run in the MATLAB SILS tool

1. Unzip `adcs_sils_matlab_<version>.zip` and start MATLAB in that folder. `TWIN.md` in it lists every part of SILS and whether this version carries it: the MATLAB tool is built together with the platform, part by part, so it always holds exactly what the platform holds.
2. Run, for example:

<!-- since P3M -->
```matlab
startup_asils                                     % adds the tool to the path
c   = asils.case.read('cases/ais_img_3u.csv');    % your case, checked
rec = asils.run('inertial_hold_3u', c, 'product', 'SYN-P-3U-FMR');
asils.viz.run(rec)                                % 3D attitude, time plots, modes
asils.result.save(rec)                            % into store/, beside the case
res = asils.campaign.run('inertial_hold_mc500', c, 'runs', 50);
asils.viz.campaign(res)
asils.app                                         % the same, in one window
```

3. The `examples/` folder walks through six common tasks, from a detumble to comparing a MATLAB result with a platform result.

## If it goes wrong

A refusal always names its reason. It is a correct answer, never a crash.

| The software says | It means | Do |
|---|---|---|
| `NotStated: mass.imax` | your case leaves blank a key this run needs | fill it, or pick a scenario that does not need it |
| `NotFitted: star tracker` | the product has no such hardware, so the row has no value | choose a product that has it, or accept that the row does not apply |
| `<part> has no <field>; it is measured at calibration` | a catalogue part is not yet measured | use another product |
| `the case format is adcs-case/2; this release reads adcs-case/1` | the file is from a newer release | use the matching release |
| `NotMeasured: hils_bay1 helmholtz_cage.field_range_T` | the lab has not measured that capability yet | nothing to fix in your case; the verification team measures the lab |
| `rig fit: short, Cage field range needed` | the lab cannot give what your case needs for OILS or HILS | ask the verification team; the quote names the shortfall |
| every result says UNCONFIRMED | a node it reads has nobody's name on it yet | nothing is wrong; to raise trust, see [Reading a node](06_reading_a_node.md) |
