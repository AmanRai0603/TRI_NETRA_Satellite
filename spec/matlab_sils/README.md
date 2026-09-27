# ADCS SILS for MATLAB

The released SILS engine as plain MATLAB. It needs base MATLAB only. The
Parallel Computing Toolbox is used when present, and never required.

**If you use the software** (a team member, a trainee, a student): your only
input is a case CSV, and every finished run is saved for you beside it. Start
with `manual/00_start_here.md`. Nothing in this folder needs editing except
your case.

**If you want something changed** — a node's content, a new node, a behaviour
you think is wrong, a new algorithm — fill that node's form from
`forms/index.html`, save a filled copy, and send it to the developer team. They
reply in the same file, and the next release carries the change.

| Folder | What it is |
|---|---|
| `manual/` | the user manual: your case, running, results, asking for changes, every case key |
| `cases/` | `case_template.csv` (every key explained in its note) and the reference cases, including `ais_3u` (AIS, 10°) and `ais_img_3u` (AIS + imaging, 0.01°). Put your own cases here. |
| `forms/case_new_case.editor.html` | the case editor: open it in a browser, start from a reference case or your own CSV, see what it can run, download the CSV |
| `forms/index.html` | the node library: every node's document and its request form |
| `forms/new_node.form.html` | a request for a node that does not exist yet |
| `store/` | your local store: `cases/`, `results/` and `index.csv`. Every result is filed beside the exact case it ran on. |
| `viewer/result_template.html` | the page every result is saved into |
| `+asils/`, `examples/`, `tests/` | the engine, six worked examples, its tests |
| `data/` | everything else the engine reads, exported from the repository. Never edit it. |

```matlab
startup_asils
c   = asils.case.read('cases/ais_img_3u.csv');            % your input: always a case CSV, checked
rec = asils.run('inertial_hold_3u', c, 'product', 'SYN-P-3U-FMR');
asils.viz.run(rec)                                        % 3D attitude, time plots, modes
f   = asils.result.save(rec);                             % store/results/ais_img_3u/<sha12>/...result.html
r   = asils.result.open(f);                               % later: everything again, without re-running
asils.result.import('from_a_colleague.result.html')       % filed beside its own case; nothing runs
asils.result.index()                                      % store/index.csv, rebuilt from the files
```

Every result file also opens in any browser, and in the web app. Results from
the web app open here. `asils.version()` prints the release, catalogue and case
format this download was built from; quote it in any feedback.
