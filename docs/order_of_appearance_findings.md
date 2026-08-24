# Order of Appearance — implementation audit

Source: `Boardgame - Remember_Gordon/tables/order_of_appearance.txt` (Campaign scenario,
§9.112 Dervish / §9.113 Anglo-Egyptian). Audit date: 2026-08-24.

## Implementation locations

| Piece | Location |
|---|---|
| Wave schedules | `omdurman-rules/src/reinforcements.rs:74` (`dervish_campaign_schedule`), `:131` (`anglo_egyptian_campaign_schedule`) |
| Enforcement | `omdurman-rules/src/effects.rs:4167` (`GameState::validate_campaign_reinforcements`) |
| Application + entry MP costs | `omdurman-rules/src/effects.rs:4707` (`apply_place_reinforcements`) |
| Stacking + enemy-occupied checks | `omdurman-rules/src/effects.rs:4118` (`can_place_reinforcements`) |

## Verified as matching

### Dervish (§9.112)

| Turn | Table file | Code wave | Status |
|---|---|---|---|
| 1 | YAKUB (12 Baggara, 25 Jaalin), SHERIF (4 Danagla), ALI WAD HEIJI (6 Kehena, 5 Degheim) | leaders Yakub/Sherif/AliWadHelu; tribes Baggara/Jaalin/Danagla/Kehena/Degheim | match |
| 2 | OSMAN DIGNA (12 Hadendowa) | leader OsmanDigna; tribe Hadendowa | match |
| 3 | SHEIK EL DIN (32 Mulazmin, 24 Jehadia) | leader SheikElDin; tribes Mulazmin/Jehadia | match |
| 4 | None more | no wave → `NoReinforcementWave` error | match |

Enforced per-placement by `TribeNotInWave` gating (tribe or leader must belong to the
current turn's wave); wrong-side placements rejected with `NotYourTurn`; non-wave phase
rejected with `WrongPhase`.

### Anglo-Egyptian (§9.113)

| Rule | Implementation | Status |
|---|---|---|
| Leaders enter anytime, free | listed in all four waves, exempt from cap | match |
| ≤3 gunboats per turn | gunboat quota counted separately from land cap (`GunboatQuotaExceeded`) | match |
| Turns 2–3: any 12 land units | `unit_cap = Some(12)` (`ReinforcementCapExceeded`) | match |
| Turn 4: all remaining | `unit_cap = None`, `all_remaining = true` | match |
| Entry areas | annotated `NamedArea` hexes enforced when present: AE entrance area / GunboatNorthEdge / AbuAlimHut (`OutsideEntranceArea`) | match |
| Entry costs | AE 1 MP; "Friendlies" 8 MP via Abu Alim hut (effects.rs:4725); Dervish pay terrain cost of entry hex | match |
| No double entry | `AlreadyDeployed` | match |
| May not enter on enemy-occupied hex (§7.1) | `EnemyOccupied` | match |

Units that skipped an earlier wave may still enter later — the schedule gates, it does not
expire (documented at effects.rs:4160).

## Deviations from a strict reading (deliberate, documented in code)

1. **Turn 1 force list is not composition-gated.** §9.113 names the turn-1 arrivals ("Any
   three gunboats; 'Friendlies' brigade; Egyptian Cavalry; Horse Artillery; and two infantry
   brigades from the Egyptian Division"). The code treats turn 1 like turns 2–3: any land
   units up to the 12-cap plus the gunboat quota. E.g. entering a British brigade on turn 1
   is accepted but not permitted by the strict table reading.
   Location: `anglo_egyptian_campaign_schedule`, reinforcements.rs:131.

2. **"All three leaders must be in play by the end of turn four!" is not enforced.** The
   leaders are *available* every turn 1–4, but nothing validates at end of turn 4 that they
   were actually placed; skipping them silently forfeits them.
   Location: same schedule function; would need an end-of-turn-4 check.

## Test coverage

- `reinforcements::tests::*` — schedule shape per side (waves, tribes, leaders, caps).
- `effects::tests::campaign_reinforcements_gate_by_wave` — tribe gating + turn ownership.
- `effects::tests::campaign_reinforcement_cap_and_double_entry` — 12-cap, double-entry,
  1 MP charge.
- `effects::tests::campaign_gunboats_quota_three_per_turn` — gunboat quota.
- `effects::tests::reinforcement_rejected_onto_enemy_occupied_hex` — §7.1 occupancy guard.
- `effects::tests:*` asserting Friendlies pay 8 MP (see `mp_spent(id) == 8`,
  effects.rs:6262).

All green (`cargo +1.97.1 test -p omdurman-rules --lib`: 356 passed).
