# spanda-continuity(1)

## NAME

continuity — Mission continuity, takeover, delegation, and succession planning.

## SYNOPSIS

```
spanda continuity|takeover|delegate|succession <file.sd> [options]
```

## DESCRIPTION

Mission continuity, takeover, delegation, and succession planning.

## OPTIONS

`--failed <name>` — failed robot or entity
`--progress <pct>` — mission progress percent
`--trigger <kind>` — continuity trigger (e.g. `robot_failed`)
`--successor` / `--to <name>` — designated successor for takeover/delegate
`--scope fleet|swarm|robot` — succession scope
`--json` / `--markdown` / `--html` — report format

## EXAMPLES

```bash
spanda continuity examples/showcase/continuity/warehouse.sd --failed ScannerAlpha --progress 72
spanda takeover examples/showcase/takeover/patrol.sd --failed RoverA
spanda delegate examples/showcase/delegation/survey.sd --failed SurveyBot --to RelayBot
spanda succession examples/showcase/fleet_succession/delivery.sd --scope fleet
spanda demo continuity
```

## EXIT STATUS

0 when planning succeeds; 1 on validation or safety gate failures.

## FILES

`continuity_policy` and `mission_plan` declarations in `.sd` source.

## SEE ALSO

spanda-recovery(1), spanda-fleet(1), [mission-continuity.md](../mission-continuity.md), [spanda(1)](./spanda.md), [spanda-reference.md](../spanda-reference.md)
