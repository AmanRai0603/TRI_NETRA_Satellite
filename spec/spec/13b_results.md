
### 13.5 Result documents and the local store

#### 13.5.1 Why a result is a file

A run or a campaign is computed once. What it produced is saved once, as a **result document**, and after that it is opened, compared, shared and reported without running anything again. The same file opens in four places:

- any browser, offline;
- the web face and workbench (§13.1);
- the portal;
- the MATLAB SILS tool (§10.8).

So a team member who ran a case last week, or received a colleague's result, sees every plot and every verdict at once. The result is filed beside the case it ran (§13.5.4). The simulation, and its compute, is only needed for a new question.

`results/template.html` is the page, and `tools/results.py` in this package writes and reads it. `results/examples/` holds two demonstration results, a detumble run and a 20-run Monte Carlo on `ais_3u`. They were made by a deliberately simple Python B-dot model written only so the page has real trajectories to show. Every page they produce says so at the top, and their numbers are not evidence (§13.5.7).

#### 13.5.2 The format: `adcs-result/1`

One self-contained HTML file, `<case>_<scenario>_<date>[_<campaign>].result.html`. Like a form document (§5.10), it is a page drawn from one JSON block (`id="adcs-result"`), and every `<` inside the block is written as `\u003c`. The block holds:

| Member | What it records |
|---|---|
| `schema`, `kind`, `title`, `created` | `adcs-result/1`; `run` or `campaign` |
| `engine` | `platform` or `matlab`, with version and commit, and a `note` shown at the top of the page for anything that is not the platform. The demonstration results use `demo`. |
| `case` | id, hash, and **the case CSV exactly as it was run** |
| `product`, `tuned`, `scenario`, `campaign` | the product id, family, parts and whether it is synthetic; the tuned set's hash, never its values; the scenario id and hash; the campaign type, run count, seed and what it dispersed |
| `requirements` | every requirement the metrics are judged against: key, row, label, unit, sense, value, level, and whether the level was an assumed default |
| `metrics` | per metric: kind, unit, bound requirement, every run's value, the ensemble statistic with a note when the run count cannot support the stated level, the margin, and the verdict |
| `runs` | per run: index, dispersed values, metric values, and whether its channels were kept |
| `channels` | rate, names, units, optional chart `groups`, and per kept run the samples as base64 of gzip of little-endian float32, column-major |
| `restricted_excluded`, `notes` | what D1 kept out of this file, and anything the reader must know |
| `credibility` | the level the result can be trusted at, and why (the lowest of the eight scores over the rows it read; `none` for the demonstration engine) |
| `limits`, `unconfirmed` | where this result breaks: what the model leaves out, and the rows it read that nobody has confirmed |

**What is kept.** A single run keeps its channels. A campaign keeps every run's metrics and dispersed values. It keeps full channels for the nominal run, the best run, the worst run and any failing run, plus every run with `--keep all`. Any other run is re-created exactly from the campaign's seed (§9.7): the page says so, and `adcs sim rerun <result> <k>` does it.

**What is left out.** Restricted content never enters a result document, and a test proves it (§14.6). That covers D1 rows and channels, controller gains and tuned values (only the tuned set's hash is kept). A result document made for a client also leaves out layer-3 channels.

#### 13.5.3 What the page shows

The page is the run view and campaign dashboard of §13.2, drawn from the file, written to the explanation standard (§5.12):

- **Answer first:** one sentence under a breadcrumb (store › case › scenario): how many requirements pass, for which case and product, in how many runs, on which engine, whether it is a demonstration, and its credibility.
- **How to read this result** (overview): the four parts, from the verdict to the data. **Using this file** (how-to, hidden in Expert).

- **Summary:** each requirement's verdict, with a status icon and a label, never colour alone. Also the case, product, scenario and run count.
- **Requirements and metrics:** each metric against its requirement at the case's level: achieved value, statistic, margin, verdict. The table is downloadable as CSV.
- **Time histories:** the kept runs' channels, one chart per group, with a crosshair and tooltip. Requirement and threshold lines are drawn dashed. Each run's channels are downloadable as CSV.
- **Attitude:** a 3D view of the body, its axes and the field direction, with a time slider and play.
- **Campaign:** for each metric, a histogram with the requirement and the ensemble statistic marked, and the table of every run. Clicking a kept run shows it above.
- **Compare:** "Open a result to compare…" loads a second result document into the same page. Its metrics sit beside these, and its first channel is overlaid, dashed. The case, product or engine may differ: a MATLAB result against a platform result is how a developer checks the twin (§10.8.6).
- **Where this came from:** engine, case and its hash, product, tuned-set hash, scenario and campaign. It also shows the case CSV itself, downloadable, so anyone can run exactly that case again.
- **Where this result breaks:** the `limits`, the unconfirmed rows read, and every level the case assumed; then the common wrong idea that a pass means the satellite will meet the requirement in orbit, and why not.
- **What a margin is** (a station under the metrics table) and, in Learn, a prediction to make before reading the verdicts and a teach-back at the end.

Every section shows its kind, and the depth switch shows the exercises (Learn), the answer and explanation (Read), or the tables alone (Expert).

The charts follow one set of rules:
- one y-axis per chart;
- at most four series, with the categorical colours in fixed order and a legend;
- dashed ink for requirement lines;
- light and dark palettes selected, not inverted;
- no external script, so the page works offline and prints.

#### 13.5.4 The local store: every result beside its case

Every place that runs SILS keeps a **store**: a folder in which each result document is filed beside the exact case it was run on.

```
<store>/
  cases/<case id>/<sha12>.csv                              the case, as run (the CSV the result carries)
  results/<case id>/<sha12>/<scenario>_<date>[_<campaign>].result.html
  candidates/<id>.toml                                     products the designer saved (§8.6)
  index.csv                                                one row per result; index.sqlite in the workbench
```

A new store starts with the release's reference cases in `cases/`, so the default case `ais_3u` is there on first run. `<sha12>` is the first twelve hex digits of the SHA-256 of the case CSV. Two versions of a case therefore never share a folder, and a result can never be shown beside a case it was not run on.

| Where | The store |
|---|---|
| CLI and workbench | `~/.adcs/store/`; `--store <dir>` for another |
| MATLAB SILS tool | the zip's `store/` |
| Portal | each project's artefact directory, indexed in its PostgreSQL `case` and `run` tables (§15.4) |

**The files are the record, and the index is a cache.** `index.csv` has one row per result: the file, its case file, kind, date, engine, case and case hash, product, scenario, campaign type, runs, overall verdict, worst margin and the file's SHA-256. The workbench keeps the same index in SQLite, `index.sqlite`, for fast search. Both are rebuilt from the files by `adcs result index` or `asils.result.index`, and deleting either loses nothing. There is no database a result lives only in, so a store, or any folder of it, can be copied, zipped or mailed as it is, and opened by any of the three.

**Saving is automatic.** Every run and campaign a user starts in the workbench, the web app or the portal is saved when it finishes; the CLI saves with `--save`, and the MATLAB tool with `asils.result.save`. A run that was refused writes no result; its refusal is in the run's log.

**Opening a result runs nothing.** "Open a result" in the web app and the workbench, `adcs result import <file>...`, `asils.result.import(file)` and the portal's result upload all do the same thing: read the result document, write the case it carries into `cases/` if that exact case is not there yet, file the result under it, and update the index. Then the result is shown — its plots, verdicts and 3D view — with no engine call. That is how a result made last week, or on another computer, or by a colleague, or by the MATLAB tool, or by the rig, is seen again. `tools/results.py import` is the stand-in in this package, and it runs today.

Commands:
- `adcs sim run … --save` and `adcs sim campaign … --save` write the result document into the store;
- `adcs result import <file>...` files results made elsewhere, each beside its case;
- `adcs result list [--case c] [--verdict fail]` searches the index;
- `adcs result open <file>` opens the page;
- `adcs result channels <file> <run> <out.csv>` exports one run's channels;
- `adcs result index [<store>]` rebuilds the index;
- `adcs sim rerun <file> <k>` re-creates a run whose channels were not kept — the one command that runs the engine, and only when asked.

#### 13.5.5 Who writes result documents

| Writer | When |
|---|---|
| `adcs-sim` (the platform) | every run and campaign with `--save`, and every portal SILS run (the portal saves by default) |
| the MATLAB SILS tool | `asils.result.save(rec)`, with the same template, shipped in the zip as `viewer/result_template.html`, into the same store layout |
| `adcs-rig` | every OILS and HILS campaign, with the rung in `engine` and the witness stream's recording |
| `tools/results.py demo` | only the package's two demonstration files |

**A result carries a pointer back to its case, never the other way round.** A case is input and never changes; results accumulate beside it. Deleting a result never touches its case, and a case with no results left stays in the store until someone deletes it.

A result document is output, never input. No engine reads a result document's numbers as evidence, and a campaign's evidence rows come only from the run ledger (§5.5). The page is a view of those numbers.

#### 13.5.6 Checks

The `forms` CI job (§18), in `tools/form_browser_check.py`, also opens the demonstration results in headless Chromium. It checks:
- every chart draws with no script error;
- `tools/results.py import` files both demonstration results under `ais_3u`'s one case folder;
- the channels decode;
- opening a second result adds the comparison;
- `tools/explain_check.py` finds every mark of the standard on both.

In the built repository the `results` job goes further:
- the platform writes a result for `detumble_3u` and the MATLAB twin writes one for the same case;
- each opens in the other's viewer;
- the channels decoded by the page equal the recorder's output;
- `adcs result index` rebuilds the same index twice;
- `adcs result import` of a result into an empty store writes exactly its case and the result, beside each other, and nothing runs;
- the restricted-content test runs over a result document.

#### 13.5.7 The demonstration results are not results

`tools/results.py demo` exists only so the viewer can be seen and tested before the engine exists. Its model is deliberately simple:
- a rigid body with `ais_3u`'s inertia;
- a circular orbit;
- a tilted dipole field fixed in inertial space, with no disturbances;
- ideal sensors, and B-dot on three ideal 0.45 A·m² coils.

Its engine is named `demo`, and every page it writes says so above everything else. It is never a parity reference, a fixture or evidence, and the builder never quotes its numbers as the platform's.
