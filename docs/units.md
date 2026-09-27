# All known unit counters

Every cell in `omdurman-rules/src/sprite_data.rs` (the source of truth), grouped by sprite sheet section (the `SectionName`) in `SectionName::SHEET_ORDER` order. Sprite files are `<serde name>_<col>_<row>.webp` under `omdurman-app/assets/sprites/` (e.g. `Ali_Wad_Helu_0_1.webp` is `AliWadHelu` cell 0,1). Stats compacted as `fire,melee,movement` (forts: `fire,melee`; gunboats: `fire,upstream,downstream`, labelled). 222 of 238 physical sheet cells are annotated: 221 counters plus the Hadendowa 7,0 `GAME TURN` marker. The 16 unannotated cells (including the six BritishBoats BREECH markers) are listed at the end. `sprite_data.rs` also defines 6 synthetic marker cells — Danagla, Degheim, Kehena, Mulazmin, OsmanDigna, Yakub — that exist in code but not on the sheet; they have their own table after the sheet sections.

The "Kind & stats" column is the raw sprite annotation. The engine's classification (`omdurman-rules/src/unit_profiles.rs`) resolves some cells differently: the leader cells (e.g. `Khalifa Abdullah` 1,1,15, `Lord Kitchener Sirdar` 0,0,15, `Gen. Gordon` 0,0,0) become leaders, KhalifaAbdullah 0,1/1,1/2,1 become Dervish artillery, and the `GAME TURN` marker is not a unit. Editor-authored overrides would come from `omdurman-app/assets/sprite_annotations.ron` (currently empty).

Note: the 32 MulazminI/MulazminII Mulazmin counters print the tribe name `Mulazmin` in the compiled data. This is a deliberate deviation (the original sheet annotations record no printed text for those cells) so the picker can label them.

## `Taiasha`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 0,1 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 1,0 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 1,1 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 2,0 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 2,1 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 3,0 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 3,1 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 4,0 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 4,1 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 5,0 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 5,1 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 6,0 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |
| 6,1 | Dervish | Taiasha | Infantry 3,6,9 | BlackWhite | Taiasha |

## `MulazminI`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 0,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 1,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 1,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 2,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 2,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 3,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 3,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 4,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 4,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 5,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 5,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 6,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 6,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 7,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 7,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |

## `KhalifaAbdullah`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Baggara | Infantry 1,1,15 | BlackWhite | Khalifa Abdullah |
| 0,1 | Dervish | Baggara | Infantry 6,1,7 | BlackWhite | - |
| 1,0 | Dervish | Baggara | Gunboat fire:4,upstream:10,downstream:16 | BlackWhite | Gunboat |
| 1,1 | Dervish | Baggara | Infantry 6,1,7 | BlackWhite | - |
| 2,0 | Dervish | Baggara | Gunboat fire:4,upstream:10,downstream:16 | BlackWhite | Gunboat |
| 2,1 | Dervish | Baggara | Infantry 6,1,7 | BlackWhite | - |

## `Sherif`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Danagla | Infantry 1,1,15 | RedBlack | Sherif |
| 1,0 | Dervish | Danagla | Infantry 4,6,12 | RedBlack | - |
| 1,1 | Dervish | Danagla | Infantry 4,6,12 | RedBlack | - |
| 2,0 | Dervish | Danagla | Infantry 4,6,12 | RedBlack | - |
| 2,1 | Dervish | Danagla | Infantry 4,6,12 | RedBlack | - |

## `MulazminII`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 0,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 1,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 1,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 2,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 2,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 3,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 3,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 4,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 4,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 5,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 5,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 6,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 6,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 7,0 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |
| 7,1 | Dervish | Mulazmin | Infantry 3,6,9 | GreenRed | Mulazmin |

## `JaalinI`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Jaalin | Infantry 1,1,15 | GrayBlack | Yakub |
| 0,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 1,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 1,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 2,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 2,1 | Dervish | Baggara | Infantry 3,6,12 | GrayBlack | Jaalin |
| 3,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 3,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 4,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 4,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 5,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 5,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 6,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 6,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |

## `Hadendowa`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | IsaZachneih | Infantry 8,6,9 | WhiteBlack | Isa Zachneih |
| 0,1 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 1,0 | Dervish | Hadendowa | Infantry 1,1,15 | WhiteBlack | Osman Digna |
| 1,1 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 2,0 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 2,1 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 3,0 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 3,1 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 4,0 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 4,1 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 5,0 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 5,1 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 6,0 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 6,1 | Dervish | Hadendowa | Infantry 3,7,9 | WhiteBlack | Hadendowa |
| 7,0 | Dervish | Hadendowa | Marker | WhiteBlack | GAME TURN |
| 7,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |

## `JaalinII`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 0,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 1,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 1,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 2,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 2,1 | Dervish | Baggara | Infantry 3,6,12 | GrayBlack | Jaalin |
| 3,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 3,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 4,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 4,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 5,0 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |
| 5,1 | Dervish | Jaalin | Infantry 3,6,12 | GrayBlack | Jaalin |

## `HadendowaForts`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 0,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 1,0 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 1,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 2,0 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 2,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 3,0 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 3,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 4,0 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 4,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 5,0 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 5,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 6,0 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 6,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 7,0 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |
| 7,1 | Dervish | Hadendowa | Fort 4,1 | WhiteBlack | Hadendowa Fort |

## `Baggara`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 0,1 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 1,0 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 1,1 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 2,0 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 2,1 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 3,0 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 3,1 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 4,0 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 4,1 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 5,0 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |
| 5,1 | Dervish | Baggara | Infantry 3,6,15 | GrayRed | Baggara |

## `BritishBoats`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 3,0 | BritishEgyptian | - | Gunboat fire:5,upstream:12,downstream:18 | SandBlack | Abu Klea |
| 3,1 | BritishEgyptian | - | Infantry 0,0,0 | SandBlack | Gen. Gordon |
| 4,0 | BritishEgyptian | - | Gunboat fire:5,upstream:12,downstream:18 | SandBlack | Sultan |
| 4,1 | BritishEgyptian | - | Gunboat fire:4,upstream:10,downstream:16 | SandBlack | Gunboat |
| 5,0 | BritishEgyptian | - | Gunboat fire:5,upstream:12,downstream:18 | SandBlack | Sheik |
| 5,1 | BritishEgyptian | - | Gunboat fire:4,upstream:10,downstream:16 | SandBlack | Gunboat |
| 6,0 | BritishEgyptian | - | Gunboat fire:5,upstream:12,downstream:18 | SandBlack | Fateh |
| 6,1 | BritishEgyptian | - | Gunboat fire:4,upstream:10,downstream:16 | SandBlack | Gunboat |
| 7,0 | BritishEgyptian | - | Gunboat fire:5,upstream:12,downstream:18 | SandBlack | Melik |
| 7,1 | BritishEgyptian | - | Gunboat fire:4,upstream:10,downstream:16 | SandBlack | Gunboat |

## `AliWadHelu`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Baggara | Infantry 1,1,15 | BlueBlack | Ali Wad Helu |
| 0,1 | Dervish | Kehena | Infantry 3,6,9 | BlueRed | Deghelim |
| 1,0 | Dervish | Baggara | Infantry 3,6,9 | BlueBlack | Deghelim |
| 1,1 | Dervish | Kehena | Infantry 3,6,9 | BlueRed | Deghelim |
| 2,0 | Dervish | Baggara | Infantry 3,6,9 | BlueBlack | Deghelim |
| 2,1 | Dervish | Kehena | Infantry 3,6,9 | BlueRed | Deghelim |
| 3,0 | Dervish | Baggara | Infantry 3,6,9 | BlueBlack | Deghelim |
| 3,1 | Dervish | Kehena | Infantry 3,6,9 | BlueRed | Deghelim |
| 4,0 | Dervish | Baggara | Infantry 3,6,9 | BlueBlack | Deghelim |
| 4,1 | Dervish | Kehena | Infantry 3,6,9 | BlueRed | Deghelim |
| 5,0 | Dervish | Baggara | Infantry 3,6,9 | BlueBlack | Deghelim |
| 5,1 | Dervish | Kehena | Infantry 3,6,9 | BlueRed | Deghelim |

## `BritishArmy`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | BritishEgyptian | - | Infantry 8,5,15 | SandBlack | 21 Lancers |
| 0,1 | BritishEgyptian | - | Infantry 10,5,8 | SandBlack | Cameron II. |
| 1,0 | BritishEgyptian | - | Infantry 5,3,8 | SandBlack | Royal Eng. |
| 1,1 | BritishEgyptian | - | Infantry 10,5,8 | SandBlack | Seaforth II. |
| 2,0 | BritishEgyptian | - | Infantry 10,1,7 | SandBlack | 32 Battery |
| 2,1 | BritishEgyptian | - | Infantry 10,5,8 | SandBlack | Lincolnshire |
| 3,0 | BritishEgyptian | - | Infantry 10,1,7 | SandBlack | 37 Battery |
| 3,1 | BritishEgyptian | - | Infantry 10,5,8 | SandBlack | Warwicksh. |
| 4,0 | BritishEgyptian | - | Infantry 6,1,12 | SandBlack | Maxim Batt. |
| 4,1 | BritishEgyptian | - | Infantry 10,5,8 | SandBlack | Rifle Brig. |
| 5,0 | BritishEgyptian | - | Infantry 6,1,12 | SandBlack | Maxim Batt. |
| 5,1 | BritishEgyptian | - | Infantry 10,5,8 | SandBlack | Gren. Grds. |
| 6,0 | BritishEgyptian | - | Infantry 6,1,12 | SandBlack | Maxim Batt. |
| 6,1 | BritishEgyptian | - | Infantry 10,5,8 | SandBlack | Lancas. Fus. |
| 7,0 | BritishEgyptian | - | Infantry 6,1,12 | SandBlack | Maxim Batt. |
| 7,1 | BritishEgyptian | - | Infantry 10,5,8 | SandBlack | Northn. Fus. |

## `SheikElDin`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Jehadia | Infantry 1,1,15 | GreenBlack | Sheik El Din |
| 0,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 1,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 1,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 2,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 2,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 3,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 3,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 4,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 4,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 5,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 5,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 6,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 6,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |

## `Kitchener`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | BritishEgyptian | - | Infantry 0,0,15 | SandBlack | Lord Kitchener Sirdar |
| 0,1 | BritishEgyptian | - | Infantry 8,6,9 | SandGreen | "Friendlies" |
| 1,0 | BritishEgyptian | - | Infantry 0,0,15 | SandBlack | Gen. Gatacre Brit. Div. |
| 1,1 | BritishEgyptian | - | Infantry 8,6,9 | SandGreen | "Friendlies" |
| 2,0 | BritishEgyptian | - | Infantry 0,0,15 | SandBlack | Gen. Hunter Egy. Div. |
| 2,1 | BritishEgyptian | - | Infantry 8,6,9 | SandGreen | "Friendlies" |
| 3,0 | BritishEgyptian | - | Infantry 8,5,12 | SandRed | Camel Corps |
| 3,1 | BritishEgyptian | - | Infantry 8,6,9 | SandGreen | "Friendlies" |
| 4,0 | BritishEgyptian | - | Infantry 8,5,12 | SandRed | Camel Corps |
| 4,1 | BritishEgyptian | - | Infantry 8,6,9 | SandGreen | "Friendlies" |
| 5,0 | BritishEgyptian | - | Infantry 9,5,8 | SandRed | IX. Sud. |
| 5,1 | BritishEgyptian | - | Infantry 9,5,8 | SandRed | XII. Sud. |
| 6,0 | BritishEgyptian | - | Infantry 9,5,8 | SandRed | X. Sud. |
| 6,1 | BritishEgyptian | - | Infantry 9,5,8 | SandRed | XIII. Sud. |
| 7,0 | BritishEgyptian | - | Infantry 9,5,8 | SandRed | XI. Sud. |
| 7,1 | BritishEgyptian | - | Infantry 9,5,8 | SandRed | XIV. Sud. |

## `Jehadia`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 0,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 1,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 1,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 2,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 2,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 3,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 3,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 4,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 4,1 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |
| 5,0 | Dervish | Jehadia | Infantry 8,6,9 | GreenBlack | Jehadia |

## `EgyptianArmy`

| Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|
| 0,0 | BritishEgyptian | - | Infantry 10,5,15 | WhiteSand | Egy. Cav. |
| 0,1 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | III Egy. |
| 1,0 | BritishEgyptian | - | Infantry 10,5,15 | WhiteSand | Egy. Cav. |
| 1,1 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | IV Egy. |
| 2,0 | BritishEgyptian | - | Infantry 6,1,12 | WhiteSand | Horse Art. |
| 2,1 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | VII Egy. |
| 3,0 | BritishEgyptian | - | Infantry 8,1,7 | WhiteSand | Egy. Batt. |
| 3,1 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | XV Egy. |
| 4,0 | BritishEgyptian | - | Infantry 8,1,7 | WhiteSand | Egy. Batt. |
| 4,1 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | I Egy. |
| 5,0 | BritishEgyptian | - | Infantry 8,1,7 | WhiteSand | Egy. Batt. |
| 5,1 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | V Egy. |
| 6,0 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | II Egy. |
| 6,1 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | VI Egy. |
| 7,0 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | VIII Egy. |
| 7,1 | BritishEgyptian | - | Infantry 9,5,8 | WhiteSand | XVI Egy. |

## Synthetic marker cells (in code, not on the sheet)

| Section | Cell | Faction | Tribe / Brigade | Kind & stats | Color | Printed text |
|---|---|---|---|---|---|---|
| Danagla | 0,0 | - | - | Marker | SandBlack | - |
| Degheim | 0,0 | - | - | Marker | SandBlack | - |
| Kehena | 0,0 | - | - | Marker | SandBlack | - |
| Mulazmin | 0,0 | - | - | Marker | SandBlack | - |
| OsmanDigna | 0,0 | - | - | Marker | SandBlack | - |
| Yakub | 0,0 | - | - | Marker | SandBlack | - |

## Unannotated sheet cells (no values in `sprite_data.rs`)

| Section | Cells | Note |
|---|---|---|
| Sherif | 0,1 | - |
| JaalinII | 6,0, 6,1 | - |
| Baggara | 6,0, 6,1 | - |
| BritishBoats | 0,0, 0,1, 1,0, 1,1, 2,0, 2,1 | BREECH markers (§6.63); `unit_profiles::british_boats` resolves them to no placeable unit |
| AliWadHelu | 6,0, 6,1 | - |
| Jehadia | 5,1, 6,0, 6,1 | - |
