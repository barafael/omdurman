//! The phrase banks of the game's press: every wording a telegram or the
//! newspaper may use, by the fact it states. Each bank holds interchangeable
//! wordings; [`super::pick`] chooses one deterministically. Placeholders in
//! braces are filled from the turn record and the game state:
//!
//! * `{n}` a number in words, `{place}` "near Kerreri" / "three miles north
//!   of Omdurman", `{units}` / `{tribes}` / `{leader}` names,
//! * `{date}` "September 2", `{time}` "6 a.m.", `{city}` Omdurman/Khartoum,
//! * `{theirs}` / `{ours}` counts of enemy / own losses in words.
//!
//! Telegrams are written in ordinary case and keyed into telegraphese
//! afterwards ([`super::telegram::telegraphese`]): capitals, STOP, FULL STOP.
//! The newspaper is Victorian prose. Edit freely: a recompile picks it up.

use super::Bank;

/// The wordings of a field telegram, one bank per kind of fact.
pub struct TelegramBanks {
    pub gordon_killed: Bank,
    pub leader_lost: Bank,
    pub khalifa_killed: Bank,
    pub emir_killed: Bank,
    pub tomb_taken: Bank,
    /// Our guns at the enemy's wall (Omdurman) / the enemy's at ours
    /// (Khartoum).
    pub we_breached: Bank,
    pub we_failed_breach: Bank,
    pub enemy_breached: Bank,
    pub enemy_failed_breach: Bank,
    pub gunboat_sunk: Bank,
    pub melee: Bank,
    pub our_losses: Bank,
    pub enemy_losses: Bank,
    pub enemy_shaken: Bank,
    pub desertion: Bank,
    pub our_arrivals: Bank,
    pub enemy_arrivals: Bank,
    pub shelling: Bank,
    pub enemy_retired: Bank,
    pub enemy_body: Bank,
    pub our_position: Bank,
    pub in_contact: Bank,
    pub enemy_distance: Bank,
    pub tomb_enemy: Bank,
    pub tomb_ours: Bank,
    pub score: Bank,
    pub gordon_holds: Bank,
    pub night: Bank,
    pub quiet: Bank,
    /// The Zariba (thorn hedge and trench) built round the camp (§5.3).
    pub zariba_begun: Bank,
    pub zariba_complete: Bank,
}

pub static TELEGRAM: TelegramBanks = TelegramBanks {
    gordon_killed: &[
        "Gordon killed at the Palace",
        "General Gordon fallen at his post",
        "Palace entered Gordon killed",
    ],
    leader_lost: &[
        "Regret {leader} killed {place}",
        "{leader} fallen {place}",
        "Deeply regret loss of {leader} {place}",
    ],
    khalifa_killed: &[
        "Khalifa Abdullah slain {place}",
        "Khalifa killed {place}",
        "Khalifa among enemy dead {place}",
    ],
    emir_killed: &[
        "Emir {leader} slain {place}",
        "Enemy leader {leader} killed {place}",
    ],
    tomb_taken: &[
        "Mahdi's tomb in our hands",
        "Our troops hold the Mahdi's tomb",
        "Tomb of the Mahdi taken",
    ],
    we_breached: &[
        "Our guns breached city wall {place}",
        "Breach made in enemy wall {place}",
    ],
    we_failed_breach: &[
        "Our guns played on city wall without effect",
        "Bombardment of enemy wall failed",
    ],
    enemy_breached: &[
        "Enemy breached our wall {place}",
        "Enemy guns opened breach in wall {place}",
    ],
    enemy_failed_breach: &[
        "Enemy guns played on wall without effect",
        "Wall holds under enemy fire",
    ],
    // {units} carries the kind: "Gunboat Sultan", "Steamer Bordein" (1885).
    gunboat_sunk: &["Regret {units} sunk", "{units} lost"],
    melee: &[
        "Hand to hand fighting {place}",
        "Spear and bayonet met {place}",
        "Sharp fighting at close quarters {place}",
    ],
    our_losses: &[
        "Regret loss {units} {place}",
        "Our losses {units} {place}",
        "{units} cut up {place}",
    ],
    enemy_losses: &[
        "{n} enemy bands destroyed {place}",
        "Enemy lost {n} bands {place}",
        "{tribes} broken {place}",
        "Fire destroyed {n} enemy bands {place}",
    ],
    enemy_shaken: &[
        "{n} enemy bands shaken by fire",
        "Our fire threw {n} enemy bands into disorder",
    ],
    desertion: &[
        "{n} enemy bands deserted in night",
        "Desertions in enemy camp {n} bands gone",
    ],
    our_arrivals: &[
        "{n} of ours came up including {units}",
        "Reinforcements arrived {n} units including {units}",
        "{units} joined force {n} units in all",
    ],
    enemy_arrivals: &[
        "Fresh enemy forces {tribes} reported {place}",
        "{n} enemy bands came up {place}",
        "Enemy reinforced {tribes}",
    ],
    shelling: &[
        "Howitzers shelled enemy {place}",
        "Lyddite shells fell {place}",
    ],
    enemy_retired: &[
        "Enemy horse fell back",
        "Enemy cavalry withdrew before charge",
    ],
    enemy_body: &[
        "Enemy main body {place}",
        "Dervish masses {place}",
        "Enemy concentrated {place}",
    ],
    our_position: &[
        "Our force {place}",
        "Our main body {place}",
        "Our columns {place}",
    ],
    in_contact: &["In contact with enemy", "Enemy close on our front"],
    enemy_distance: &["Enemy {n} miles distant", "Nearest enemy {n} miles"],
    tomb_enemy: &["Tomb still in enemy hands", "Mahdi's tomb held by enemy"],
    tomb_ours: &["Tomb secure"],
    score: &[
        "Points stand {ours} to {theirs}",
        "Score {ours} against {theirs}",
    ],
    gordon_holds: &[
        "Gordon holds Palace",
        "Gordon safe at Palace",
        "Palace holds out",
    ],
    night: &["Night falling", "Darkness coming on"],
    quiet: &["All quiet", "Nothing to report from the front"],
    zariba_begun: &[
        "Zariba begun at Egeiga",
        "Men cutting thorn for zariba at Egeiga",
    ],
    zariba_complete: &[
        "Zariba complete camp entrenched at Egeiga",
        "Camp at Egeiga now enclosed by zariba",
    ],
};

/// The building blocks of the newspaper's lead article.
pub struct LeadBanks {
    /// The opening paragraph, by outcome.
    pub opening_won: Bank,
    pub opening_drawn: Bank,
    pub opening_lost: Bank,
    /// A draw by the reckoning of victory levels, with lopsided losses.
    pub drawn_but_ahead: Bank,
    pub drawn_but_behind: Bank,
    pub opening_gordon_saved: Bank,
    pub opening_gordon_fell_avenged: Bank,
    pub opening_gordon_fell: Bank,
    /// The course of the battle.
    pub first_contact: Bank,
    pub no_contact: Bank,
    pub bloodiest: Bank,
    pub melees: Bank,
    /// The Zariba round the camp at Egeiga (§5.3): finished / begun.
    pub zariba_complete: Bank,
    pub zariba_begun: Bank,
    pub we_breached: Bank,
    pub enemy_breached: Bank,
    pub emirs_killed: Bank,
    pub tomb_taken: Bank,
    pub khalifa_killed: Bank,
    pub gordon_fell: Bank,
    pub desertion: Bank,
    pub gunboats: Bank,
    /// Losses in words.
    pub enemy_losses_heavy: Bank,
    pub enemy_losses_moderate: Bank,
    pub enemy_losses_light: Bank,
    pub our_losses_heavy: Bank,
    pub our_losses_moderate: Bank,
    pub our_losses_light: Bank,
    pub our_losses_none: Bank,
    pub leaders_lost: Bank,
    /// The closing paragraph, by outcome.
    pub closing_won: Bank,
    pub closing_drawn: Bank,
    pub closing_lost: Bank,
    pub closing_gordon_saved: Bank,
    pub closing_gordon_fell: Bank,
    /// The third deck under the headline.
    pub deck_losses: Bank,
}

pub static LEAD: LeadBanks = LeadBanks {
    opening_won: &[
        "The Sirdar's army has won a great victory before {city}. After {n} hours of fighting \
         the Anglo-Egyptian force stands master of the field, and the power of the Khalifa is \
         broken.",
        "News reached us late last night of a signal success for British and Egyptian arms. \
         The Khalifa's host, which had waited so long behind the walls of {city}, has been met \
         in the open and thrown back with great loss.",
    ],
    opening_drawn: &[
        "The long-awaited battle before {city} has been fought, and neither side may fairly \
         claim the day. Both armies held their ground when the fighting ceased.",
        "The action before {city} ended without a decision. The Sirdar's force remains in the \
         field, but the Khalifa's army is not destroyed, and the issue of the campaign hangs in \
         the balance.",
    ],
    opening_lost: &[
        "We regret to announce a grave reverse before {city}. The Khalifa's army has held its \
         ground, and the Anglo-Egyptian force has been compelled to give up the field.",
        "It is with the deepest regret that we record the check sustained by the Sirdar's army \
         before {city}. The Dervish host, so long underrated, has proved itself formidable.",
    ],
    drawn_but_ahead: &[
        "By the strict reckoning of the War Office the result is judged indecisive, although \
         the enemy's losses far exceed our own.",
        "The enemy has suffered far more heavily than we, yet the day is reckoned a drawn \
         one.",
    ],
    drawn_but_behind: &["The result is reckoned a draw, but our own losses have been the heavier."],
    opening_gordon_saved: &[
        "Khartoum stands. Against every expectation the garrison has thrown back the Mahdi's \
         assault, and General Gordon still holds the Palace.",
        "The long anxiety of the nation is relieved: General Gordon is safe, and the Mahdi's \
         host has broken itself against the defences of Khartoum.",
    ],
    opening_gordon_fell_avenged: &[
        "General Gordon is dead. The Mahdi's warriors reached the Palace at last, but they \
         have paid for it so dearly that their army is spent.",
        "Khartoum has fallen and General Gordon with it; yet the besiegers have been bled so \
         white that the victory is theirs in name alone.",
    ],
    opening_gordon_fell: &[
        "Khartoum has fallen. The Mahdi's host forced its way into the city, and General \
         Gordon was killed at the Palace, at his post to the last.",
        "The worst is confirmed. Khartoum is in the hands of the Mahdi, and General Gordon has \
         perished in its defence.",
    ],
    first_contact: &[
        "The first shots were exchanged at {time} on {date}, {place}.",
        "Fighting began at {time} on {date}, when the two forces met {place}.",
        "The armies came into contact {place} at {time} on {date}.",
    ],
    no_contact: &[
        "The two armies manoeuvred at a distance throughout, and no serious engagement took \
         place.",
    ],
    bloodiest: &[
        "The heaviest fighting fell at {time} on {date}, when {n} bands were destroyed {place}.",
        "The crisis came at {time} on {date}: {n} bands perished {place} in a single turn of \
         the battle.",
    ],
    melees: &[
        "On {n} occasions the fighting came to hand-to-hand.",
        "The bayonet and the spear met at close quarters {n} times.",
    ],
    zariba_complete: &[
        "By {time} on {date} the army lay at Egeiga behind a completed zariba of thorn and trench.",
        "The camp at Egeiga was enclosed by its zariba at {time} on {date}.",
    ],
    zariba_begun: &[
        "A zariba was begun about the camp at Egeiga at {time} on {date}.",
        "The troops set to cutting thorn for a zariba at Egeiga at {time} on {date}.",
    ],
    we_breached: &[
        "Our guns opened a breach in the city wall {place}.",
        "A breach was made in the enemy's wall {place}.",
    ],
    enemy_breached: &[
        "The enemy's guns opened a breach in the wall {place}.",
        "The Mahdi's gunners made a breach in the wall {place}.",
    ],
    emirs_killed: &[
        "The emirs {units} are among the slain.",
        "Of the enemy's leaders, {units} fell.",
    ],
    tomb_taken: &[
        "At {time} on {date} our troops entered the Mahdi's Tomb.",
        "The Mahdi's Tomb passed into our hands at {time} on {date}.",
    ],
    khalifa_killed: &[
        "The Khalifa Abdullah himself fell {place}.",
        "Among the enemy dead is the Khalifa Abdullah, killed {place}.",
    ],
    gordon_fell: &[
        "The Palace was entered at {time} on {date}, and General Gordon fell there.",
        "At {time} on {date} the enemy reached the Palace, where General Gordon was killed.",
    ],
    desertion: &["During the night {n} of the enemy's bands deserted his camp."],
    gunboats: &[
        "The gunboat flotilla lost {units}.",
        "On the river the flotilla mourns the loss of {units}.",
    ],
    enemy_losses_heavy: &[
        "The losses of the Dervish are enormous: {n} of their bands were destroyed.",
        "The enemy's loss is terrible. {n} bands were destroyed on the field.",
    ],
    enemy_losses_moderate: &[
        "The enemy lost {n} bands, a heavy toll though not a fatal one.",
        "{n} of the enemy's bands were destroyed.",
    ],
    enemy_losses_light: &[
        "The enemy's losses were light: {n} bands in all.",
        "Few of the enemy fell; {n} bands are reported destroyed.",
    ],
    our_losses_heavy: &[
        "Our own losses are severe. {n} of our units were lost, among them {units}.",
        "The day has been dearly bought: {n} units were lost, among them {units}.",
    ],
    our_losses_moderate: &[
        "Our losses are {n} units, among them {units}.",
        "The force lost {n} units, among them {units}.",
    ],
    our_losses_light: &[
        "Our losses were happily light: {units}.",
        "Our casualties were few: {units}.",
    ],
    our_losses_none: &[
        "Our own force lost not a single unit.",
        "Not one of our units was lost.",
    ],
    leaders_lost: &[
        "The army mourns {units}.",
        "Among the fallen we deeply regret to name {units}.",
    ],
    closing_won: &[
        "The nation will learn with pride of the conduct of the troops, and the name of Gordon \
         is avenged.",
        "Thirteen years after the fall of Khartoum the debt is paid; the Soudan lies open to the \
         Sirdar.",
    ],
    closing_drawn: &[
        "Another season may be needed to complete the work begun before {city}.",
        "The country must wait for the next despatches to know whether the campaign is won.",
    ],
    closing_lost: &[
        "It is too early to measure the consequences of this reverse; but the country will \
         demand an explanation.",
        "The Government must now decide whether the Soudan is worth another army.",
    ],
    closing_gordon_saved: &[
        "The relief column may now arrive in time, and General Gordon's name will be honoured \
         wherever the English tongue is spoken.",
    ],
    closing_gordon_fell: &[
        "The nation mourns a hero. It will not soon forget the name of Charles Gordon, nor \
         forgive those who sent help too late.",
    ],
    deck_losses: &[
        "{theirs} Dervish Bands Destroyed \u{2014} {ours} of Ours Lost",
        "The Enemy Loses {theirs} Bands; Our Loss {ours}",
    ],
};

/// A short article on another subject, one paragraph per bank (one wording
/// picked from each).
pub struct Feature {
    pub head: &'static str,
    pub paragraphs: &'static [Bank],
}

/// The other news of September 1898.
pub static FEATURES_1898: &[Feature] = &[
    Feature {
        head: "THE FRENCH ON THE UPPER NILE",
        paragraphs: &[
            &[
                "Reports from Cairo speak of a French expedition under Major Marchand which is \
                 said to have reached the Nile at Fashoda.",
                "Persistent rumours place a small French force at Fashoda, some four hundred \
                 miles above Khartoum.",
            ],
            &[
                "Should the rumour be confirmed, the Sirdar's next march will be watched with \
                 the closest attention in Paris and in London alike.",
            ],
        ],
    },
    Feature {
        head: "THE TSAR'S PROPOSAL",
        paragraphs: &[&[
            "The Russian circular inviting the Powers to a conference on the limitation of \
             armaments continues to occupy the chancelleries of Europe.",
            "Opinion at Berlin and Vienna on the Tsar's proposal for a conference on armaments \
             remains reserved.",
        ]],
    },
    Feature {
        head: "THE DREYFUS AFFAIR",
        paragraphs: &[&[
            "The confession and death of Colonel Henry have revived the agitation for a \
             revision of the Dreyfus case, and Paris is greatly excited.",
            "After the discovery of the Henry forgery, the demand for a revision of the trial \
             of Captain Dreyfus grows louder by the day.",
        ]],
    },
    Feature {
        head: "COURT CIRCULAR",
        paragraphs: &[&[
            "Balmoral, Friday.\u{2014}The Queen drove out yesterday afternoon, attended by the \
             Hon. Harriet Phipps.",
            "Balmoral, Friday.\u{2014}Her Majesty walked in the grounds this morning; the weather \
             is fine.",
        ]],
    },
    Feature {
        head: "THE WEATHER",
        paragraphs: &[&[
            "London: fair and warm, light airs from the south-west. Barometer steady.",
            "The heat continues; a thunderstorm is expected in the south of England.",
        ]],
    },
];

/// The other news of January 1885.
pub static FEATURES_1885: &[Feature] = &[
    Feature {
        head: "THE DESERT COLUMN",
        paragraphs: &[&[
            "Sir Herbert Stewart's column, after its gallant action at Abu Klea, has reached \
             the Nile near Metemmeh; Sir Herbert is reported severely wounded.",
            "The Desert Column has fought its way to the river, and steamers sent down by \
             General Gordon are said to be in touch with it.",
        ]],
    },
    Feature {
        head: "OUTRAGES AT WESTMINSTER",
        paragraphs: &[&[
            "The explosions at Westminster Hall and the Tower on Saturday have caused the \
             greatest indignation; several police constables were injured.",
            "The dynamite outrages at the Houses of Parliament and the Tower are attributed to \
             Fenian conspirators. Strong measures are demanded.",
        ]],
    },
    Feature {
        head: "THE CONFERENCE AT BERLIN",
        paragraphs: &[&[
            "The Congo Conference continues its sittings at Berlin, and its labours are said \
             to be nearly concluded.",
        ]],
    },
    Feature {
        head: "COURT CIRCULAR",
        paragraphs: &[&[
            "Osborne, Monday.\u{2014}The Queen walked and drove yesterday, attended by the Hon. \
             Ethel Cadogan.",
        ]],
    },
    Feature {
        head: "THE WEATHER",
        paragraphs: &[&[
            "Sharp frost in London; skating on the Serpentine. Wind north-easterly.",
            "Cold and dull, with snow showers in the north.",
        ]],
    },
];

/// A small advertisement (invented firms of the day).
pub struct Advert {
    pub head: &'static str,
    pub lines: &'static [&'static str],
}

pub static ADVERTS: &[Advert] = &[
    Advert {
        head: "HARGREAVE'S DESERT COCOA",
        lines: &["Sustains the Soldier.", "Of all Grocers, 1s. the tin."],
    },
    Advert {
        head: "DR. MORTIMER'S NILE TONIC",
        lines: &["For Heat, Fatigue & Fever.", "Recommended to Officers."],
    },
    Advert {
        head: "THE PATENT CAMPAIGN KIT",
        lines: &[
            "Folding Bed, Chair & Lamp",
            "in one Portmanteau. 4 Guineas.",
        ],
    },
    Advert {
        head: "PRESTON'S WAR MAPS",
        lines: &[
            "The Seat of War in the Soudan,",
            "coloured, post free 1s. 6d.",
        ],
    },
];

/// The "Late Telegrams" column's heading line for each turn's telegram.
pub static TELEGRAM_HEADS: Bank = &["{date}, {time}", "{date}, {time} (by telegraph)"];
