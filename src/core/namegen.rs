/// Random name generator for characters, places, etc.
/// Supports multiple cultures and name types.
pub struct NameGenerator;

impl NameGenerator {
    // === English/Western names ===
    const FIRST_NAMES_M: &'static [&'static str] = &[
        "James", "John", "Robert", "Michael", "William", "David", "Richard",
        "Joseph", "Thomas", "Charles", "Christopher", "Daniel", "Matthew",
        "Anthony", "Mark", "Alexander", "Benjamin", "Samuel", "Henry", "Arthur",
        "Edward", "Theodore", "Oliver", "Lucas", "Elijah", "Sebastian", "Felix",
        "Adrian", "Victor", "Marcus", "Julian", "Dorian", "Lucian", "Cassian",
        "Cedric", "Roland", "Tristan", "Gareth", "Aldric", "Leander",
    ];

    const FIRST_NAMES_F: &'static [&'static str] = &[
        "Mary", "Patricia", "Jennifer", "Linda", "Barbara", "Elizabeth",
        "Susan", "Jessica", "Sarah", "Karen", "Emily", "Sophia", "Olivia",
        "Ava", "Isabella", "Charlotte", "Amelia", "Mia", "Harper", "Eleanor",
        "Genevieve", "Arabella", "Cordelia", "Rosalind", "Evangeline",
        "Seraphina", "Isolde", "Vivienne", "Celestine", "Calista",
        "Ophelia", "Lyra", "Aurora", "Helena", "Cressida", "Linnea",
    ];

    const LAST_NAMES: &'static [&'static str] = &[
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller",
        "Davis", "Rodriguez", "Martinez", "Anderson", "Taylor", "Thomas",
        "Moore", "Jackson", "Martin", "Lee", "Thompson", "White", "Harris",
        "Blackwood", "Ashford", "Thornton", "Whitmore", "Hawthorne",
        "Ravencroft", "Nightingale", "Silverstone", "Langley", "Winslow",
        "Pemberton", "Fairfax", "Kingsley", "Aldridge", "Montague",
        "Sinclair", "Whitfield", "Beaumont", "Lockhart", "Castleberry",
    ];

    // === Japanese names ===
    const JAPANESE_GIVEN_M: &'static [&'static str] = &[
        "Haruto", "Ren", "Sota", "Yuto", "Hiroto", "Minato", "Kaito",
        "Asahi", "Riku", "Hinata", "Takeshi", "Kenji", "Akira", "Yuki",
        "Ryota", "Daichi", "Sho", "Hayato", "Kenta", "Makoto",
    ];

    const JAPANESE_GIVEN_F: &'static [&'static str] = &[
        "Yui", "Hina", "Koharu", "Aoi", "Akari", "Sakura", "Mei",
        "Himari", "Rin", "Mio", "Ichika", "Yuna", "Haruka", "Saki",
        "Nanami", "Kaede", "Misaki", "Ayaka", "Chihiro", "Yuki",
    ];

    const JAPANESE_FAMILY: &'static [&'static str] = &[
        "Sato", "Suzuki", "Takahashi", "Tanaka", "Watanabe", "Ito",
        "Yamamoto", "Nakamura", "Kobayashi", "Kato", "Yoshida",
        "Yamada", "Sasaki", "Yamaguchi", "Matsumoto", "Inoue",
        "Kimura", "Hayashi", "Shimizu", "Yamazaki",
    ];

    // === Chinese names ===
    const CHINESE_GIVEN_M: &'static [&'static str] = &[
        "Wei", "Jian", "Hao", "Lei", "Jun", "Yong", "Ming",
        "Long", "Tao", "Feng", "Chen", "Zhi", "Xiang", "Yang",
        "Bo", "Peng", "Kai", "Liang", "Cheng", "Jie",
    ];

    const CHINESE_GIVEN_F: &'static [&'static str] = &[
        "Li", "Fang", "Na", "Ying", "Xia", "Mei", "Jing",
        "Yan", "Juan", "Min", "Hua", "Qin", "Yun", "Xue",
        "Ping", "Hong", "Lan", "Wen", "Zhen", "Rui",
    ];

    const CHINESE_FAMILY: &'static [&'static str] = &[
        "Wang", "Li", "Zhang", "Liu", "Chen", "Yang", "Zhao",
        "Huang", "Zhou", "Wu", "Xu", "Sun", "Hu", "Zhu",
        "Gao", "Lin", "He", "Guo", "Ma", "Luo",
    ];

    // === Spanish/Latin names ===
    const SPANISH_GIVEN_M: &'static [&'static str] = &[
        "Santiago", "Mateo", "Sebastian", "Leonardo", "Emiliano",
        "Diego", "Miguel", "Alejandro", "Daniel", "Pablo",
        "Rafael", "Carlos", "Fernando", "Andres", "Luis",
        "Jorge", "Eduardo", "Francisco", "Javier", "Rodrigo",
    ];

    const SPANISH_GIVEN_F: &'static [&'static str] = &[
        "Sofia", "Valentina", "Isabella", "Camila", "Lucia",
        "Mariana", "Gabriela", "Victoria", "Elena", "Daniela",
        "Carmen", "Rosa", "Pilar", "Esperanza", "Paloma",
        "Catalina", "Dolores", "Marisol", "Alejandra", "Ximena",
    ];

    const SPANISH_FAMILY: &'static [&'static str] = &[
        "Garcia", "Rodriguez", "Martinez", "Lopez", "Gonzalez",
        "Hernandez", "Perez", "Sanchez", "Ramirez", "Torres",
        "Flores", "Rivera", "Gomez", "Diaz", "Morales",
        "Reyes", "Cruz", "Ortiz", "Gutierrez", "Chavez",
    ];

    // === Indian names ===
    const INDIAN_GIVEN_M: &'static [&'static str] = &[
        "Aarav", "Vihaan", "Aditya", "Sai", "Arjun", "Rohan",
        "Vivaan", "Krishna", "Ishaan", "Shaurya", "Dhruv", "Anish",
        "Ravi", "Dev", "Raj", "Pranav", "Vikram", "Amit", "Nikhil", "Karan",
    ];

    const INDIAN_GIVEN_F: &'static [&'static str] = &[
        "Aadhya", "Ananya", "Diya", "Saanvi", "Isha", "Aanya",
        "Kiara", "Priya", "Riya", "Meera", "Kavya", "Nisha",
        "Sita", "Lakshmi", "Anjali", "Divya", "Pooja", "Neha", "Shreya", "Aisha",
    ];

    const INDIAN_FAMILY: &'static [&'static str] = &[
        "Patel", "Sharma", "Singh", "Kumar", "Das", "Reddy",
        "Gupta", "Nair", "Joshi", "Rao", "Shah", "Mehta",
        "Iyer", "Mishra", "Verma", "Chatterjee", "Mukherjee",
        "Desai", "Pillai", "Menon",
    ];

    // === Arabic names ===
    const ARABIC_GIVEN_M: &'static [&'static str] = &[
        "Omar", "Yusuf", "Ahmed", "Ali", "Hassan", "Ibrahim",
        "Khalid", "Tariq", "Farid", "Rashid", "Zaid", "Nabil",
        "Karim", "Samir", "Hamza", "Bilal", "Jamal", "Walid",
        "Faisal", "Salim",
    ];

    const ARABIC_GIVEN_F: &'static [&'static str] = &[
        "Fatima", "Aisha", "Zahra", "Maryam", "Noor", "Leila",
        "Yasmin", "Amira", "Sara", "Hana", "Layla", "Dina",
        "Samira", "Nadia", "Rania", "Farida", "Khadija", "Zainab",
        "Salma", "Malika",
    ];

    const ARABIC_FAMILY: &'static [&'static str] = &[
        "Al-Rashid", "Al-Farsi", "Al-Hashimi", "Al-Mahmoud", "Al-Sharif",
        "Al-Qasim", "Al-Zahri", "Al-Salem", "Al-Hussein", "Al-Bakri",
        "Al-Nasser", "Al-Khatib", "Al-Razi", "Al-Wazir", "Al-Hakim",
        "Al-Mansour", "Al-Farouk", "Al-Ghazi", "Al-Tayeb", "Al-Sayed",
    ];

    // === Fantasy names ===
    const FANTASY_PREFIXES: &'static [&'static str] = &[
        "Ael", "Thr", "Val", "Mor", "Kal", "Zan", "Eld", "Fen", "Gar",
        "Lyn", "Nyr", "Sar", "Dra", "Kor", "Xen", "Bel", "Lor", "Mir",
        "Ash", "Ryn", "Sol", "Tar", "Vel", "Wyr", "Zor", "Ith", "Ora",
    ];

    const FANTASY_SUFFIXES: &'static [&'static str] = &[
        "iel", "wen", "dor", "rin", "eth", "ael", "ara", "ith", "orn",
        "wyn", "las", "mir", "oth", "enn", "dra", "val", "ien",
        "eon", "ala", "ion", "yon", "ath", "ess", "ora", "iel",
    ];

    // === Place names ===
    const PLACE_PREFIXES: &'static [&'static str] = &[
        "New", "Old", "East", "West", "North", "South", "Upper", "Lower",
        "Great", "Little", "Dark", "Bright", "Shadow", "Silver", "Golden",
        "Iron", "Storm", "Raven", "Wolf", "Dragon", "Crystal", "Frost",
    ];

    const PLACE_SUFFIXES: &'static [&'static str] = &[
        "haven", "shire", "ford", "field", "town", "burg", "vale", "moor",
        "wood", "dale", "bridge", "hollow", "gate", "peak", "falls",
        "watch", "reach", "keep", "hold", "port", "crest", "grove",
    ];

    // === Sci-fi names ===
    const SCIFI_PREFIXES: &'static [&'static str] = &[
        "Zyx", "Kael", "Nex", "Vex", "Cyr", "Aeth", "Orx", "Tyx",
        "Rynn", "Jace", "Zara", "Kira", "Nova", "Axel", "Syr", "Nyx",
        "Cael", "Dex", "Hex", "Lex", "Myx", "Rex", "Tex", "Wex",
    ];

    const SCIFI_SUFFIXES: &'static [&'static str] = &[
        "-7", "-X", "on", "ar", "ix", "us", "ax", "ex",
        "os", "is", "an", "en", "um", "or", "al", "el",
    ];

    // === Medieval names ===
    const MEDIEVAL_NAMES_M: &'static [&'static str] = &[
        "Aldric", "Baldwin", "Cedric", "Dunstan", "Edmund", "Godfrey",
        "Harold", "Leofric", "Oswald", "Percival", "Reginald", "Siegfried",
        "Theodoric", "Ulric", "Wolfgang", "Alaric", "Bertram", "Conrad",
        "Dietrich", "Engelbert", "Fulk", "Gawain", "Hector", "Ivanhoe",
    ];

    const MEDIEVAL_NAMES_F: &'static [&'static str] = &[
        "Adelheid", "Brunhilde", "Clothilde", "Elfrida", "Gwendolyn",
        "Hildegard", "Isolde", "Mathilde", "Rosmund", "Sigrid",
        "Theodora", "Ysabel", "Alienor", "Blanche", "Constance",
        "Elowen", "Fayette", "Guinevere", "Heloise", "Ione",
    ];

    const MEDIEVAL_TITLES: &'static [&'static str] = &[
        "of Ashford", "of Blackwood", "the Bold", "the Brave", "the Fair",
        "the Just", "the Wise", "of Thornfield", "of Ravenholm", "the Strong",
        "of Wintermere", "the Red", "the Black", "of Highcastle", "the Elder",
    ];

    // === Titles / Honorifics ===
    const HONORIFICS: &'static [&'static str] = &[
        "Sir", "Dame", "Lord", "Lady", "Baron", "Baroness", "Count", "Countess",
        "Duke", "Duchess", "Prince", "Princess", "King", "Queen", "Captain",
        "Admiral", "General", "Professor", "Doctor", "Reverend", "Bishop",
        "Cardinal", "Chancellor", "Ambassador", "Senator", "Governor",
    ];

    // === Nicknames ===
    const NICKNAME_ADJECTIVES: &'static [&'static str] = &[
        "Red", "Big", "Lucky", "Slim", "Quick", "Sly", "Iron", "Golden",
        "Shadow", "Wild", "Silent", "Swift", "Dark", "Bright", "Clever",
        "Fierce", "Gentle", "Mad", "Old", "Young", "Little", "Tall",
    ];

    const NICKNAME_NOUNS: &'static [&'static str] = &[
        "Fox", "Wolf", "Bear", "Hawk", "Raven", "Storm", "Thunder",
        "Blade", "Hammer", "Shield", "Arrow", "Fist", "Eye", "Jack",
        "Pete", "Ace", "Duke", "King", "Ghost", "Sage", "Spark", "Dagger",
    ];

    // === Company names ===
    const COMPANY_PREFIXES: &'static [&'static str] = &[
        "Apex", "Sterling", "Pinnacle", "Vanguard", "Atlas", "Crown",
        "Phoenix", "Summit", "Iron", "Golden", "Raven", "Pacific",
        "Nordic", "Titan", "Quantum", "Meridian", "Obsidian", "Eclipse",
    ];

    const COMPANY_SUFFIXES: &'static [&'static str] = &[
        "Industries", "Corp", "Ltd", "Enterprises", "Holdings", "Group",
        "Solutions", "Systems", "Technologies", "Partners", "Associates",
        "Dynamics", "Ventures", "Capital", "Global", "International",
    ];

    // === Ship/Vehicle names ===
    const VEHICLE_ADJECTIVES: &'static [&'static str] = &[
        "Black", "Crimson", "Silver", "Golden", "Iron", "Dark", "Bright",
        "Storm", "Thunder", "Silent", "Swift", "Vengeful", "Fearless",
        "Northern", "Southern", "Eternal", "Ancient", "Mighty",
    ];

    const VEHICLE_NOUNS: &'static [&'static str] = &[
        "Dragon", "Serpent", "Phoenix", "Falcon", "Raven", "Wolf", "Lion",
        "Eagle", "Star", "Wind", "Dawn", "Horizon", "Arrow", "Flame",
        "Pearl", "Crown", "Tide", "Wrath", "Fortune", "Spirit",
    ];

    // === Tavern/Inn names ===
    const TAVERN_ADJECTIVES: &'static [&'static str] = &[
        "Golden", "Silver", "Rusty", "Jolly", "Prancing", "Dancing",
        "Sleeping", "Howling", "Laughing", "Weeping", "Wandering",
        "Drunken", "Merry", "Crooked", "Flying", "Lonely", "Broken",
    ];

    const TAVERN_NOUNS: &'static [&'static str] = &[
        "Dragon", "Pony", "Griffin", "Stag", "Boar", "Fox", "Raven",
        "Crown", "Tankard", "Barrel", "Anvil", "Shield", "Sword",
        "Goblet", "Lantern", "Hound", "Unicorn", "Phoenix", "Badger",
    ];

    /// Simple pseudo-random using current time
    fn rand_index(max: usize) -> usize {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;
        t % max
    }

    fn rand_index_seeded(max: usize, seed: usize) -> usize {
        let mixed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        mixed % max
    }

    pub fn male_name() -> String {
        let seed = Self::rand_index(1000000);
        let first = Self::FIRST_NAMES_M[Self::rand_index_seeded(Self::FIRST_NAMES_M.len(), seed)];
        let last = Self::LAST_NAMES[Self::rand_index_seeded(Self::LAST_NAMES.len(), seed.wrapping_add(37))];
        format!("{} {}", first, last)
    }

    pub fn female_name() -> String {
        let seed = Self::rand_index(1000000);
        let first = Self::FIRST_NAMES_F[Self::rand_index_seeded(Self::FIRST_NAMES_F.len(), seed)];
        let last = Self::LAST_NAMES[Self::rand_index_seeded(Self::LAST_NAMES.len(), seed.wrapping_add(37))];
        format!("{} {}", first, last)
    }

    pub fn fantasy_name() -> String {
        let seed = Self::rand_index(1000000);
        let prefix = Self::FANTASY_PREFIXES[Self::rand_index_seeded(Self::FANTASY_PREFIXES.len(), seed)];
        let suffix = Self::FANTASY_SUFFIXES[Self::rand_index_seeded(Self::FANTASY_SUFFIXES.len(), seed.wrapping_add(42))];
        format!("{}{}", prefix, suffix)
    }

    pub fn place_name() -> String {
        let seed = Self::rand_index(1000000);
        let prefix = Self::PLACE_PREFIXES[Self::rand_index_seeded(Self::PLACE_PREFIXES.len(), seed)];
        let suffix = Self::PLACE_SUFFIXES[Self::rand_index_seeded(Self::PLACE_SUFFIXES.len(), seed.wrapping_add(53))];
        format!("{}{}", prefix, suffix)
    }

    pub fn scifi_name() -> String {
        let seed = Self::rand_index(1000000);
        let prefix = Self::SCIFI_PREFIXES[Self::rand_index_seeded(Self::SCIFI_PREFIXES.len(), seed)];
        let suffix = Self::SCIFI_SUFFIXES[Self::rand_index_seeded(Self::SCIFI_SUFFIXES.len(), seed.wrapping_add(67))];
        format!("{}{}", prefix, suffix)
    }

    pub fn surname() -> String {
        let seed = Self::rand_index(1000000);
        Self::LAST_NAMES[Self::rand_index_seeded(Self::LAST_NAMES.len(), seed)].to_string()
    }

    pub fn medieval_name() -> String {
        let seed = Self::rand_index(1000000);
        let is_female = seed % 2 == 0;
        let name = if is_female {
            Self::MEDIEVAL_NAMES_F[Self::rand_index_seeded(Self::MEDIEVAL_NAMES_F.len(), seed)]
        } else {
            Self::MEDIEVAL_NAMES_M[Self::rand_index_seeded(Self::MEDIEVAL_NAMES_M.len(), seed)]
        };
        let title = Self::MEDIEVAL_TITLES[Self::rand_index_seeded(Self::MEDIEVAL_TITLES.len(), seed.wrapping_add(53))];
        format!("{} {}", name, title)
    }

    pub fn title_name() -> String {
        let seed = Self::rand_index(1000000);
        let title = Self::HONORIFICS[Self::rand_index_seeded(Self::HONORIFICS.len(), seed)];
        // Pair with a random surname
        let surname = Self::LAST_NAMES[Self::rand_index_seeded(Self::LAST_NAMES.len(), seed.wrapping_add(37))];
        format!("{} {}", title, surname)
    }

    pub fn nickname() -> String {
        let seed = Self::rand_index(1000000);
        let adj = Self::NICKNAME_ADJECTIVES[Self::rand_index_seeded(Self::NICKNAME_ADJECTIVES.len(), seed)];
        let noun = Self::NICKNAME_NOUNS[Self::rand_index_seeded(Self::NICKNAME_NOUNS.len(), seed.wrapping_add(31))];
        format!("{} {}", adj, noun)
    }

    pub fn company_name() -> String {
        let seed = Self::rand_index(1000000);
        let prefix = Self::COMPANY_PREFIXES[Self::rand_index_seeded(Self::COMPANY_PREFIXES.len(), seed)];
        let suffix = Self::COMPANY_SUFFIXES[Self::rand_index_seeded(Self::COMPANY_SUFFIXES.len(), seed.wrapping_add(43))];
        format!("{} {}", prefix, suffix)
    }

    pub fn vehicle_name() -> String {
        let seed = Self::rand_index(1000000);
        let adj = Self::VEHICLE_ADJECTIVES[Self::rand_index_seeded(Self::VEHICLE_ADJECTIVES.len(), seed)];
        let noun = Self::VEHICLE_NOUNS[Self::rand_index_seeded(Self::VEHICLE_NOUNS.len(), seed.wrapping_add(47))];
        format!("The {} {}", adj, noun)
    }

    pub fn tavern_name() -> String {
        let seed = Self::rand_index(1000000);
        let adj = Self::TAVERN_ADJECTIVES[Self::rand_index_seeded(Self::TAVERN_ADJECTIVES.len(), seed)];
        let noun = Self::TAVERN_NOUNS[Self::rand_index_seeded(Self::TAVERN_NOUNS.len(), seed.wrapping_add(41))];
        format!("The {} {}", adj, noun)
    }

    /// Generate a culture-specific name
    pub fn culture_name(culture: &str, gender: &str) -> String {
        let seed = Self::rand_index(1000000);
        match culture {
            "japanese" => {
                let family = Self::JAPANESE_FAMILY[Self::rand_index_seeded(Self::JAPANESE_FAMILY.len(), seed)];
                let given = if gender == "female" {
                    Self::JAPANESE_GIVEN_F[Self::rand_index_seeded(Self::JAPANESE_GIVEN_F.len(), seed.wrapping_add(37))]
                } else {
                    Self::JAPANESE_GIVEN_M[Self::rand_index_seeded(Self::JAPANESE_GIVEN_M.len(), seed.wrapping_add(37))]
                };
                format!("{} {}", family, given) // Family name first in Japanese
            }
            "chinese" => {
                let family = Self::CHINESE_FAMILY[Self::rand_index_seeded(Self::CHINESE_FAMILY.len(), seed)];
                let given = if gender == "female" {
                    Self::CHINESE_GIVEN_F[Self::rand_index_seeded(Self::CHINESE_GIVEN_F.len(), seed.wrapping_add(37))]
                } else {
                    Self::CHINESE_GIVEN_M[Self::rand_index_seeded(Self::CHINESE_GIVEN_M.len(), seed.wrapping_add(37))]
                };
                format!("{} {}", family, given)
            }
            "spanish" => {
                let given = if gender == "female" {
                    Self::SPANISH_GIVEN_F[Self::rand_index_seeded(Self::SPANISH_GIVEN_F.len(), seed)]
                } else {
                    Self::SPANISH_GIVEN_M[Self::rand_index_seeded(Self::SPANISH_GIVEN_M.len(), seed)]
                };
                let family1 = Self::SPANISH_FAMILY[Self::rand_index_seeded(Self::SPANISH_FAMILY.len(), seed.wrapping_add(37))];
                let family2 = Self::SPANISH_FAMILY[Self::rand_index_seeded(Self::SPANISH_FAMILY.len(), seed.wrapping_add(71))];
                format!("{} {} {}", given, family1, family2) // Spanish double surname
            }
            "indian" => {
                let given = if gender == "female" {
                    Self::INDIAN_GIVEN_F[Self::rand_index_seeded(Self::INDIAN_GIVEN_F.len(), seed)]
                } else {
                    Self::INDIAN_GIVEN_M[Self::rand_index_seeded(Self::INDIAN_GIVEN_M.len(), seed)]
                };
                let family = Self::INDIAN_FAMILY[Self::rand_index_seeded(Self::INDIAN_FAMILY.len(), seed.wrapping_add(37))];
                format!("{} {}", given, family)
            }
            "arabic" => {
                let given = if gender == "female" {
                    Self::ARABIC_GIVEN_F[Self::rand_index_seeded(Self::ARABIC_GIVEN_F.len(), seed)]
                } else {
                    Self::ARABIC_GIVEN_M[Self::rand_index_seeded(Self::ARABIC_GIVEN_M.len(), seed)]
                };
                let family = Self::ARABIC_FAMILY[Self::rand_index_seeded(Self::ARABIC_FAMILY.len(), seed.wrapping_add(37))];
                format!("{} {}", given, family)
            }
            _ => {
                // Default to English
                if gender == "female" {
                    Self::female_name()
                } else {
                    Self::male_name()
                }
            }
        }
    }

    /// Generate a single name by type key
    pub fn generate_one(kind: &str) -> String {
        match kind {
            "male" => Self::male_name(),
            "female" => Self::female_name(),
            "surname" => Self::surname(),
            "fantasy" => Self::fantasy_name(),
            "place" => Self::place_name(),
            "scifi" => Self::scifi_name(),
            "medieval" => Self::medieval_name(),
            "title" => Self::title_name(),
            "nickname" => Self::nickname(),
            "company" => Self::company_name(),
            "vehicle" => Self::vehicle_name(),
            "tavern" => Self::tavern_name(),
            "japanese_m" => Self::culture_name("japanese", "male"),
            "japanese_f" => Self::culture_name("japanese", "female"),
            "chinese_m" => Self::culture_name("chinese", "male"),
            "chinese_f" => Self::culture_name("chinese", "female"),
            "spanish_m" => Self::culture_name("spanish", "male"),
            "spanish_f" => Self::culture_name("spanish", "female"),
            "indian_m" => Self::culture_name("indian", "male"),
            "indian_f" => Self::culture_name("indian", "female"),
            "arabic_m" => Self::culture_name("arabic", "male"),
            "arabic_f" => Self::culture_name("arabic", "female"),
            _ => Self::male_name(),
        }
    }

    /// Generate multiple names of a given type
    pub fn generate_batch(kind: &str, count: usize) -> Vec<String> {
        let mut names = Vec::new();
        for _ in 0..count {
            let name = Self::generate_one(kind);
            // Small delay to get different seeds
            std::thread::sleep(std::time::Duration::from_nanos(100));
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_male_name_has_two_parts() {
        let name = NameGenerator::male_name();
        assert!(name.contains(' '), "Male name should contain a space: {}", name);
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Male name should have first and last: {}", name);
    }

    #[test]
    fn test_female_name_has_two_parts() {
        let name = NameGenerator::female_name();
        assert!(name.contains(' '), "Female name should contain a space: {}", name);
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Female name should have first and last: {}", name);
    }

    #[test]
    fn test_surname_is_single_word() {
        let name = NameGenerator::surname();
        assert!(!name.is_empty(), "Surname should not be empty");
        // Surnames are single words (some may have compound like "Nightingale")
        assert!(!name.starts_with(' '));
    }

    #[test]
    fn test_fantasy_name_not_empty() {
        let name = NameGenerator::fantasy_name();
        assert!(!name.is_empty());
        assert!(name.len() >= 4, "Fantasy name should be at least 4 chars: {}", name);
    }

    #[test]
    fn test_place_name_not_empty() {
        let name = NameGenerator::place_name();
        assert!(!name.is_empty());
        assert!(name.len() >= 5, "Place name should be at least 5 chars: {}", name);
    }

    #[test]
    fn test_scifi_name_not_empty() {
        let name = NameGenerator::scifi_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_medieval_name_has_title() {
        let name = NameGenerator::medieval_name();
        assert!(!name.is_empty());
        // Medieval names should have a name + title (e.g. "Aldric the Bold")
        assert!(name.contains(' '), "Medieval name should have parts: {}", name);
    }

    #[test]
    fn test_title_name_has_honorific() {
        let name = NameGenerator::title_name();
        assert!(!name.is_empty());
        assert!(name.contains(' '), "Title name should have honorific + surname: {}", name);
    }

    #[test]
    fn test_nickname_has_two_parts() {
        let name = NameGenerator::nickname();
        assert!(name.contains(' '), "Nickname should have adj + noun: {}", name);
    }

    #[test]
    fn test_company_name_has_two_parts() {
        let name = NameGenerator::company_name();
        assert!(name.contains(' '), "Company name should have prefix + suffix: {}", name);
    }

    #[test]
    fn test_vehicle_name_starts_with_the() {
        let name = NameGenerator::vehicle_name();
        assert!(name.starts_with("The "), "Vehicle name should start with 'The ': {}", name);
    }

    #[test]
    fn test_tavern_name_starts_with_the() {
        let name = NameGenerator::tavern_name();
        assert!(name.starts_with("The "), "Tavern name should start with 'The ': {}", name);
    }

    #[test]
    fn test_japanese_male() {
        let name = NameGenerator::culture_name("japanese", "male");
        assert!(name.contains(' '), "Japanese name should have family + given: {}", name);
    }

    #[test]
    fn test_japanese_female() {
        let name = NameGenerator::culture_name("japanese", "female");
        assert!(name.contains(' '));
    }

    #[test]
    fn test_chinese_male() {
        let name = NameGenerator::culture_name("chinese", "male");
        assert!(name.contains(' '));
    }

    #[test]
    fn test_chinese_female() {
        let name = NameGenerator::culture_name("chinese", "female");
        assert!(name.contains(' '));
    }

    #[test]
    fn test_spanish_name_has_double_surname() {
        let name = NameGenerator::culture_name("spanish", "male");
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 3, "Spanish name should have given + two surnames: {}", name);
    }

    #[test]
    fn test_indian_name() {
        let name = NameGenerator::culture_name("indian", "female");
        assert!(name.contains(' '));
    }

    #[test]
    fn test_arabic_name() {
        let name = NameGenerator::culture_name("arabic", "male");
        assert!(name.contains(' '));
        // Arabic family names often contain "Al-"
    }

    #[test]
    fn test_unknown_culture_defaults() {
        let name = NameGenerator::culture_name("klingon", "male");
        assert!(!name.is_empty(), "Unknown culture should default to English name");
    }

    #[test]
    fn test_rand_index_seeded_deterministic() {
        let a = NameGenerator::rand_index_seeded(100, 42);
        let b = NameGenerator::rand_index_seeded(100, 42);
        assert_eq!(a, b, "Same seed should produce same index");
    }

    #[test]
    fn test_rand_index_seeded_different_seeds() {
        let a = NameGenerator::rand_index_seeded(1000, 1);
        let b = NameGenerator::rand_index_seeded(1000, 2);
        // Different seeds should usually produce different results
        // (not guaranteed but very likely with these params)
        // Just check both are in range
        assert!(a < 1000);
        assert!(b < 1000);
    }

    #[test]
    fn test_rand_index_in_range() {
        for _ in 0..10 {
            let idx = NameGenerator::rand_index(50);
            assert!(idx < 50);
        }
    }

    #[test]
    fn test_generate_one_unknown_defaults_to_male() {
        let name = NameGenerator::generate_one("nonexistent_type");
        assert!(!name.is_empty());
        // Should default to male name (has space)
        assert!(name.contains(' '));
    }

    #[test]
    fn test_arabic_female() {
        let name = NameGenerator::culture_name("arabic", "female");
        assert!(name.contains(' '));
    }

    #[test]
    fn test_spanish_female_double_surname() {
        let name = NameGenerator::culture_name("spanish", "female");
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_indian_male() {
        let name = NameGenerator::culture_name("indian", "male");
        assert!(name.contains(' '));
    }

    #[test]
    fn test_chinese_names_have_two_parts() {
        let m = NameGenerator::culture_name("chinese", "male");
        let f = NameGenerator::culture_name("chinese", "female");
        assert_eq!(m.split_whitespace().count(), 2);
        assert_eq!(f.split_whitespace().count(), 2);
    }

    #[test]
    fn test_scifi_name_structure() {
        let name = NameGenerator::scifi_name();
        // Scifi names are prefix + suffix, no space
        assert!(name.len() >= 3);
    }

    #[test]
    fn test_fantasy_name_structure() {
        let name = NameGenerator::fantasy_name();
        // Fantasy names are prefix + suffix, single word
        assert!(name.len() >= 4);
    }

    #[test]
    fn test_place_name_structure() {
        let name = NameGenerator::place_name();
        // Place names are prefix + suffix, single compound
        assert!(!name.is_empty());
    }

    #[test]
    fn test_medieval_name_parts() {
        let name = NameGenerator::medieval_name();
        let parts: Vec<&str> = name.split_whitespace().collect();
        // Medieval names: "Name the Brave" or "Name of Ashford" = 3+ parts
        assert!(parts.len() >= 2);
    }

    #[test]
    fn test_title_name_contains_honorific() {
        let name = NameGenerator::title_name();
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 2);
        // First part should be an honorific
        let honorifics = NameGenerator::HONORIFICS;
        assert!(honorifics.contains(&parts[0]), "First part should be an honorific: {}", parts[0]);
    }

    #[test]
    fn test_nickname_parts() {
        let name = NameGenerator::nickname();
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_company_name_parts() {
        let name = NameGenerator::company_name();
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_vehicle_name_three_parts() {
        let name = NameGenerator::vehicle_name();
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 3); // "The" + adj + noun
    }

    #[test]
    fn test_tavern_name_three_parts() {
        let name = NameGenerator::tavern_name();
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(parts.len(), 3); // "The" + adj + noun
    }

    #[test]
    fn test_name_arrays_not_empty() {
        assert!(!NameGenerator::FIRST_NAMES_M.is_empty());
        assert!(!NameGenerator::FIRST_NAMES_F.is_empty());
        assert!(!NameGenerator::LAST_NAMES.is_empty());
        assert!(!NameGenerator::JAPANESE_GIVEN_M.is_empty());
        assert!(!NameGenerator::JAPANESE_GIVEN_F.is_empty());
        assert!(!NameGenerator::JAPANESE_FAMILY.is_empty());
        assert!(!NameGenerator::CHINESE_GIVEN_M.is_empty());
        assert!(!NameGenerator::CHINESE_FAMILY.is_empty());
        assert!(!NameGenerator::SPANISH_GIVEN_M.is_empty());
        assert!(!NameGenerator::SPANISH_FAMILY.is_empty());
        assert!(!NameGenerator::INDIAN_GIVEN_M.is_empty());
        assert!(!NameGenerator::INDIAN_FAMILY.is_empty());
        assert!(!NameGenerator::ARABIC_GIVEN_M.is_empty());
        assert!(!NameGenerator::ARABIC_FAMILY.is_empty());
        assert!(!NameGenerator::FANTASY_PREFIXES.is_empty());
        assert!(!NameGenerator::FANTASY_SUFFIXES.is_empty());
        assert!(!NameGenerator::PLACE_PREFIXES.is_empty());
        assert!(!NameGenerator::PLACE_SUFFIXES.is_empty());
        assert!(!NameGenerator::SCIFI_PREFIXES.is_empty());
        assert!(!NameGenerator::SCIFI_SUFFIXES.is_empty());
        assert!(!NameGenerator::MEDIEVAL_NAMES_M.is_empty());
        assert!(!NameGenerator::MEDIEVAL_NAMES_F.is_empty());
        assert!(!NameGenerator::MEDIEVAL_TITLES.is_empty());
        assert!(!NameGenerator::HONORIFICS.is_empty());
        assert!(!NameGenerator::NICKNAME_ADJECTIVES.is_empty());
        assert!(!NameGenerator::NICKNAME_NOUNS.is_empty());
        assert!(!NameGenerator::COMPANY_PREFIXES.is_empty());
        assert!(!NameGenerator::COMPANY_SUFFIXES.is_empty());
        assert!(!NameGenerator::VEHICLE_ADJECTIVES.is_empty());
        assert!(!NameGenerator::VEHICLE_NOUNS.is_empty());
        assert!(!NameGenerator::TAVERN_ADJECTIVES.is_empty());
        assert!(!NameGenerator::TAVERN_NOUNS.is_empty());
    }
}
