
---

## 13. Visualisation

### 13.1 One set of views for every rung

A SILS run, an OILS run and a HILS run are shown by the same views, reading the same `adcs-rec/1` channels. The only difference on screen is a rung badge (SILS, PIL, OILS, HILS, LAB TWIN) and, for a live rig run, a latency and deadline-miss strip. A view that works for SILS therefore works for HILS on the day the rig first runs, and a client who learned the SILS view reads the witness view without being taught.

### 13.2 The views

| View | Shows | Who |
|---|---|---|
| Case | the case editor (§8.3.3) and, once a CSV is opened, the case report: each value, where it went (the tree row, the scenario fields that read it), the assumptions taken, every unstated key with what it blocks, and which scenarios the case can run | client, team member |
| Solution | per product tried: margins after tuning against each requirement, the governing credibility, the confirming campaign; or "no catalogue product meets this case" with the gap per requirement and the design-request button | client (offered products only), engineer |
| Catalogue (internal) | products by class and status, candidates included, with their envelope and worst margin; a design job's sweep, combinations pruned, tuned and passed; the promotion checklist of §8.6 | designer, catalogue owner |
| Run | choose the case, the product (or let the solver choose), a scenario of the release and a campaign type, and a run count within quota; the case's values the scenario reads are shown beside it, read-only. Nothing here edits a scenario (§10.1). | client, team member |
| Run, live and replay | **3D attitude**: the body with its faces, each actuator's axis, the magnetometer's field vector, the Sun vector, nadir, the target, and the star tracker's field of view and exclusion cones. **Time plots**: APE and AKE against the requirement line, body rate, ring momenta and speeds, wheel speeds, commanded dipole, measured field, power. **Mode timeline** with fault markers. **Actuator panel**: each actuator's use against its limit. **Requirement gauges**: each bound metric against its requirement, with the margin. | client (SILS), witness (OILS, HILS), engineer |
| Campaign dashboard | per bound metric: a histogram and CDF across runs with the requirement line and the ensemble percentile marked; scatter of the metric against each dispersed parameter, so the thing that drives a failure is visible; a pass and fail table; one click opens the worst run in the run view | client (SILS), engineer |
| Parity | one metric or channel overlaid across SILS, lab twin, OILS and HILS for the same scenario, with the parity-ledger line and its stated cause | engineer; client after the order |
| Facility (internal) | commanded against measured cage field, bearing tilt and balance, stimulator state, IEU link health, latency histogram | test operator |
| Witness (client, after the order) | the run view, read-only and live, streamed from the rig for the client's own unit, with the campaign's schedule and a comment box routed to sales | client |
| Evidence | the verification matrix, parity ledger, reports and certificate, per order | client (their order), quality |
| Results | the local store (§13.5.4): every saved result, by case, with its verdict and worst margin, searchable; "Open a result", which files a result made anywhere beside its case and shows it without running anything; compare any two | everyone |
| Node | a node's document, read-only, as the node library shows it, and "Ask for a change", which downloads its node form (§5.10) | team member |
| Risk management | the platform's conclusion (highest open level, net risks closed this quarter, share of beliefs tested), the seven areas on one 0–5 scale, the open L4 and L5 risks with their closing tests, and each quarter's narrative (§5.13); read-only | everyone internal; quality |

Every view follows the explanation standard (§5.12): a breadcrumb, an answer-first sentence, an overview before detail, each panel's kind, the Learn · Read · Expert switch, claim tags on every number that is not the user's own, and a "where this breaks" panel.

### 13.3 How it is built

It is the web face: one `index.html`, one `app.css`, ES modules served individually, no bundler, `app.js` owning every listener, and views emitting `data-` attributes. The new modules are:

- `sim.js`: the run view;
- `campaign.js`: the dashboard;
- `parity.js`;
- `attitude3d.js`: the 3D view;
- `stream.js`: live data;
- `scenario.js`: the scenario picker (read-only);
- `witness.js`;
- `evidence.js`;
- `case.js`: the case view, with the case editor served inside it;
- `results.js`: the results store view;
- `solve.js`: the solution view;
- `catalogue.js`: the internal catalogue view.

The node page (read-only) gains "Ask for a change", which serves the node's form (§5.10). No page of the web face writes anything to the repository or to a sheet (§3.4); the only things a user puts into the software are a case CSV, a result document to view, and, in the portal, a client's FMU.

- **3D:** `three.js`, vendored as one ES module file under `web/vendor/`, MIT licence, recorded in `ADOPTION.lock` with its version and fallback. The fallback is a 2D projection of the same scene.
- **Charts:** The `chart.js` canvas routine and its measured light and dark `SCHEMES`, extended with a streaming time series that keeps a fixed window.
- **Live data:** server-sent events, `GET /v1/stream/<run>`: one-way, simple, and they pass through proxies. At most 20 Hz of downsampled channels; the recorder keeps full rate.
- **Replay:** the same view reads the recorder's files; the time slider scrubs, and the 3D and the plots move together.
- **Colour and tokens:** The `app.css` tokens and dark override. The rung badge colours are new tokens, validated like the rest.
- **Units:** the face converts for display only. Everything it receives is SI (`areas/faces.md`).

### 13.4 Panels, checked in a real browser

Every new view is a declared panel in `panels/<id>.toml` and passes `panel_check.py`'s checks:

- it renders;
- it moves when each declared input moves;
- it reads, changing when the engine's answer is intercepted;
- it matches its reference image in light and dark, or declines the pixel check with a reason.

The new panels are `case_report`, `solution_margins`, `catalogue_classes`, `run_3d`, `run_plots`, `run_modes`, `campaign_histogram`, `campaign_scatter`, `parity_overlay`, `facility_cage`, `witness` and `evidence_matrix`. Their `confirmed_by` stays `UNCONFIRMED` until a person has looked at the reference and agreed with it. Their reference images are recorded from the mock rig and a synthetic campaign, so they exist on day one and are replaced when real data exists. A panel reference is a picture of what the code drew, taken to catch the picture changing. It is never evidence about the ADCS, and no evidence package includes one.
