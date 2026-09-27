
---

## 4. Repository layout

Suggested repository name: `ADCS_PLATFORM`. The name appears in `Cargo.toml [workspace.package] repository`, `.devcontainer` and the README, and changes nowhere else.

```
ADCS_PLATFORM/
  AGENTS.md  CONTRIBUTING.md  README.md  CODEOWNERS(generated)  ADOPTION.lock
  Cargo.toml  Cargo.lock  rust-toolchain.toml  rustfmt.toml  .gitignore
  .cargo/config.toml               aliases: xtask, adcs
  .claude/hooks/  .claude/settings.json     the scope hook the implementation agent runs under (§17.4)
  .github/workflows/{gate,nightly,release}.yml  dependabot.yml  pull_request_template.md
  .github/ISSUE_TEMPLATE/request.yml        "attach a node form" — where a request can arrive
  .devcontainer/{devcontainer.json,setup.sh,start.sh}
  areas/{generators,faces,document,data,numerics,release,simulation,rig,portal,evidence,solver,intake}.md

  plan/                            this package's plan/: tree.json, seed_content.toml, kpis.toml, units.toml,
                                   physics.toml, case_inputs.toml, case_template.csv, cases/*.csv, expected_node_ids.json,
                                   twin_map.toml (every SILS element's platform and MATLAB sides, §10.8.7)
  layers/                          generated once by the seeder; the tree's shape, changed only by a reviewed
                                   developer change (a new group is "something else", §5.10)
  cases/                           the reference cases, imported: `adcs case import plan/cases/*.csv`
  sources/sources.toml             generated once from plan/seed_content.toml [[source]]; grows through intake
  bundles/{igrf14,atmos-density,catalogue}/<version>/{manifest.toml,...}
  catalogue/{schema.toml,families.toml,classes.toml}     the source the catalogue bundle is published from
  catalogue/{parts,products,algorithms}/*.toml  catalogue/classes/*.csv
  designs/*.toml                   design jobs for `adcs design` (§8.6)
  devices/<part>.toml              each part's port protocol: registers, framing, faults (§9.5)
  scenarios/*.toml                 the one template (§10)
  campaigns/*.toml                 campaign definitions
  rig/{device_maps,labs}/*.toml    the rig's wiring, and the lab as a plant and as measured facility capabilities (§12.10)
  fsw/                             reference flight software, C99 (§9.10)
    include/{adcs_hal.h,adcs_fsw.h}
    src/  tm/  tc/  tests/  CMakeLists.txt
  matlab_sils/                     the MATLAB SILS twin (§10.8): +asils/, examples/, tests/; written in lockstep with
                                   crates/ and fsw/, element by element (§10.8.7)
  forms/                           node_form.html, case_editor.html, library.html (§5.10, §8.3.3); examples/
  results/                         template.html, the result document (§13.5); examples/
  manual/user/  manual/developer/  the two manuals (§16.4), shipped and checked
  derisk/                          the de-risking ledger (§5.13): risks.toml, beliefs/<id>.toml, narratives/<quarter>.md
                                   (a person's prose), narrative_template.xlsx, rollup.toml (written at release)
  intake/                          AGENT.md: the implementation agent's standing instructions (§17.4);
                                   requests/<request id>/ — never committed: the working area of `xtask intake`
  crates/adcs-mod-*/nodes/<node>/requests/<request id>.request.html   every request that changed the node
  crates/adcs-mod-*/nodes/<node>/versions.toml, versions/<n>/          every version of the node, and why (§5.13)
  deploy/compose.yaml              the portal stack (§15.9)
  docs/phases/P<n>.md              one report per build phase (§19)
  docs/  panels/  tools/  web/  matlab/  xtask/
  dist/                            built downloads, never committed: adcs_sils_matlab_<version>.zip, the node library

  crates/
    adcs-units       RING 0  quantities, units, frames, pmath          (ported; §6.1 adds)
    adcs-core        RING 1  resolver, graph, credibility, physics     (ported; §6.2 rewrites physics)
    adcs-sheet               the sheet: loader, generators, gate       (ported; F1, F2, F7; write paths removed, F17)
    adcs-bus         RING 2  wire types + the loop contract (§11)      (ported; §11 adds `loop`)
    adcs-data                bundles, store, lockfile                  (ported; F8)
    adcs-mod-*       RING 3  node crates, one per owner layer (§5.4)   (seeded; content by intake)
    adcs-modules             the facade: generated tables              (ported)
    adcs-intake              the node form: read, check, write the sheet, verify, reply (§5.11)  NEW
    adcs-derisk              the ledger: check, record, rollup, narrative (§5.13)               NEW
    adcs-catalogue           reads parts, families, products, algorithms and classes, from the working copy or a bundle (§7)  NEW
    adcs-case                the case format: CSV check, import and export, the report, the case store (§8.3)  NEW
    adcs-config              screening: supply map, closures over candidates (§8.4)                NEW
    adcs-tune                SILS tuning of a candidate's algorithms (§8.5)                        NEW
    adcs-solve               client mode and designer mode; saves candidates to the store (§8.6)   NEW
    adcs-sim-core            the loop engine's pure core, no_std (§9)  NEW
    adcs-sim                 scheduler, device emulators, campaigns, recorder, metrics (§9, §10)  NEW
    adcs-result              result documents and the local store: write, read, import, index (§13.5)  NEW
    adcs-fsw-abi             implements adcs_hal.h for SILS; links the flight C code (§9.6)       NEW
    adcs-rig                 real-time runner, transports, environment-simulator drivers (§12)   NEW
    adcs-evidence            verification matrix, parity ledger, reports, certificate (§14)       NEW
    adcs-cli  adcs-daemon  adcs-ffi  adcs-wasm  adcs-py              FACES (ported; read and run only; §16 adds)
    adcs-portal              multi-tenant HTTP service (§15)           NEW
    adcs-worker              run-queue executor (§15.4)                NEW
```

The ring rule extends to the new crates:

```
adcs-units -> adcs-core -> adcs-bus -> adcs-mod-* -> adcs-modules -> faces
                  |           |
                  +-> adcs-sim-core (no_std: reads adcs-core::physics, adcs-units)
                              |
                  adcs-catalogue -> adcs-units;  adcs-case -> adcs-bus, adcs-modules (node ids and sheet values)
                  adcs-intake -> adcs-sheet, adcs-units, adcs-modules (read), adcs-derisk; used by xtask only, never by a face
                  adcs-derisk -> adcs-units; used by xtask; faces read its rollup.toml and narrative only
                  adcs-config -> adcs-case, adcs-catalogue, adcs-modules
                  adcs-sim (std) -> adcs-fsw-abi, adcs-case, adcs-catalogue;  adcs-tune -> adcs-sim
                  adcs-result -> adcs-case, adcs-sim (types only)
                  adcs-solve -> adcs-config, adcs-tune, adcs-sim;  adcs-evidence -> adcs-sim
                  adcs-rig (std) -> adcs-sim, adcs-bus::loop
                  adcs-portal, adcs-worker -> faces' library APIs only
```

- `adcs-sim-core` may not depend on anything above `adcs-core`, so the plant can never read the tree's generated tables. Parameters reach it as a resolved case and as descriptors.
- `adcs-intake` is a developer tool. No face, no worker and no portal route links it, so the released software has no path by which a request changes it.
- `adcs-portal` never links the engine directly. It enqueues work, and `adcs-worker` runs it, so a portal crash cannot corrupt a run and a run cannot hang the portal.
- `xtask graph`'s ring check is extended to the new crates, and a manifest line that breaks the direction fails the gate.
