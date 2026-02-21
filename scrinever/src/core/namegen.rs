/// Random name generator for characters, places, etc.
pub struct NameGenerator;

impl NameGenerator {
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

    const FANTASY_PREFIXES: &'static [&'static str] = &[
        "Ael", "Thr", "Val", "Mor", "Kal", "Zan", "Eld", "Fen", "Gar",
        "Lyn", "Nyr", "Sar", "Dra", "Kor", "Xen", "Bel", "Lor", "Mir",
        "Ash", "Ryn", "Sol", "Tar", "Vel", "Wyr", "Zor", "Ith", "Ora",
    ];

    const FANTASY_SUFFIXES: &'static [&'static str] = &[
        "iel", "wen", "dor", "rin", "eth", "ael", "ara", "ith", "orn",
        "wyn", "las", "mir", "oth", "enn", "iel", "dra", "val", "ien",
        "eon", "ala", "ion", "iel", "yon", "ath", "ess", "ora", "iel",
    ];

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

    /// Simple pseudo-random using current time
    fn rand_index(max: usize) -> usize {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;
        t % max
    }

    fn rand_index_seeded(max: usize, seed: usize) -> usize {
        // Simple hash-based offset
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

    /// Generate multiple names of a given type
    pub fn generate_batch(kind: &str, count: usize) -> Vec<String> {
        let mut names = Vec::new();
        for _ in 0..count {
            let name = match kind {
                "male" => Self::male_name(),
                "female" => Self::female_name(),
                "fantasy" => Self::fantasy_name(),
                "place" => Self::place_name(),
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
}
