-- What the manual leaves open (kind: ambiguity | gap | inconsistency), and the historical notes.

INSERT INTO open_question [
  { id: open_question:fired_at_once_scope, kind: "ambiguity", question: "Does 'may only be fired at once' limit each unit or each hex/stack?", refs: [section:`6.14`, rule:r6_14_once, section:`6.42`],
    reading: "6.14 says 'a combat unit … may only be fired at once'; fire is directed at enemy-occupied hexes (6.14, 6.22)." },
  { id: open_question:desertion_rounding, kind: "gap", question: "How is 1½ × an odd desertion roll rounded, and how does a d10 face read (0 or 10)?", refs: [section:`8.2`, rule:r8_2_desertion] },
  { id: open_question:fort_melee_reference, kind: "inconsistency", question: "6.54 says forts may be destroyed by infantry melee attack 'see 7.6', but 7.6 says nothing about forts. How does melee destroy a fort, and what happens to its garrison?",
    refs: [section:`6.54`, section:`7.6`, rule:r6_54_fort_destruction] },
  { id: open_question:howitzer_accuracy_bonus, kind: "ambiguity", question: "Does the Anglo-Egyptian +1 accuracy bonus apply to howitzer fire?", refs: [section:`6.24`, section:`6.64`, modifier:ae_direct_fire],
    reading: "6.24 grants it to 'all Anglo-Egyptian direct fire attacks'; howitzer fire ignores line of sight and is fired in a different subphase than direct fire (6.42, 6.64)." },
  { id: open_question:howitzer_maxim_scatter, kind: "gap", question: "When howitzer fire allocated with Maxim fire scatters, how is the combined attack resolved?", refs: [section:`6.42`, rule:r6_42_combine_howitzer] },
  { id: open_question:friendlies_accuracy_bonus, kind: "ambiguity", question: "Do the Friendlies get the Anglo-Egyptian +1 direct fire bonus while firing on the Dervish Range Effects Table?", refs: [section:`6.24`, section:`6.52`, unit_class:friendlies] },
  { id: open_question:melee_crt_column, kind: "gap", question: "Which column does each side use on the Combat Results Table in melee (total melee strength of the participants?), and how are attacks on one hex from several hexes combined?", refs: [section:`7.7`, rule:r7_7_resolution] },
  { id: open_question:melee_losses_scope, kind: "ambiguity", question: "Does 'melee losses must be taken from meleeing units first' bind the defender, or only attackers with non-participating stackmates?", refs: [section:`7.7`, rule:r7_7_losses_first] },
  { id: open_question:retreat_timing, kind: "gap", question: "When is a cavalry/camel retreat declared, does the attacker still roll, may the attacker advance into the vacated hex, and may the retreat enter or pass enemy ZOCs?", refs: [section:`7.5`, rule:r7_5_retreat] },
  { id: open_question:disrupted_melee_defense, kind: "ambiguity", question: "May disrupted units melee defend? The back cover says they 'may not melee'; 7.4 says all units except gunboats may melee defend.", refs: [section:`Disrupted`, section:`7.4`, state:disrupted] },
  { id: open_question:excess_eliminations, kind: "gap", question: "What happens to a '#' result larger than the number of units in the target hex?", refs: [section:`CombatResults`, combat_result:eliminated] },
  { id: open_question:disrupt_leaders_forts, kind: "gap", question: "Can Anglo-Egyptian leaders or forts be disrupted (it matters for the Tomb)?", refs: [section:`CombatResults`, section:`9.14`, unit_class:ae_leader, unit_class:fort] },
  { id: open_question:british_leader_term, kind: "ambiguity", question: "9.14 says 'British leader'; are all Anglo-Egyptian leaders (Kitchener, Gatacre, Hunter) British leaders for the Tomb and for victory points?", refs: [section:`9.14`, unit:kitchener, unit:gatacre, unit:hunter] },
  { id: open_question:engineers_timing, kind: "ambiguity", question: "Royal Engineers that move adjacent can complete demolition at the end of the same player turn, whereas Zariba builders must begin and end the turn adjacent. Intended?", refs: [section:`6.53`, section:`5.3`] },
  { id: open_question:builder_defensive_fire, kind: "gap", question: "May a unit constructing the Zariba fire defensively? Only offensive fire and melee attack are forbidden.", refs: [section:`5.3`, rule:r5_3_constructor_restrictions] },
  { id: open_question:engineers_build_zariba, kind: "gap", question: "Is the Royal Engineers unit an 'Anglo-Egyptian infantry unit' for Zariba construction? Are the Friendlies?", refs: [section:`5.3`, unit:royal_engineers, unit_class:friendlies] },
  { id: open_question:advance_overruns_leader, kind: "ambiguity", question: "Does an advance after combat into a hex holding a lone Anglo-Egyptian leader eliminate it? 9.346 says so explicitly for Gordon only.", refs: [section:`6.51`, section:`9.346`] },
  { id: open_question:lone_leader_fire, kind: "gap", question: "Can fire target a hex holding only an Anglo-Egyptian leader, and what would a hit do? 6.51 names only overrun and the loss of all stackmates.", refs: [section:`6.51`, unit_class:ae_leader] },
  { id: open_question:night_range_rounding, kind: "gap", question: "Exact night ranges depend on the Range Effects Table cells, which are not in the manual text.", refs: [section:`8.1`, rule:r8_1_ranges_halved] },
  { id: open_question:mine_drift_trigger, kind: "gap", question: "Does drifting into a mined hex trigger it? Does a mine-damaged gunboat trigger the second mine?", refs: [section:`10.12`, section:`10.13`] },
  { id: open_question:chain_complete_turn, kind: "ambiguity", question: "Is 'one complete turn' for sinking the chain a full game turn or one Anglo-Egyptian player turn?", refs: [section:`10.23`] },
  { id: open_question:historical_leaders_count, kind: "gap", question: "Do leaders count as 'units eliminated' for the historical scenario's victory levels?", refs: [section:`9.24`] },
  { id: open_question:fok_later_turns, kind: "gap", question: "In FALL OF KHARTOUM only turn 1 is stated to be a night turn; are later turns day turns?", refs: [section:`9.341`, scenario:fall_of_khartoum] },
  { id: open_question:friendlies_brigade_integrity, kind: "ambiguity", question: "Can the Friendlies brigade get brigade integrity? 5.54 speaks of British, Sudanese and Egyptian infantry units.", refs: [section:`5.54`, unit_class:friendlies] },
  { id: open_question:artillery_meaning, kind: "ambiguity", question: "Does 'only artillery' (6.61–6.63, 10.23) include gunboats and forts firing their artillery factor on the artillery line?", refs: [section:`6.61`, section:`6.62`, section:`6.63`, section:`2.31`, section:`2.32`, section:`6.54`],
    reading: "The corpus records capability edges for gunboats and forts with basis 'implied'." },
  { id: open_question:advance_leaders_stacking, kind: "gap", question: "Do leaders count toward the stacking limit that caps the Dervish compulsory advance (7.6)? 5.51 makes them free stacking.", refs: [section:`7.6`, section:`5.51`] },
  { id: open_question:vp_overlap, kind: "ambiguity", question: "Are VP awards cumulative where categories overlap (a British leader is also an Anglo-Egyptian land unit; a Friendlies unit is also a land unit; the Khalifa is also a Dervish unit)?",
    refs: [section:`9.14`, vp_award:d_british_leader, vp_award:d_land_unit, vp_award:d_friendlies_east, vp_award:ae_khalifa, vp_award:ae_dervish_unit] },
  { id: open_question:turn_marker_hour, kind: "inconsistency", question: "4.C says to advance the Game Turn marker 'to the next hour', but day turns are two hours (1.2) and the track steps by two hours.", refs: [section:`4.C`, section:`1.2`, component:turn_record_track] },
  { id: open_question:gunboat_land, kind: "gap", question: "May gunboats leave Nile River hexes? 5.22 says only gunboats may enter Nile hexes, not that gunboats must stay in them.", refs: [section:`5.22`] },
  { id: open_question:dervish_defensive_maxims, kind: "gap", question: "Dervish units have no second fire subphase; do any Dervish units fire twice (Dervish gunboats)? 6.14 names gunboats as an exception to firing once without saying which.", refs: [section:`6.14`, section:`6.42`, unit_class:d_gunboat] },
  { id: open_question:dervish_cavalry, kind: "gap", question: "Which Dervish units, other than the sample Danagla camel counter, are cavalry or camel units (and so may retreat under 7.5)?", refs: [section:`7.5`, section:`2.3`] },
  { id: open_question:tribe_colors, kind: "gap", question: "The manual gives a color only for the green command (Mulazmin, Jehadia); other tribes' colors and the individual turn-1 leader assignments are on the counters, not in the text.", refs: [section:`5.52`, section:`9.112`, command_group:turn1_leaders] },
  { id: open_question:fort_target_vs_garrison, kind: "gap", question: "Is artillery fire at a fort (6.62) a separate attack from fire at the units stacked inside it, and may the same fire do both?", refs: [section:`6.62`, section:`6.54`, rule:r6_62_fort_target] },
  { id: open_question:zero_superiority, kind: "gap", question: "What is the campaign result when both sides' victory point totals are equal? The 9.14 bands start at 1 point.", refs: [section:`9.14`, scenario:campaign] },
  { id: open_question:historical_equal_levels, kind: "gap", question: "In the historical scenario, what is the net result when both sides reach the same victory level (net 0)? 9.24's table starts at 1 = DRAW.", refs: [section:`9.24`, scenario:historical] },
  { id: open_question:gordon_late_elimination, kind: "ambiguity", question: "If GORDON is eliminated on turn 7 or 8, is the result the British level for having survived the end of turn 6 or 7? 9.35 lists only survival thresholds for those turns.", refs: [section:`9.35`, unit:gordon] },
  { id: open_question:fok_gunboats_night, kind: "ambiguity", question: "Are British gunboat movement allowances halved on FALL OF KHARTOUM's night turn 1, as 'all Anglo-Egyptian movement allowances' (8.1) would suggest?", refs: [section:`9.341`, section:`8.1`, unit_class:old_gunboat] },
  { id: open_question:gunboat_zoc_scope, kind: "gap", question: "How far does a gunboat's ZOC against enemy gunboats reach: across a river chain, or into Nile hexes beside islands?", refs: [section:`5.41`, section:`5.44`, section:`10.23`, unit_class:gunboat] },
  { id: open_question:fort_stacking, kind: "gap", question: "Does a fort count toward the four-unit stacking limit of its hex? 5.51 excepts only leaders and gunboats; 6.54 speaks of friendly units 'stacked inside the fort'.", refs: [section:`5.51`, section:`6.54`, unit_class:fort] },
  { id: open_question:move_through_enemy, kind: "gap", question: "May units move through hexes occupied by enemy units? Only moving through friendly units (5.51) and through a hex holding a lone Anglo-Egyptian leader (6.51) are mentioned.", refs: [section:`5.51`, section:`6.51`, action:move_through_friendly] },
  { id: open_question:disrupt_disrupted, kind: "gap", question: "What does a D result do to units that are already disrupted?", refs: [section:`CombatResults`, section:`Disrupted`, state:disrupted] },
  { id: open_question:range_cells, kind: "gap", question: "The Range Effects Table, Terrain Effects Chart, Line of Sight Table, Combat Results Table and Scattergram cells are not transcribed in the manual text.", refs: [section:`2.2`, section:`6.22`, section:`6.23`, section:`6.3`] }
];
FOR $q IN (SELECT id, refs FROM open_question) { FOR $r IN $q.refs { RELATE ($q.id)->raises->$r; }; };

-- ------------------------------------------------------------------ persons & historical events (section 11)
INSERT INTO person [
  { id: person:mahdi, name: "Mohammed Ahmed Ibn Al-Sayid Abdullah", known_as: "the Mahdi", role: "proclaimed himself the Mahdi in 1881; died June 1885", section: section:`11` },
  { id: person:khalifa, name: "Abdullah the Taiasha", known_as: "the Khalifa", role: "succeeded the Mahdi; a chief of the Baggaras; made Omdurman his capital", counter: unit:khalifa_abdullah, section: section:`11` },
  { id: person:gordon, name: "General Sir Charles Gordon", role: "Governor General of the Sudan; killed at the fall of Khartoum, 25 Jan 1885", counter: unit:gordon, section: section:`11` },
  { id: person:kitchener, name: "General Herbert Kitchener", role: "Sirdar (commander) of the Egyptian army; led the 1896–1898 expedition", counter: unit:kitchener, section: section:`11` },
  { id: person:hunter, name: "General Hunter", role: "commanded Kitchener's Egyptian Division (re-occupied Berber, July 1897)", counter: unit:hunter, section: section:`11` },
  { id: person:osman_digna, name: "Osman Digna", role: "one of the Mahdi's Emirs; broke squares with his Hadendowa", counter: unit:osman_digna, section: section:`11` },
  { id: person:sheik_el_din, name: "Sheik El Din", role: "the Khalifa's son; commanded the Jehadia", counter: unit:sheik_el_din, section: section:`11` },
  { id: person:mahmud, name: "Mahmud", role: "one of the Khalifa's few remaining competent generals; captured at the Atbara", section: section:`11` },
  { id: person:wolseley, name: "General Sir Garnet Wolseley", role: "subdued Egypt in 1882; led the Gordon relief expedition 1884–85", section: section:`11` },
  { id: person:hicks, name: "General William Hicks (Hicks Pasha)", role: "his Egyptian army was destroyed at Kashgeil, November 1883", section: section:`11` },
  { id: person:baker, name: "Valentine Baker Pasha", role: "his expedition was destroyed in February 1884", section: section:`11` },
  { id: person:graham, name: "General Sir Gerald Graham", role: "fought Osman Digna at El-Teb (Feb 29) and again on March 12, 1884", section: section:`11` },
  { id: person:wilson, name: "Colonel Wilson", role: "commanded the desert strike force after casualties; delayed three days", section: section:`11` },
  { id: person:gladstone, name: "William Gladstone", role: "British prime minister", section: section:`11` },
  { id: person:victoria, name: "Queen Victoria", quotes: ["The government alone is to blame", "Surely now he is avenged"], section: section:`11` },
  { id: person:maxwell, name: "Maxwell", role: "with the XIII Sudanese, first to enter the city", section: section:`11` },
  { id: person:wad_el_nejumi, name: "Wad El Nejumi", role: "his forces are represented by Mulazmin units in FALL OF KHARTOUM", section: section:`9.322` },
  { id: person:abu_girgeh, name: "Abu Girgeh", role: "his forces are represented by Mulazmin units in FALL OF KHARTOUM", section: section:`9.322` },
  { id: person:sheik_el_obeid, name: "Sheik El Obeid", role: "his forces are represented by Mulazmin units in FALL OF KHARTOUM", section: section:`9.322` },
  { id: person:peter_bertram, name: "Peter Bertram", role: "game design, development, components design", section: section:`credits` }
];
INSERT INTO historical_event [
  { id: historical_event:mahdi_proclaimed, date: "1881", text: "Mohammed Ahmed proclaims himself the Mahdi", section: section:`11` },
  { id: historical_event:alexandria, date: "1882-05", text: "Mediterranean Squadron ordered to Alexandria; Marines and Bluejackets restore order", section: section:`11` },
  { id: historical_event:tel_el_kabir, date: "1882-09", text: "Wolseley wins Mahsama and Tel-el-Kabir and subdues Egypt", section: section:`11` },
  { id: historical_event:kashgeil, date: "1883-11-04", text: "Hicks Pasha's army destroyed at Kashgeil, about 225 miles southwest of Khartoum", section: section:`11` },
  { id: historical_event:baker_defeat, date: "1884-02", text: "Osman Digna's Hadendowa break Baker Pasha's square", section: section:`11` },
  { id: historical_event:el_teb, date: "1884-02-29", text: "Graham narrowly wins at El-Teb near Suakim", section: section:`11` },
  { id: historical_event:british_square_broken, date: "1884-03-12", text: "the Fuzzy-Wuzzies break a British square; Graham narrowly wins", section: section:`11` },
  { id: historical_event:khartoum_invested, date: "1884-03-12", text: "the Mahdi invests Khartoum", section: section:`11` },
  { id: historical_event:berber_falls, date: "1884-05", text: "Berber falls to the Mahdi; Khartoum isolated", section: section:`11` },
  { id: historical_event:abu_klea, date: "1885-01-17", text: "desert strike force attacked near Abu Klea; a British square broken", section: section:`11` },
  { id: historical_event:abu_kru, date: "1885-01-19", text: "Dervishes repulsed at Abu Kru", section: section:`11` },
  { id: historical_event:fall_of_khartoum, date: "1885-01-25", text: "some 20,000 Dervishes pour through the gap in the wall; the garrison and Gordon are killed", scenario: scenario:fall_of_khartoum, section: section:`11` },
  { id: historical_event:mahdi_dies, date: "1885-06", text: "the Mahdi dies; succeeded by the Khalifa", section: section:`11` },
  { id: historical_event:expedition_starts, date: "1896-07", text: "Kitchener's expedition gets under way, with the Sudan Military Railroad", section: section:`11` },
  { id: historical_event:berber_reoccupied, date: "1897-07", text: "Egyptian Division under Hunter re-occupies Berber", section: section:`11` },
  { id: historical_event:atbara, date: "1898-04", text: "Mahmud's zariba stormed at the Atbara: 3,000 Dervishes dead for 80 of Kitchener's men", section: section:`11` },
  { id: historical_event:kerreri_arrival, date: "1898-09-01", text: "Kitchener arrives at Kerreri", scenario: scenario:campaign, section: section:`11` },
  { id: historical_event:battle_of_omdurman, date: "1898-09-02", text: "Battle of Omdurman: by 11:30 virtually over; 10,000 Dervishes dead, 20,000 wounded, 5,000 prisoners, for just over 400 killed and wounded", scenario: scenario:historical, section: section:`11` },
  { id: historical_event:gordon_memorial, date: "1898-09-04", text: "Kitchener holds a memorial service for Gordon before the ruins of the palace in Khartoum", section: section:`11` }
];
UPDATE historical_event:battle_of_omdurman SET game_note = "The game's own framing (1.1): fought September 2nd, 1898; broke the Dervish rebellion; gained Britain a million square miles and two million subjects.", framing_section = section:`1.1`;
FOR $e IN (SELECT id, scenario FROM historical_event WHERE scenario != NONE) { RELATE ($e.id)->appears_in->($e.scenario); };

-- section 11 facts attached to the game entities they explain
UPDATE unit_class:named_gunboat SET history = "The new shallow draft gunboats (140 by 24 feet, drawing 39 inches) carried 12 pounders, 6 pounders and Maxim guns on the upper deck and 4 inch howitzers on the gun deck (11).", history_section = section:`11`;
UPDATE unit:fort SET history = "To guard the approaches to the city, seventeen forts were constructed and armed with old artillery pieces (11).", history_section = section:`11`;
UPDATE tribe:danagla SET history = "issued the few guns available (old Remingtons and brass muzzle loaders using home made cartridges), with the Jehadia (11)", history_section = section:`11`;
UPDATE tribe:jehadia SET history_section = section:`11`;
UPDATE open_question:artillery_meaning SET refs += section:`10.23`;
RELATE open_question:artillery_meaning->raises->section:`10.23`;
