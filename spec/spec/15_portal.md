
---

## 15. The portal — `adcs-portal` and `adcs-worker`

### 15.1 Shape

VLEO's daemon stays what it is: one local process on loopback, the engineer's workbench. The portal is a separate service for everyone else, and it never computes a number itself.

```
browser --TLS--> reverse proxy --> adcs-portal (axum + tokio)  --SQL--> PostgreSQL (operational data)
                                        |  enqueue                        ^
                                        v                                 | status, results
                                   run queue (a PostgreSQL table) <--- adcs-worker xN --> engine libraries
                                                                          |           (adcs-case, adcs-solve, adcs-sim, adcs-evidence)
                                                                          v
                                                                    run artefacts (object directory)
facility network --outbound only--> adcs-portal /internal/rig/stream   (live OILS/HILS witness)
```

- **No database on the physics path.** This is VLEO's rule, kept: workers read verified bundles from the local store and write run artefacts. PostgreSQL holds identity, entitlement, projects, the queue, the ledger index, quotes and orders: the "operational data" of VLEO's DELIVERY_PLAN.
- **The queue is a table.** Workers claim jobs with `SELECT … FOR UPDATE SKIP LOCKED`, so there is no second broker. A job runs in its own subprocess (§9.6: one flight software instance per process), with CPU, memory and time limits.
- **The rig pushes; the portal never reaches in.** The facility network makes outbound, authenticated connections to the portal to stream witness data. Nothing on the internet can address a rig host.

### 15.2 Roles

| Role | Can |
|---|---|
| `client_viewer` | see their tenant's projects, runs, quotes, orders, witness streams, evidence |
| `client_engineer` | as viewer, plus upload a case CSV or fill the case editor on screen, run the solver, switch products, run the release's scenarios and campaign types within quota, open result documents, request a design, upload an FMU, accept a quote, upload a PO |
| `sales` | all tenants' projects; price tables; issue quotes (with H-quote); accept POs |
| `engineer` | all projects read-only; run internal campaigns; the node library; send requests (node forms) and see their status |
| `designer` | design jobs; the internal catalogue view, candidates included; the design-request queue; send requests |
| `catalogue_owner` | decide to offer a catalogue candidate, or to retire a product (H14, with quality); the decision is recorded with their name, and the developer team writes it into the next catalogue release (§8.6) |
| `trainee` | download the MATLAB SILS twin (§10.8); nothing else, and no client data. A trainee is a member of the company's own tenant; nobody outside the company holds the role until D24 decides who may, and on what terms. |
| `test_operator` | schedule and run rig campaigns; the facility view; fault injection (internal) |
| `production` | upload each unit's measured descriptor at calibration, checked against the schema on upload (§7.4) |
| `quality` | the verification matrix, parity causes, deviations; sign certificates (H-cert) |
| `developer` | the requests inbox: every request, its file, its check report and its status; post replies (§5.11). Never content: a developer changes the software in the repository, not here. |
| `admin` | tenants, users, quotas; never content |

Every request is authorised against its tenant. A test suite tries every route as every role against another tenant's ids and expects refusal.

### 15.3 The order, as a state machine

```
enquiry -> project_open (case uploaded) -> sizing (solver) -> sils -> quoted --(client accepts, uploads PO)--> po_received
                                              \-> no_product -> design_requested -> (designer saves candidates -> added to the catalogue
                                                 by request in a release -> H14 decision recorded -> catalogue bundle published) -> sizing
  -> po_accepted (sales) -> building (production) -> calibrated (production, serials written)
  -> oils_scheduled -> oils_running -> oils_passed (test_operator)
  -> hils_scheduled -> hils_running -> hils_passed (test_operator)
  -> certifying -> certified (quality, H-cert) -> delivered -> in_flight
```

Any state can go to `on_hold` with a reason. A failed campaign goes back to its `_scheduled` state with a deviation record. Uploading a changed case, or choosing another product, after `quoted` voids the quote and returns to `sizing`, and the old quote stays on record. `no_product` is not a failure: it is the solver's honest answer, and it turns into a design request only when the client asks for one. Every transition is an audit-log row with actor, time and reason.

### 15.4 Data model

The tables: `tenant`, `app_user`, `membership (user, tenant, role)`, `session`, `project (tenant, title)`, `case (project, version, csv_path, csv_sha256, case_hash, schema, imported_at)` (a new upload is a new version; none is overwritten), `request (id, request_id, node, type, file_path, submitted_by, submitted_at, status, issue_url, last_reply_at)` (the inbox of §5.10.6; the file is the record), `decision (kind, subject, decided_by, role, decided_at, reason, record_path)` (every person's decision the software records: H14 promotions, retirements and bounds, H15 causes, panel sign-offs; `xtask decision record` reads the exported record), `solution (case, run, outcome, gap_report_path)`, `candidate (solution, product_id, tuned_set_hash, case_hash, classification, worst_margin, confirm_campaign_hash)`, `product (id, status, origin, class, file_sha256, bundle_version)` (the internal catalogue: the released catalogue's products, plus candidates the moment a portal design job saves them, or a designer uploads one from their store), `design_request (case, requested_by, state, product_ids)`, `design_job (id, class_or_case, toml_hash, state, saved_products)`, `campaign (project, case, product_id, scenario_id, scenario_hash, type, runs, hash, rung, status)` (a scenario is the release's, named by id and hash, never stored per project), `run (campaign, k, status, manifest_hash, artefact_path, result_path)`, `result (project, case, file_path, sha256, engine, kind, verdict, worst_margin, uploaded)` (every result document, made here or uploaded), `evidence_supply (project, row_id, value_si, rung, campaign_hash, runs, statistic, confidence)`, `parity_line`, `price_table (version, part_number, price, currency)`, `quote (project, candidate, hash, total, currency, catalogue_version, price_table_version, valid_until, document_path, state)`, `purchase_order (quote, document_path, received_at, accepted_by)`, `sales_order (quote, state, serials)`, `unit (serial, part_number, sales_order, descriptor_path, checked_at, recorded_by)`, `certificate (sales_order, package_hash, signed_by, signed_at)`, `fmu_upload (project, sha256, fmi_version, status)`, `quota (project, sils_runs_per_month)` (per project, as the layer-1 row states it), `comment`, `audit_log`.

Migrations are plain SQL files under `crates/adcs-portal/migrations/`, applied in order. A test applies them to an empty database, and another applies them then rolls them back.

### 15.5 Routes

Client-facing, JSON under `/api/v1`, server-sent events where marked:

```
POST /auth/login  /auth/logout  /auth/totp
GET  /projects                 POST /projects
GET  /case-template.csv                                             (the blank form)
GET  /projects/{id}            POST /projects/{id}/case            (a CSV upload, or the on-screen form, which writes the same CSV)
GET  /projects/{id}/case       GET  /projects/{id}/case.csv        GET /projects/{id}/case/report   (the import report)
POST /projects/{id}/solve      GET  /projects/{id}/solution        POST /projects/{id}/design-request
GET  /scenarios                (the release's scenarios and campaign types)
POST /projects/{id}/campaigns  GET  /campaigns/{id}                GET /campaigns/{id}/dashboard   (scenario id, type, runs)
GET  /projects/{id}/results    POST /projects/{id}/results         GET /results/{id}               (result documents: list, open an uploaded one, download)
GET  /runs/{id}                GET  /runs/{id}/stream   (SSE)       GET /runs/{id}/replay
POST /projects/{id}/fmu
POST /projects/{id}/quote-request                                 GET /quotes/{id}   POST /quotes/{id}/accept
POST /orders/{id}/po           GET  /orders/{id}                   GET /orders/{id}/witness (SSE)
GET  /orders/{id}/evidence     GET  /certificates/{id}
```

Internal, under `/api/v1/internal`, by role: price tables, quote issue, PO acceptance, rig scheduling, `rig/stream` (the facility's outbound push), unit descriptors (production), deviations, parity causes, certificate signing, design jobs and the design-request queue, the catalogue view with candidates, promotion and retirement decisions (recorded, H14), and:

```
GET  /api/v1/internal/forms/node/<node>      a node's form (internal roles only: layer 3 is restricted from clients)
GET  /api/v1/internal/forms/library          the node library, as a zip
POST /api/v1/internal/requests               send a filled node form: stored, an issue opened, status "received"
GET  /api/v1/internal/requests[/<id>]        the sender's requests and their status; every request for `developer`
POST /api/v1/internal/requests/<id>/reply    `developer` only: a reply, written into the returned file's history
GET  /api/v1/internal/downloads/matlab-sils  the latest release of the twin's zip, for internal roles and `trainee`
GET  /api/v1/internal/downloads/manual       the user manual
GET  /api/v1/internal/risk                   the Risk management conclusion, the register and each quarter's narrative (§5.13), read-only
POST /api/v1/internal/orders/{id}/units      production: one unit's measured descriptor, checked against the schema (§7.4)
GET  /api/v1/internal/orders/{id}/serials    the order's unit descriptors as one hashed folder, for the rig, fswcfg and eeprom (§10.5)
POST /api/v1/internal/decisions              a person records an H14 decision, a parity cause (H15) or a panel sign-off (§17.2)
```

No route changes a node, a scenario, a product file or any other part of the software. A request is stored and forwarded; the change is made in the repository, through intake (§5.11), and arrives with a release.

### 15.6 Client runs and uploaded models

- A client's SILS campaigns count against `quota`. The solver's screening costs nothing, and its tuning and confirmation run in the company's time, not the client's quota (D19 may change that); a client's own 500-run Monte Carlo costs 500.
- The case form on screen is the case editor of §8.3.3, served inside the page. It has the same keys, units, blank policies and checks as the CSV template. Submitting it writes a CSV and imports it through `adcs-case`, exactly as an upload does, so there is one path into the engine. The same page can be downloaded, filled offline, and uploaded later.
- A client picks a scenario of the release and a campaign type; a client never writes a scenario. A new kind of test a client needs reaches the company through sales.
- A client may upload a plant extension or a controller as an FMU (FMI 2.0 or 3.0, co-simulation). An FMU carries native code, so it runs only in a sandboxed worker: a separate user namespace (`bubblewrap` or `nsjail`), no network, a read-only root, and CPU, memory and wall-time limits. Only Linux x86-64 binaries are accepted, and a source FMU is compiled inside the sandbox. Whether to adopt a Rust FMI host crate or bind the FMI reference C headers is D16.
- Every SILS run a client starts is saved as a result document (§13.5) in the project, beside the case version it ran, and listed and downloadable there. The client can upload a result document they were sent, or saved earlier, to see it again in the portal without running anything: the portal files it beside its case, as the local store does (§13.5.4). A result made for a client leaves out layer-3 channels and restricted content.
- A client's FMU is the one input besides the case, and only in the portal. It extends the plant or replaces the controller for that client's runs; it never changes the software.
- A client never sees the reference flight software's source, the controller gains or the plant internals. They see their inputs, their runs' channels and metrics, and the evidence for their order.

### 15.7 Quote to purchase order

- **Price** comes from `price_table` (sales), never from the tree. The tree's layer-1 cost rows are the company's view of cost; the price table is what it charges.
- **A quote's identity** is `SHA-256` over the canonical JSON of: the case hash, the product id and file hash, the tuned-set hash, the kernel and graph hashes, the catalogue bundle version and hash, the price-table version, the currency and the validity date. A quote is a commercial record, so its hash is cryptographic. FNV stays the engine's cache key, as in VLEO.
- **Issuing a quote** is H-quote: a person in sales confirms the price and the export-classification check before the client sees it.
- **The purchase order** is a document the client uploads against a quote id and hash. Sales accepts it, and the order starts. A PO against a voided or expired quote is refused by name.

### 15.8 Security

- TLS at the proxy; `argon2id` password hashes; TOTP second factor for every account; server-side sessions in `HttpOnly`, `Secure`, `SameSite=Strict` cookies; CSRF tokens on every state-changing request.
- Tenant isolation is tested (§15.2). The audit log is append-only; the application role has no `UPDATE` or `DELETE` on it.
- Restricted content (D1 rows, fswcfg sections `0x05` and `0x06`, tuned-set values, other tenants' data) is excluded by the API layer and tested for (§14.6).
- Where the portal and client data are hosted, and what residency clients require, is D17.

### 15.9 Deployment

A first deployment is one server: reverse proxy with TLS, `adcs-portal`, two or more `adcs-worker`, PostgreSQL and the artefact directory, defined in `deploy/compose.yaml`. Every night, a database dump and an artefact sync go off the server. `docs/RUNBOOK.md` gains the restore procedure, and a monthly restore test is a scheduled task a person runs.
