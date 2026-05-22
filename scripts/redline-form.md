# Redline form (fill in once, then run `./redline-placeholders.sh apply`)

> 38 placeholder occurrences across 7 unique tokens.
> Open `/app/scripts/placeholders.csv` to see where each appears.

Fill in REAL values below. The script will substitute everywhere on apply.

```yaml
# Names
vp_platform:        "REPLACE_ME (e.g., Alice Chen)"
cto:                "REPLACE_ME"
ciso:               "REPLACE_ME"
cfo_delegate:       "REPLACE_ME"
head_of_sre:        "REPLACE_ME"
head_of_compliance: "REPLACE_ME"
head_of_devex:      "REPLACE_ME"
finops_lead:        "REPLACE_ME"
vp_revenue:         "REPLACE_ME"
platform_lead:      "REPLACE_ME"
compliance_pm:      "REPLACE_ME"
program_pm:         "REPLACE_ME"
procurement_lead:   "REPLACE_ME"

# Contact details (used in vendor cover emails)
platform_lead_phone:    "+1-555-XXX-XXXX"
compliance_pm_phone:    "+1-555-XXX-XXXX"

# Dates (override defaults only if your kickoff was not on the assumed date)
day_0:        "2026-01-06"
day_3_steerco: "2026-01-09"
day_5:        "2026-01-11"
week_4_gate:  "2026-02-06"
retrospective: "2026-02-13"
phase_1_kickoff: "2026-02-09"
phase_1_mid_point: "2026-03-20"
phase_1_gate: "2026-05-08"
```

When done, run:
```bash
./redline-placeholders.sh apply
```
