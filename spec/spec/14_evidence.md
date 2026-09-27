
---

## 14. Evidence and certification — `adcs-evidence`

### 14.1 The requirement package: before the order

Generated from the case, the selected product and its tuned-set hash, and shown to the client with the quote. The case CSV itself, as uploaded, is attached with its hash, so the requirement package can be traced back to the file the client sent:

- **ADCS requirement specification**: every written requirement row with its value, sense and analysis margin, laid out in the structure of ECSS-E-ST-60-30C (AOCS requirements). The pointing and knowledge requirements are stated as ECSS-E-ST-60-10C indices with both statistical levels (§10.3).
- **Interface control document**: power, data interfaces, mass, mounting and the EEPROM contents, from the chosen parts' descriptors.
- **Verification matrix**: per requirement, the method (test, analysis, review of design or inspection, per ECSS-E-ST-10-02C), the rung that will supply it (analysis, SILS, OILS, HILS) and the campaign that will run.

After the order, that matrix becomes the test plan.

### 14.2 The parity ledger

One line per scenario × metric × rung × engine. The engine is `platform`, or `matlab` for the MATLAB twin (§10.8.6). A certificate reads only the lines of its own order's campaigns, which are always `platform`. Twin lines belong to the development ledger: they hold no certificate, and their causes are the design team's work, not an order's. Each line holds:

- the value;
- the campaign hash;
- the difference from the rung before it;
- the difference from the analysis row;
- the difference from any parity reference;
- a **cause**, written by a person.

A difference without a cause is an open item that holds the certificate. Nothing is tuned to close a line: a model is changed only when the cause says the model was wrong, and the change is a reviewed commit that names the line.

### 14.3 Reports

Every campaign produces a report: configuration (parts and serials), flight software build id, scenario and campaign hashes, rig and lab, metrics with both statistics, pass or fail per bound requirement, deviations, and links to its runs. The report is HTML generated from the ledger. The PDF is produced by the same pipeline, and regenerating from the same ledger gives the same bytes; CI checks this.

### 14.4 The as-built twin

Before a unit's OILS campaign, the client's SILS scenarios are re-run with that unit's **serial** descriptors in place of the catalogue values. A unit whose as-built run no longer closes is caught before any rig time is spent on it. The as-built results are a parity-ledger line of their own.

### 14.5 The certificate of conformance

The certificate lists:

- the order, the unit serials and the flight software build id;
- every requirement with its closing evidence: rung, campaign, value and margin;
- open deviations and waivers;
- the parity ledger's state;
- the hash of the evidence package.

It is issued only when:

- every requirement in the verification matrix is closed or waived;
- every parity line has a cause;
- no evidence campaign names a synthetic or placeholder part;
- a person in quality signs it (H-cert).

The builder writes the generator and never a signatory. Until D3 there is no signing key, so the certificate carries its package hash and the signatory's name, and says it is unsigned.

### 14.6 The evidence package

A zip of the requirement package, the verification matrix, the parity ledger, every campaign report, the run manifests (without the full-rate channels unless requested), the certificate and a manifest of hashes. Restricted content is excluded by rule and never by hand: the controller gains and tuned parameters (fswcfg sections `0x05` and `0x06`), restricted rows (D1), and other clients' data. A test builds a package from a campaign that contains restricted values and checks that none of them are in the package.

### 14.7 Standards mapping

| Standard | Where it lands |
|---|---|
| ECSS-E-ST-60-30C, AOCS requirements | requirement specification structure (§14.1); layer-1 row "Requirement clauses covered" |
| ECSS-E-ST-60-10C, control performance | metric definitions and statistics (§10.3); the pointing budget (`gp_5`) |
| ECSS-E-ST-10-02C, verification | verification methods and the matrix (§14.1) |
| ECSS-E-ST-10-03C, testing | test levels applied at OILS and HILS; layer-1 row "Test levels applied" |
| ECSS-E-ST-40C and ECSS-Q-ST-80C, software | the flight software's process and CI (§9.6, §18); the software criticality category is D15 |

Which clauses are tailored is a person's decision per order, recorded as layer-1 "Tailoring items".
