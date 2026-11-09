-- Presentation, not rules: the counter-sheet image of each unit, tribe and unit class, as cut for
-- the game (omdurman-app/assets/sprites/<sprite>.webp, keyed by counter-sheet section, column and
-- row). The archive (web/) draws its counters from these. A class names one example counter.

-- named counters
UPDATE unit:kitchener SET sprite = "Kitchener_0_0";
UPDATE unit:gatacre SET sprite = "Kitchener_1_0";
UPDATE unit:hunter SET sprite = "Kitchener_2_0";
UPDATE unit:gordon SET sprite = "British_Boats_3_1";
UPDATE unit:khalifa_abdullah SET sprite = "Khalifa_Abdullah_0_0";
UPDATE unit:yakub SET sprite = "Jaalin_I_0_0";
UPDATE unit:sherif SET sprite = "Sherif_0_0";
UPDATE unit:ali_wad_helu SET sprite = "Ali_Wad_Helu_0_0";
UPDATE unit:osman_digna SET sprite = "Hadendowa_1_0";
UPDATE unit:sheik_el_din SET sprite = "Sheik_El_Din_0_0";
UPDATE unit:isa_zachneih SET sprite = "Hadendowa_0_0";
UPDATE unit:danagla_sample SET sprite = "Sherif_1_0";
UPDATE unit:dervish_artillery SET sprite = "Khalifa_Abdullah_0_1";
UPDATE unit:dervish_gunboat SET sprite = "Khalifa_Abdullah_1_0";
UPDATE unit:fort SET sprite = "Hadendowa_Forts_0_0";
UPDATE unit:lancers_21 SET sprite = "British_Army_0_0";
UPDATE unit:egyptian_cavalry SET sprite = "Egyptian_Army_0_0";
UPDATE unit:camel_corps SET sprite = "Kitchener_3_0";
UPDATE unit:battery_32 SET sprite = "British_Army_2_0";
UPDATE unit:horse_artillery SET sprite = "Egyptian_Army_2_0";
UPDATE unit:egyptian_battalion_artillery SET sprite = "Egyptian_Army_3_0";
UPDATE unit:maxim_battery SET sprite = "British_Army_4_0";
UPDATE unit:royal_engineers SET sprite = "British_Army_1_0";
UPDATE unit:egyptian_battalion_sample SET sprite = "Egyptian_Army_0_1";
UPDATE unit:old_gunboat SET sprite = "British_Boats_4_1";
UPDATE unit:sultan SET sprite = "British_Boats_4_0";

-- tribes (one counter of each)
UPDATE tribe:baggara SET sprite = "Baggara_0_0";
UPDATE tribe:jaalin SET sprite = "Jaalin_II_0_0";
UPDATE tribe:danagla SET sprite = "Sherif_1_0";
UPDATE tribe:kehena SET sprite = "Ali_Wad_Helu_0_1";
UPDATE tribe:degheim SET sprite = "Ali_Wad_Helu_1_0";
UPDATE tribe:hadendowa SET sprite = "Hadendowa_0_1";
UPDATE tribe:mulazmin SET sprite = "Mulazmin_I_0_0";
UPDATE tribe:jehadia SET sprite = "Jehadia_0_0";
UPDATE tribe:taiasha SET sprite = "Taiasha_0_0";

-- leaf classes (an example counter)
UPDATE unit_class:british_infantry SET sprite = "British_Army_0_1";
UPDATE unit_class:egyptian_infantry SET sprite = "Egyptian_Army_0_1";
UPDATE unit_class:sudanese_infantry SET sprite = "Kitchener_5_0";
UPDATE unit_class:friendlies SET sprite = "Kitchener_0_1";
UPDATE unit_class:ae_cavalry SET sprite = "British_Army_0_0";
UPDATE unit_class:camel_corps SET sprite = "Kitchener_3_0";
UPDATE unit_class:ae_artillery SET sprite = "British_Army_2_0";
UPDATE unit_class:maxim_battery SET sprite = "British_Army_4_0";
UPDATE unit_class:royal_engineers SET sprite = "British_Army_1_0";
UPDATE unit_class:old_gunboat SET sprite = "British_Boats_4_1";
UPDATE unit_class:named_gunboat SET sprite = "British_Boats_4_0";
UPDATE unit_class:d_artillery SET sprite = "Khalifa_Abdullah_0_1";
UPDATE unit_class:d_gunboat SET sprite = "Khalifa_Abdullah_1_0";
UPDATE unit_class:d_camel SET sprite = "Sherif_1_0";
UPDATE unit_class:fort SET sprite = "Hadendowa_Forts_0_0";
