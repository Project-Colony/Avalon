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

    /// Generate multiple names of a given type
    pub fn generate_batch(kind: &str, count: usize) -> Vec<String> {
        let mut names = Vec::new();
        for _ in 0..count {
            let name = match kind {
                "male" => Self::male_name(),
                "female" => Self::female_name(),
                "fantasy" => Self::fantasy_name(),
                "place" => Self::place_name(),
                "scifi" => Self::scifi_name(),
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
            };
            // Small delay to get different seeds
            std::thread::sleep(std::time::Duration::from_nanos(100));
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }

    /// List all available name generator types
    pub fn available_types() -> Vec<(&'static str, &'static str)> {
        vec![
            ("male", "Male (English)"),
            ("female", "Female (English)"),
            ("surname", "Surname"),
            ("fantasy", "Fantasy"),
            ("place", "Place"),
            ("scifi", "Sci-Fi"),
            ("medieval", "Medieval"),
            ("title", "Title/Honorific"),
            ("nickname", "Nickname"),
            ("company", "Company"),
            ("vehicle", "Ship/Vehicle"),
            ("tavern", "Tavern/Inn"),
            ("japanese_m", "Japanese (M)"),
            ("japanese_f", "Japanese (F)"),
            ("chinese_m", "Chinese (M)"),
            ("chinese_f", "Chinese (F)"),
            ("spanish_m", "Spanish (M)"),
            ("spanish_f", "Spanish (F)"),
            ("indian_m", "Indian (M)"),
            ("indian_f", "Indian (F)"),
            ("arabic_m", "Arabic (M)"),
            ("arabic_f", "Arabic (F)"),
        ]
    }

    /// Total number of name types available
    pub fn type_count() -> usize {
        Self::available_types().len()
    }

    /// Generate a full character name (first + surname)
    pub fn full_name(category: &str) -> String {
        let first_batch = Self::generate_batch(category, 1);
        let surname_batch = Self::generate_batch("surname", 1);
        let first = first_batch.first().cloned().unwrap_or_default();
        let surname = surname_batch.first().cloned().unwrap_or_default();
        format!("{} {}", first, surname)
    }

    /// Generate multiple full character names
    pub fn full_names(category: &str, count: usize) -> Vec<String> {
        let mut names = Vec::new();
        for _ in 0..count {
            let name = Self::full_name(category);
            if !names.contains(&name) {
                names.push(name);
            }
            std::thread::sleep(std::time::Duration::from_nanos(100));
        }
        names
    }

    /// Check if a name type is culture-specific
    pub fn is_culture_type(name_type: &str) -> bool {
        matches!(name_type,
            "japanese_m" | "japanese_f" | "chinese_m" | "chinese_f"
            | "spanish_m" | "spanish_f" | "indian_m" | "indian_f"
            | "arabic_m" | "arabic_f"
        )
    }
}
