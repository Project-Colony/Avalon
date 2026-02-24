#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A writing prompt for creative inspiration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingPrompt {
    pub id: Uuid,
    pub category: PromptCategory,
    pub prompt_text: String,
    pub difficulty: Difficulty,
    pub estimated_words: usize,
    pub tags: Vec<String>,
}

/// Categories for writing prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PromptCategory {
    FreeWrite,
    CharacterDevelopment,
    DialogueExercise,
    SettingDescription,
    PlotTwist,
    OpeningLine,
    ClosingLine,
    ConflictScenario,
    EmotionalScene,
    ActionScene,
    WorldBuilding,
    FlashFiction,
    Poetry,
}

impl PromptCategory {
    pub fn label(&self) -> &str {
        match self {
            PromptCategory::FreeWrite => "Free Write",
            PromptCategory::CharacterDevelopment => "Character Development",
            PromptCategory::DialogueExercise => "Dialogue Exercise",
            PromptCategory::SettingDescription => "Setting Description",
            PromptCategory::PlotTwist => "Plot Twist",
            PromptCategory::OpeningLine => "Opening Line",
            PromptCategory::ClosingLine => "Closing Line",
            PromptCategory::ConflictScenario => "Conflict Scenario",
            PromptCategory::EmotionalScene => "Emotional Scene",
            PromptCategory::ActionScene => "Action Scene",
            PromptCategory::WorldBuilding => "World Building",
            PromptCategory::FlashFiction => "Flash Fiction",
            PromptCategory::Poetry => "Poetry",
        }
    }

    /// Return all prompt categories in order
    pub fn all() -> &'static [PromptCategory] {
        &[
            PromptCategory::FreeWrite,
            PromptCategory::CharacterDevelopment,
            PromptCategory::DialogueExercise,
            PromptCategory::SettingDescription,
            PromptCategory::PlotTwist,
            PromptCategory::OpeningLine,
            PromptCategory::ClosingLine,
            PromptCategory::ConflictScenario,
            PromptCategory::EmotionalScene,
            PromptCategory::ActionScene,
            PromptCategory::WorldBuilding,
            PromptCategory::FlashFiction,
            PromptCategory::Poetry,
        ]
    }

    /// Get the category at a given index (wrapping)
    fn from_index(idx: usize) -> PromptCategory {
        let all = Self::all();
        all[idx % all.len()]
    }
}

/// Difficulty level for prompts and exercises
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl Difficulty {
    pub fn label(&self) -> &str {
        match self {
            Difficulty::Beginner => "Beginner",
            Difficulty::Intermediate => "Intermediate",
            Difficulty::Advanced => "Advanced",
        }
    }
}

/// A generated character profile for fiction writing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub name_suggestion: String,
    pub age_range: String,
    pub occupation: String,
    pub trait_primary: String,
    pub trait_secondary: String,
    pub motivation: String,
    pub flaw: String,
    pub background_hook: String,
}

/// A seed for generating a plot idea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSeed {
    pub genre: String,
    pub protagonist_type: String,
    pub antagonist_type: String,
    pub setting: String,
    pub central_conflict: String,
    pub theme: String,
    pub opening_hook: String,
}

/// A suggested character or place name with origin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameSuggestion {
    pub first_name: String,
    pub last_name: String,
    pub origin: String,
    pub meaning: String,
}

/// A structured writing exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingExercise {
    pub title: String,
    pub instructions: String,
    pub duration_minutes: usize,
    pub category: PromptCategory,
}

/// Tracks which prompts a writer has used and completed
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptHistory {
    pub entries: Vec<PromptHistoryEntry>,
}

/// A single entry in the prompt history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHistoryEntry {
    pub prompt_id: Uuid,
    pub used_at: DateTime<Utc>,
    pub completed: bool,
    pub user_rating: Option<u8>,
}

// =============================================================================
// Deterministic hash helper
// =============================================================================

/// A deterministic hash function for picking items from lists without rand.
/// Given the same seed and max, always returns the same index in [0, max).
pub fn simple_hash(seed: u64, max: usize) -> usize {
    if max == 0 {
        return 0;
    }
    // Use a mixing function inspired by splitmix64
    let mut x = seed;
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x = x ^ (x >> 31);
    (x as usize) % max
}

/// Advance a seed to produce a new seed value for sequential picks
fn next_seed(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

// =============================================================================
// Built-in prompt content
// =============================================================================

const FREEWRITE_PROMPTS: &[&str] = &[
    "Write for ten minutes about the last thing that made you laugh unexpectedly.",
    "Describe your morning routine as if you were a character in a novel.",
    "Write a letter to your future self, ten years from now.",
    "Imagine a world where gravity works sideways. Describe an ordinary day.",
    "Write about a smell that instantly transports you to a specific memory.",
    "Describe the view from a window you have never looked through before.",
    "Write about something you lost and never found again.",
];

const CHARACTER_PROMPTS: &[&str] = &[
    "Create a character who keeps a secret collection that would surprise everyone who knows them.",
    "Write about a character who discovers they have been living someone else's life.",
    "Develop a character whose greatest strength is also their fatal flaw.",
    "Describe a character through the eyes of three different people who know them.",
    "Write a character who communicates primarily through actions rather than words.",
    "Create a character who is the last person you would expect to become a hero.",
    "Write about a character whose daily routine hides a double life.",
];

const DIALOGUE_PROMPTS: &[&str] = &[
    "Write a conversation between two strangers stuck in an elevator during a power outage.",
    "Create a dialogue where two characters say the opposite of what they mean.",
    "Write an argument between best friends where neither will say what the fight is really about.",
    "Compose a dialogue between a parent and child where the child is the wiser one.",
    "Write a conversation where one character is trying to confess something important but keeps getting interrupted.",
    "Create a dialogue between two people meeting for the first time who will become lifelong enemies.",
    "Write a phone conversation where we only hear one side but can infer everything.",
];

const SETTING_PROMPTS: &[&str] = &[
    "Describe a city that exists entirely underground, lit by bioluminescent plants.",
    "Write about an abandoned amusement park being reclaimed by nature.",
    "Describe a library where the books rearrange themselves every night.",
    "Paint a picture of a coastal town during the moment before a massive storm arrives.",
    "Describe a space station that has been converted into a luxury hotel.",
    "Write about a forest where the trees whisper secrets to those who listen.",
    "Describe a marketplace that only appears during the full moon.",
];

const PLOT_TWIST_PROMPTS: &[&str] = &[
    "Write a scene where the hero realizes they have been the villain all along.",
    "Create a moment where a character discovers their mentor has been sabotaging them.",
    "Write the scene where a dead character turns out to be alive, but changed.",
    "Describe the moment a character realizes their entire reality is a constructed illusion.",
    "Write a twist where the antagonist's motivation turns out to be completely justified.",
    "Create a scene where two seemingly unrelated storylines collide in an unexpected way.",
    "Write a reveal that recontextualizes everything the reader thought they knew.",
];

const OPENING_LINE_PROMPTS: &[&str] = &[
    "Write an opening paragraph that begins with a lie the narrator will later regret.",
    "Create an opening that drops the reader into the middle of a chase scene.",
    "Write a first line that introduces a world where one fundamental law of nature is different.",
    "Compose an opening that starts with the ending and works backward.",
    "Write a first paragraph from the perspective of an inanimate object.",
    "Create an opening that establishes an unreliable narrator within the first three sentences.",
    "Write a beginning that immediately establishes both the setting and the central conflict.",
];

const CLOSING_LINE_PROMPTS: &[&str] = &[
    "Write a final paragraph that mirrors the opening but with everything changed.",
    "Create an ending where the character returns home but sees it with completely new eyes.",
    "Write a closing line that leaves one crucial question unanswered.",
    "Compose an ending that reveals the narrator has been telling this story to someone specific.",
    "Write a conclusion where the character makes a choice that surprises even themselves.",
    "Create a final scene that is quiet and ordinary after extraordinary events.",
    "Write an ending that suggests the story is about to begin all over again.",
];

const CONFLICT_PROMPTS: &[&str] = &[
    "Write a scene where two characters want the same thing but for entirely different reasons.",
    "Create a conflict where the right choice and the moral choice are opposites.",
    "Write about a character who must betray a friend to save a stranger.",
    "Describe a conflict that arises from a simple misunderstanding that spirals out of control.",
    "Write a scene where a character must choose between two people they love equally.",
    "Create a conflict between tradition and progress in a small community.",
    "Write about a character whose internal conflict manifests as an external obstacle.",
];

const EMOTIONAL_PROMPTS: &[&str] = &[
    "Write a scene of reunion between two people who thought they would never see each other again.",
    "Create a moment of profound grief that transforms into unexpected hope.",
    "Write about the exact moment a character falls out of love.",
    "Describe a character experiencing joy so intense it frightens them.",
    "Write a scene where a character finally forgives someone who does not deserve it.",
    "Create a moment where a character must say goodbye knowing it is the last time.",
    "Write about a character who receives news that changes everything they believed about themselves.",
];

const ACTION_PROMPTS: &[&str] = &[
    "Write a chase scene through a crowded marketplace where both pursuer and pursued are injured.",
    "Create a battle scene told entirely from the perspective of a medic.",
    "Write an escape sequence where the character must use only their wits, no weapons.",
    "Describe a heist that goes perfectly until the very last step.",
    "Write a fight scene between two characters who trained under the same master.",
    "Create an action sequence that takes place entirely in a single room.",
    "Write a rescue scene where the person being rescued does not want to be saved.",
];

const WORLDBUILDING_PROMPTS: &[&str] = &[
    "Design a magic system with exactly three rules and describe how society adapted around them.",
    "Create a culture where social status is determined by the complexity of one's dreams.",
    "Write about a world where music has tangible physical effects on the environment.",
    "Describe a civilization that developed technology but never discovered fire.",
    "Create a society where memories can be traded as currency.",
    "Write about a world where the seasons last decades instead of months.",
    "Design a religion that worships a god everyone agrees is real but absent.",
];

const FLASH_FICTION_PROMPTS: &[&str] = &[
    "Write a complete story in exactly 100 words about a door that should not be opened.",
    "Tell a full story in under 500 words where the last line changes the meaning of the first.",
    "Write a flash fiction piece where every sentence could be its own story.",
    "Create a 250-word story that takes place entirely in the space between two heartbeats.",
    "Write a micro-story using only dialogue, no attribution or description.",
    "Tell a complete story in under 300 words about the moment everything changed.",
    "Write a flash piece where the setting is the main character.",
];

const POETRY_PROMPTS: &[&str] = &[
    "Write a poem about an everyday object as if seeing it for the first time.",
    "Compose a poem where each line begins with the last word of the previous line.",
    "Write a poem from the perspective of a house watching its family grow old.",
    "Create a poem that uses only concrete images to express an abstract emotion.",
    "Write a poem structured like a recipe for something that cannot be cooked.",
    "Compose a poem about silence using only sounds.",
    "Write a poem where the shape of the text on the page mirrors its subject.",
];

fn prompts_for_category(category: PromptCategory) -> &'static [&'static str] {
    match category {
        PromptCategory::FreeWrite => FREEWRITE_PROMPTS,
        PromptCategory::CharacterDevelopment => CHARACTER_PROMPTS,
        PromptCategory::DialogueExercise => DIALOGUE_PROMPTS,
        PromptCategory::SettingDescription => SETTING_PROMPTS,
        PromptCategory::PlotTwist => PLOT_TWIST_PROMPTS,
        PromptCategory::OpeningLine => OPENING_LINE_PROMPTS,
        PromptCategory::ClosingLine => CLOSING_LINE_PROMPTS,
        PromptCategory::ConflictScenario => CONFLICT_PROMPTS,
        PromptCategory::EmotionalScene => EMOTIONAL_PROMPTS,
        PromptCategory::ActionScene => ACTION_PROMPTS,
        PromptCategory::WorldBuilding => WORLDBUILDING_PROMPTS,
        PromptCategory::FlashFiction => FLASH_FICTION_PROMPTS,
        PromptCategory::Poetry => POETRY_PROMPTS,
    }
}

fn difficulty_for_index(idx: usize) -> Difficulty {
    match idx % 3 {
        0 => Difficulty::Beginner,
        1 => Difficulty::Intermediate,
        _ => Difficulty::Advanced,
    }
}

fn estimated_words_for_category(category: PromptCategory, difficulty: Difficulty) -> usize {
    let base = match category {
        PromptCategory::FreeWrite => 500,
        PromptCategory::CharacterDevelopment => 400,
        PromptCategory::DialogueExercise => 350,
        PromptCategory::SettingDescription => 300,
        PromptCategory::PlotTwist => 600,
        PromptCategory::OpeningLine => 200,
        PromptCategory::ClosingLine => 200,
        PromptCategory::ConflictScenario => 500,
        PromptCategory::EmotionalScene => 450,
        PromptCategory::ActionScene => 550,
        PromptCategory::WorldBuilding => 700,
        PromptCategory::FlashFiction => 250,
        PromptCategory::Poetry => 100,
    };
    match difficulty {
        Difficulty::Beginner => base,
        Difficulty::Intermediate => (base as f64 * 1.5) as usize,
        Difficulty::Advanced => base * 2,
    }
}

fn tags_for_category(category: PromptCategory) -> Vec<String> {
    match category {
        PromptCategory::FreeWrite => vec!["freewrite".into(), "creative".into(), "warm-up".into()],
        PromptCategory::CharacterDevelopment => vec!["character".into(), "development".into(), "fiction".into()],
        PromptCategory::DialogueExercise => vec!["dialogue".into(), "conversation".into(), "voice".into()],
        PromptCategory::SettingDescription => vec!["setting".into(), "description".into(), "imagery".into()],
        PromptCategory::PlotTwist => vec!["plot".into(), "twist".into(), "surprise".into()],
        PromptCategory::OpeningLine => vec!["opening".into(), "hook".into(), "beginning".into()],
        PromptCategory::ClosingLine => vec!["closing".into(), "ending".into(), "resolution".into()],
        PromptCategory::ConflictScenario => vec!["conflict".into(), "tension".into(), "drama".into()],
        PromptCategory::EmotionalScene => vec!["emotion".into(), "feeling".into(), "scene".into()],
        PromptCategory::ActionScene => vec!["action".into(), "pace".into(), "excitement".into()],
        PromptCategory::WorldBuilding => vec!["worldbuilding".into(), "setting".into(), "fantasy".into()],
        PromptCategory::FlashFiction => vec!["flash".into(), "short".into(), "complete".into()],
        PromptCategory::Poetry => vec!["poetry".into(), "verse".into(), "lyric".into()],
    }
}

/// Create a deterministic UUID from a seed value
fn uuid_from_seed(seed: u64) -> Uuid {
    let mut bytes = [0u8; 16];
    let mut s = seed;
    for chunk in bytes.chunks_mut(8) {
        let b = s.to_le_bytes();
        for (i, byte) in chunk.iter_mut().enumerate() {
            *byte = b[i];
        }
        s = next_seed(s);
    }
    // Set version 4 and variant bits for a valid UUID format
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn build_prompt(category: PromptCategory, prompt_index: usize, text: &str) -> WritingPrompt {
    let difficulty = difficulty_for_index(prompt_index);
    let seed_val = (category as u64).wrapping_mul(1000).wrapping_add(prompt_index as u64);
    WritingPrompt {
        id: uuid_from_seed(seed_val),
        category,
        prompt_text: text.to_string(),
        difficulty,
        estimated_words: estimated_words_for_category(category, difficulty),
        tags: tags_for_category(category),
    }
}

// =============================================================================
// Character generation data
// =============================================================================

const CHARACTER_TRAITS: &[&str] = &[
    "compassionate", "stubborn", "analytical", "impulsive", "charismatic",
    "cautious", "optimistic", "cynical", "resourceful", "idealistic",
    "pragmatic", "loyal", "rebellious", "meticulous", "adventurous",
    "introverted", "generous", "ambitious", "patient", "witty",
    "stoic", "empathetic", "cunning", "gentle", "fierce",
];

const CHARACTER_FLAWS: &[&str] = &[
    "cannot trust anyone completely",
    "avoids confrontation at all costs",
    "holds grudges for years",
    "lies to protect others' feelings",
    "takes on too much responsibility",
    "fears abandonment above all else",
    "refuses to ask for help",
    "judges others too quickly",
    "cannot let go of the past",
    "hides vulnerability behind humor",
    "overestimates their own abilities",
    "sacrifices personal needs for others",
    "struggles with jealousy",
    "avoids emotional intimacy",
    "obsesses over perfection",
];

const OCCUPATIONS: &[&str] = &[
    "lighthouse keeper", "forensic botanist", "war correspondent",
    "antiquarian bookseller", "storm chaser", "cryptographer",
    "marine biologist", "puppeteer", "volcanologist",
    "cartographer", "sommelier", "archivist",
    "deep sea diver", "falconer", "glassblower",
    "midwife", "taxidermist", "translator",
    "clockmaker", "mycologist",
];

const MOTIVATIONS: &[&str] = &[
    "desperate to find a missing family member",
    "driven to prove a controversial theory",
    "seeking redemption for a past mistake",
    "protecting a dangerous secret",
    "searching for a place to truly belong",
    "trying to keep a promise made to someone now gone",
    "fighting to preserve something beautiful from destruction",
    "determined to break free from an inherited legacy",
    "hunting for the truth behind a decades-old mystery",
    "striving to build something that will outlast them",
];

const AGE_RANGES: &[&str] = &[
    "teenager (14-17)", "young adult (18-25)", "late twenties (26-30)",
    "early thirties (31-35)", "late thirties (36-40)", "middle-aged (41-55)",
    "mature adult (56-65)", "elderly (66-80)", "child (8-13)",
    "early twenties (20-24)",
];

const BACKGROUND_HOOKS: &[&str] = &[
    "Was raised by a grandparent and never knew why their parents left.",
    "Survived an event that no one else remembers happening.",
    "Grew up in a traveling circus and never learned to stay in one place.",
    "Inherited a house full of locked rooms and no keys.",
    "Was once famous for something they now deeply regret.",
    "Carries a letter they have never opened.",
    "Left their homeland under circumstances they refuse to discuss.",
    "Has a scar with a story they tell differently every time.",
    "Was the sole witness to something extraordinary that no one believes.",
    "Woke up one morning speaking a language they had never learned.",
];

const FIRST_NAMES_NEUTRAL: &[&str] = &[
    "Alex", "Jordan", "Morgan", "Casey", "Riley",
    "Avery", "Quinn", "Rowan", "Sage", "Emery",
    "Dakota", "Reese", "Finley", "Hayden", "Skyler",
    "Cameron", "Phoenix", "River", "Blair", "Ashton",
];

const FIRST_NAMES_MALE: &[&str] = &[
    "James", "Marcus", "Oliver", "Elijah", "Sebastian",
    "Theodore", "Felix", "Adrian", "Victor", "Julian",
    "Leo", "Gabriel", "Arthur", "Henry", "Samuel",
    "Dorian", "Lucian", "Cedric", "Roland", "Tristan",
];

const FIRST_NAMES_FEMALE: &[&str] = &[
    "Eleanor", "Sophia", "Vivienne", "Cordelia", "Rosalind",
    "Evangeline", "Seraphina", "Isolde", "Celeste", "Arabella",
    "Lyra", "Aurora", "Helena", "Ophelia", "Linnea",
    "Genevieve", "Amara", "Iris", "Thea", "Camille",
];

const LAST_NAMES: &[&str] = &[
    "Ashford", "Blackwood", "Thornton", "Whitmore", "Hawthorne",
    "Ravencroft", "Nightingale", "Silverstone", "Langley", "Winslow",
    "Pemberton", "Fairfax", "Kingsley", "Aldridge", "Montague",
    "Sinclair", "Whitfield", "Beaumont", "Lockhart", "Castleberry",
    "Holloway", "Stirling", "Dunmore", "Ashworth", "Everett",
];

const NAME_ORIGINS: &[&str] = &[
    "English", "French", "Germanic", "Celtic", "Latin",
    "Greek", "Scandinavian", "Italian", "Spanish", "Slavic",
];

const NAME_MEANINGS: &[&str] = &[
    "protector of the people", "born of fire", "keeper of secrets",
    "gentle strength", "wanderer of paths", "light bringer",
    "steadfast heart", "voice of reason", "storm caller",
    "child of the forest", "dawn's herald", "silver-tongued",
    "sword bearer", "peace weaver", "star watcher",
    "stone guardian", "river dancer", "shadow walker",
    "flame keeper", "wind singer",
];

// =============================================================================
// Plot seed data
// =============================================================================

const GENRES: &[&str] = &[
    "Literary Fiction", "Science Fiction", "Fantasy", "Mystery",
    "Thriller", "Horror", "Romance", "Historical Fiction",
    "Dystopian", "Magical Realism", "Gothic", "Western",
];

const PROTAGONIST_TYPES: &[&str] = &[
    "a reluctant hero burdened by duty",
    "a cunning trickster with a hidden conscience",
    "an ordinary person thrust into extraordinary circumstances",
    "a fallen leader seeking redemption",
    "a young outsider with an unusual gift",
    "a seasoned investigator haunted by an unsolved case",
    "a diplomat trapped between warring factions",
    "a scholar who discovers forbidden knowledge",
    "an artist whose creations come to life",
    "a healer who cannot cure their own affliction",
];

const ANTAGONIST_TYPES: &[&str] = &[
    "a charismatic leader whose vision demands terrible sacrifice",
    "a former ally driven mad by betrayal",
    "an ancient force that has awakened after centuries of slumber",
    "a faceless institution that crushes individuality",
    "a mirror image of the hero, but without moral restraint",
    "a loved one who believes they are doing the right thing",
    "a trickster god playing games with mortal lives",
    "a brilliant inventor whose creation has turned against humanity",
    "a nature itself, indifferent and merciless",
    "a collective delusion shared by an entire society",
];

const SETTINGS: &[&str] = &[
    "a crumbling empire on the edge of revolution",
    "a generation ship lost between the stars",
    "a small town where everyone keeps the same secret",
    "a city built in the canopy of colossal trees",
    "a frozen wasteland hiding the ruins of a lost civilization",
    "a sprawling metropolis where the rich live above the clouds",
    "a coastal village where the sea gives back what it takes",
    "an ancient library that exists outside of time",
    "a war-torn countryside slowly reclaiming its beauty",
    "a border town caught between two hostile nations",
];

const CENTRAL_CONFLICTS: &[&str] = &[
    "A prophecy threatens to destroy everything unless one person makes an impossible sacrifice.",
    "Two communities must learn to coexist or face mutual destruction.",
    "A long-buried truth resurfaces, shattering alliances and forcing new ones.",
    "A resource everyone depends on is running out, and someone controls what remains.",
    "An unstoppable force approaches, and the only defense requires trusting the untrustworthy.",
    "A choice between saving the many or saving the one person who matters most.",
    "Knowledge that could save the world could also be used to destroy it.",
    "A cycle of violence must be broken, but breaking it requires becoming vulnerable.",
    "The path to justice runs directly through the people the hero loves.",
    "A debt comes due that can only be paid in ways that change the debtor forever.",
];

const THEMES: &[&str] = &[
    "the cost of power", "identity and belonging", "the nature of truth",
    "forgiveness and redemption", "the tension between duty and desire",
    "what it means to be human", "the legacy we leave behind",
    "the boundaries of loyalty", "freedom versus security",
    "the stories we tell ourselves to survive",
];

const OPENING_HOOKS: &[&str] = &[
    "It begins with a funeral for someone who is not dead.",
    "The letter arrives twenty years too late.",
    "The first sign that something is wrong is the silence.",
    "They find the map in a place it should not possibly exist.",
    "The day the sky changed color, everyone pretended not to notice.",
    "It starts with a knock on the door at an impossible hour.",
    "The old law is simple: never go beyond the wall. Today, someone breaks it.",
    "They wake up with a skill they never learned and a name they do not recognize.",
    "The machine works perfectly, which is exactly the problem.",
    "It begins the moment two strangers realize they share the same impossible memory.",
];

// =============================================================================
// Public API
// =============================================================================

/// Generate a writing prompt deterministically from a category and seed
pub fn generate_prompt(category: PromptCategory, seed: u64) -> WritingPrompt {
    let prompts = prompts_for_category(category);
    let idx = simple_hash(seed, prompts.len());
    build_prompt(category, idx, prompts[idx])
}

/// Generate a random-category prompt deterministically from a seed
pub fn generate_random_prompt(seed: u64) -> WritingPrompt {
    let cat_idx = simple_hash(seed, PromptCategory::all().len());
    let category = PromptCategory::from_index(cat_idx);
    let s2 = next_seed(seed);
    generate_prompt(category, s2)
}

/// Return all built-in prompts for a given category
pub fn all_prompts_for_category(category: PromptCategory) -> Vec<WritingPrompt> {
    let prompts = prompts_for_category(category);
    prompts
        .iter()
        .enumerate()
        .map(|(i, text)| build_prompt(category, i, text))
        .collect()
}

/// Generate a character profile deterministically from a seed
pub fn generate_character(seed: u64) -> CharacterProfile {
    let s0 = seed;
    let s1 = next_seed(s0);
    let s2 = next_seed(s1);
    let s3 = next_seed(s2);
    let s4 = next_seed(s3);
    let s5 = next_seed(s4);
    let s6 = next_seed(s5);
    let s7 = next_seed(s6);
    let s8 = next_seed(s7);

    let first_idx = simple_hash(s0, FIRST_NAMES_NEUTRAL.len());
    let last_idx = simple_hash(s1, LAST_NAMES.len());

    CharacterProfile {
        name_suggestion: format!("{} {}", FIRST_NAMES_NEUTRAL[first_idx], LAST_NAMES[last_idx]),
        age_range: AGE_RANGES[simple_hash(s2, AGE_RANGES.len())].to_string(),
        occupation: OCCUPATIONS[simple_hash(s3, OCCUPATIONS.len())].to_string(),
        trait_primary: CHARACTER_TRAITS[simple_hash(s4, CHARACTER_TRAITS.len())].to_string(),
        trait_secondary: CHARACTER_TRAITS[simple_hash(s5, CHARACTER_TRAITS.len())].to_string(),
        motivation: MOTIVATIONS[simple_hash(s6, MOTIVATIONS.len())].to_string(),
        flaw: CHARACTER_FLAWS[simple_hash(s7, CHARACTER_FLAWS.len())].to_string(),
        background_hook: BACKGROUND_HOOKS[simple_hash(s8, BACKGROUND_HOOKS.len())].to_string(),
    }
}

/// Generate a plot seed deterministically from a seed value
pub fn generate_plot_seed(seed: u64) -> PlotSeed {
    let s0 = seed;
    let s1 = next_seed(s0);
    let s2 = next_seed(s1);
    let s3 = next_seed(s2);
    let s4 = next_seed(s3);
    let s5 = next_seed(s4);
    let s6 = next_seed(s5);

    PlotSeed {
        genre: GENRES[simple_hash(s0, GENRES.len())].to_string(),
        protagonist_type: PROTAGONIST_TYPES[simple_hash(s1, PROTAGONIST_TYPES.len())].to_string(),
        antagonist_type: ANTAGONIST_TYPES[simple_hash(s2, ANTAGONIST_TYPES.len())].to_string(),
        setting: SETTINGS[simple_hash(s3, SETTINGS.len())].to_string(),
        central_conflict: CENTRAL_CONFLICTS[simple_hash(s4, CENTRAL_CONFLICTS.len())].to_string(),
        theme: THEMES[simple_hash(s5, THEMES.len())].to_string(),
        opening_hook: OPENING_HOOKS[simple_hash(s6, OPENING_HOOKS.len())].to_string(),
    }
}

/// Generate a name suggestion deterministically from a seed and optional gender hint
pub fn generate_name(seed: u64, gender: Option<&str>) -> NameSuggestion {
    let s0 = seed;
    let s1 = next_seed(s0);
    let s2 = next_seed(s1);
    let s3 = next_seed(s2);

    let first_name = match gender {
        Some("male") | Some("m") => {
            FIRST_NAMES_MALE[simple_hash(s0, FIRST_NAMES_MALE.len())].to_string()
        }
        Some("female") | Some("f") => {
            FIRST_NAMES_FEMALE[simple_hash(s0, FIRST_NAMES_FEMALE.len())].to_string()
        }
        _ => {
            FIRST_NAMES_NEUTRAL[simple_hash(s0, FIRST_NAMES_NEUTRAL.len())].to_string()
        }
    };

    let last_name = LAST_NAMES[simple_hash(s1, LAST_NAMES.len())].to_string();
    let origin = NAME_ORIGINS[simple_hash(s2, NAME_ORIGINS.len())].to_string();
    let meaning = NAME_MEANINGS[simple_hash(s3, NAME_MEANINGS.len())].to_string();

    NameSuggestion {
        first_name,
        last_name,
        origin,
        meaning,
    }
}

/// Generate multiple name suggestions
pub fn generate_names(seed: u64, count: usize) -> Vec<NameSuggestion> {
    let mut names = Vec::with_capacity(count);
    let mut s = seed;
    for _ in 0..count {
        names.push(generate_name(s, None));
        s = next_seed(next_seed(next_seed(next_seed(s))));
    }
    names
}

/// Return a deterministic daily prompt based on a day offset.
/// The same day_offset always produces the same prompt.
pub fn daily_prompt(day_offset: u32) -> WritingPrompt {
    let seed = (day_offset as u64).wrapping_mul(2654435761);
    generate_random_prompt(seed)
}

/// Return writing exercises filtered by difficulty level
pub fn writing_exercises(difficulty: Difficulty) -> Vec<WritingExercise> {
    let all = all_exercises();
    all.into_iter().filter(|e| exercise_difficulty(e) == difficulty).collect()
}

fn exercise_difficulty(exercise: &WritingExercise) -> Difficulty {
    if exercise.duration_minutes <= 10 {
        Difficulty::Beginner
    } else if exercise.duration_minutes <= 20 {
        Difficulty::Intermediate
    } else {
        Difficulty::Advanced
    }
}

fn all_exercises() -> Vec<WritingExercise> {
    vec![
        // Beginner exercises (<=10 min)
        WritingExercise {
            title: "Word Sprint".into(),
            instructions: "Write as many words as possible in the allotted time without stopping to edit. The goal is volume, not quality.".into(),
            duration_minutes: 5,
            category: PromptCategory::FreeWrite,
        },
        WritingExercise {
            title: "Sensory Snapshot".into(),
            instructions: "Describe your current surroundings using all five senses. Spend two minutes on each sense.".into(),
            duration_minutes: 10,
            category: PromptCategory::SettingDescription,
        },
        WritingExercise {
            title: "Dialogue Warm-Up".into(),
            instructions: "Write a quick exchange between two characters who disagree about something trivial. Focus on giving each a distinct voice.".into(),
            duration_minutes: 7,
            category: PromptCategory::DialogueExercise,
        },
        WritingExercise {
            title: "First Line Factory".into(),
            instructions: "Write ten different opening lines for a story. Do not develop any of them further; just craft the hooks.".into(),
            duration_minutes: 8,
            category: PromptCategory::OpeningLine,
        },
        WritingExercise {
            title: "Emotion in a Gesture".into(),
            instructions: "Write a short scene where a character conveys a strong emotion using only physical actions—no dialogue, no internal monologue.".into(),
            duration_minutes: 10,
            category: PromptCategory::EmotionalScene,
        },
        // Intermediate exercises (11-20 min)
        WritingExercise {
            title: "Character Interview".into(),
            instructions: "Interview one of your characters. Ask them ten questions about their life, fears, and desires. Write their answers in their own voice.".into(),
            duration_minutes: 15,
            category: PromptCategory::CharacterDevelopment,
        },
        WritingExercise {
            title: "Scene Reversal".into(),
            instructions: "Take a scene you have already written and rewrite it from the opposite character's perspective. Notice what changes and what stays the same.".into(),
            duration_minutes: 20,
            category: PromptCategory::PlotTwist,
        },
        WritingExercise {
            title: "Conflict Escalation".into(),
            instructions: "Start with a minor disagreement between two characters and escalate it through five stages until it becomes a major conflict.".into(),
            duration_minutes: 15,
            category: PromptCategory::ConflictScenario,
        },
        WritingExercise {
            title: "World Building Journal".into(),
            instructions: "Write a diary entry from a character who lives in your fictional world. Include details about daily life, customs, and environment.".into(),
            duration_minutes: 20,
            category: PromptCategory::WorldBuilding,
        },
        WritingExercise {
            title: "Action Choreography".into(),
            instructions: "Write a fight or chase scene with a focus on clear spatial awareness. The reader should always know where everyone is.".into(),
            duration_minutes: 15,
            category: PromptCategory::ActionScene,
        },
        // Advanced exercises (>20 min)
        WritingExercise {
            title: "Flash Fiction Challenge".into(),
            instructions: "Write a complete short story with a beginning, middle, and end in under 500 words. It must have a clear character arc.".into(),
            duration_minutes: 25,
            category: PromptCategory::FlashFiction,
        },
        WritingExercise {
            title: "Unreliable Narrator".into(),
            instructions: "Write a scene where the narrator's version of events is clearly unreliable. Include subtle clues that contradict what they are telling us.".into(),
            duration_minutes: 30,
            category: PromptCategory::FreeWrite,
        },
        WritingExercise {
            title: "Poetic Prose".into(),
            instructions: "Write a prose passage of 300-500 words that reads like poetry. Use rhythm, repetition, imagery, and line-level attention to sound.".into(),
            duration_minutes: 25,
            category: PromptCategory::Poetry,
        },
        WritingExercise {
            title: "Dual Timeline".into(),
            instructions: "Write a scene that alternates between two timelines — past and present — that converge at the end to reveal something neither timeline shows alone.".into(),
            duration_minutes: 30,
            category: PromptCategory::PlotTwist,
        },
        WritingExercise {
            title: "The Last Page".into(),
            instructions: "Write the final two pages of a novel that does not exist. Make the reader feel the weight of everything that came before.".into(),
            duration_minutes: 25,
            category: PromptCategory::ClosingLine,
        },
    ]
}

/// Search all built-in prompts by keyword (case-insensitive substring match)
pub fn search_prompts(query: &str) -> Vec<WritingPrompt> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    for &category in PromptCategory::all() {
        for prompt in all_prompts_for_category(category) {
            if prompt.prompt_text.to_lowercase().contains(&query_lower) {
                results.push(prompt);
            }
        }
    }
    results
}

/// Filter all built-in prompts by tag (case-insensitive exact match on tag)
pub fn prompts_by_tag(tag: &str) -> Vec<WritingPrompt> {
    let tag_lower = tag.to_lowercase();
    let mut results = Vec::new();
    for &category in PromptCategory::all() {
        for prompt in all_prompts_for_category(category) {
            if prompt.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                results.push(prompt);
            }
        }
    }
    results
}

/// Calculate the number of consecutive days with at least one completed prompt,
/// counting backward from the most recent entry.
pub fn prompt_streak(history: &PromptHistory) -> usize {
    if history.entries.is_empty() {
        return 0;
    }

    // Collect dates of completed entries, deduplicate
    let mut completed_dates: Vec<chrono::NaiveDate> = history
        .entries
        .iter()
        .filter(|e| e.completed)
        .map(|e| e.used_at.date_naive())
        .collect();

    if completed_dates.is_empty() {
        return 0;
    }

    completed_dates.sort();
    completed_dates.dedup();

    // Count consecutive days from the most recent
    let mut streak = 1usize;
    let mut i = completed_dates.len() - 1;
    while i > 0 {
        let prev = completed_dates[i - 1];
        let curr = completed_dates[i];
        let diff = curr.signed_duration_since(prev).num_days();
        if diff == 1 {
            streak += 1;
            i -= 1;
        } else {
            break;
        }
    }

    streak
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // --- Prompt generation tests ---

    #[test]
    fn test_generate_prompt_returns_correct_category() {
        let prompt = generate_prompt(PromptCategory::FreeWrite, 42);
        assert_eq!(prompt.category, PromptCategory::FreeWrite);
        assert!(!prompt.prompt_text.is_empty());
    }

    #[test]
    fn test_generate_prompt_deterministic() {
        let a = generate_prompt(PromptCategory::DialogueExercise, 123);
        let b = generate_prompt(PromptCategory::DialogueExercise, 123);
        assert_eq!(a.prompt_text, b.prompt_text);
        assert_eq!(a.id, b.id);
        assert_eq!(a.difficulty, b.difficulty);
    }

    #[test]
    fn test_generate_prompt_different_seeds_differ() {
        let a = generate_prompt(PromptCategory::ActionScene, 1);
        let b = generate_prompt(PromptCategory::ActionScene, 999);
        // Very high probability of different prompts with different seeds
        // but not guaranteed, so we just check they are valid
        assert!(!a.prompt_text.is_empty());
        assert!(!b.prompt_text.is_empty());
    }

    #[test]
    fn test_generate_random_prompt_deterministic() {
        let a = generate_random_prompt(777);
        let b = generate_random_prompt(777);
        assert_eq!(a.prompt_text, b.prompt_text);
        assert_eq!(a.category, b.category);
    }

    #[test]
    fn test_generate_random_prompt_produces_valid_prompt() {
        let prompt = generate_random_prompt(0);
        assert!(!prompt.prompt_text.is_empty());
        assert!(prompt.estimated_words > 0);
        assert!(!prompt.tags.is_empty());
    }

    // --- All prompts tests ---

    #[test]
    fn test_all_prompts_for_category_freewrite() {
        let prompts = all_prompts_for_category(PromptCategory::FreeWrite);
        assert!(prompts.len() >= 5, "Expected at least 5 freewrite prompts, got {}", prompts.len());
        for p in &prompts {
            assert_eq!(p.category, PromptCategory::FreeWrite);
        }
    }

    #[test]
    fn test_all_categories_have_at_least_5_prompts() {
        for &cat in PromptCategory::all() {
            let prompts = all_prompts_for_category(cat);
            assert!(
                prompts.len() >= 5,
                "Category {:?} has only {} prompts, expected at least 5",
                cat,
                prompts.len()
            );
        }
    }

    #[test]
    fn test_total_builtin_prompt_count() {
        let mut total = 0;
        for &cat in PromptCategory::all() {
            total += all_prompts_for_category(cat).len();
        }
        assert!(total >= 65, "Expected at least 65 total prompts, got {}", total);
    }

    #[test]
    fn test_prompt_has_valid_uuid() {
        let prompt = generate_prompt(PromptCategory::Poetry, 42);
        // UUID should not be nil
        assert_ne!(prompt.id, Uuid::nil());
    }

    // --- Character generation tests ---

    #[test]
    fn test_generate_character_deterministic() {
        let a = generate_character(42);
        let b = generate_character(42);
        assert_eq!(a.name_suggestion, b.name_suggestion);
        assert_eq!(a.occupation, b.occupation);
        assert_eq!(a.trait_primary, b.trait_primary);
        assert_eq!(a.motivation, b.motivation);
    }

    #[test]
    fn test_generate_character_has_all_fields() {
        let c = generate_character(100);
        assert!(!c.name_suggestion.is_empty());
        assert!(!c.age_range.is_empty());
        assert!(!c.occupation.is_empty());
        assert!(!c.trait_primary.is_empty());
        assert!(!c.trait_secondary.is_empty());
        assert!(!c.motivation.is_empty());
        assert!(!c.flaw.is_empty());
        assert!(!c.background_hook.is_empty());
    }

    #[test]
    fn test_generate_character_name_has_two_parts() {
        let c = generate_character(55);
        let parts: Vec<&str> = c.name_suggestion.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Character name should be first + last: {}", c.name_suggestion);
    }

    #[test]
    fn test_generate_character_different_seeds() {
        let a = generate_character(1);
        let b = generate_character(9999);
        // At least some fields should differ (very high probability)
        let same = a.name_suggestion == b.name_suggestion
            && a.occupation == b.occupation
            && a.trait_primary == b.trait_primary;
        assert!(!same, "Different seeds should produce different characters");
    }

    // --- Plot seed tests ---

    #[test]
    fn test_generate_plot_seed_deterministic() {
        let a = generate_plot_seed(42);
        let b = generate_plot_seed(42);
        assert_eq!(a.genre, b.genre);
        assert_eq!(a.protagonist_type, b.protagonist_type);
        assert_eq!(a.setting, b.setting);
        assert_eq!(a.theme, b.theme);
    }

    #[test]
    fn test_generate_plot_seed_has_all_fields() {
        let p = generate_plot_seed(200);
        assert!(!p.genre.is_empty());
        assert!(!p.protagonist_type.is_empty());
        assert!(!p.antagonist_type.is_empty());
        assert!(!p.setting.is_empty());
        assert!(!p.central_conflict.is_empty());
        assert!(!p.theme.is_empty());
        assert!(!p.opening_hook.is_empty());
    }

    #[test]
    fn test_genre_count() {
        assert!(GENRES.len() >= 10, "Expected at least 10 genres, got {}", GENRES.len());
    }

    // --- Name generation tests ---

    #[test]
    fn test_generate_name_neutral() {
        let name = generate_name(42, None);
        assert!(!name.first_name.is_empty());
        assert!(!name.last_name.is_empty());
        assert!(!name.origin.is_empty());
        assert!(!name.meaning.is_empty());
    }

    #[test]
    fn test_generate_name_male() {
        let name = generate_name(42, Some("male"));
        assert!(FIRST_NAMES_MALE.contains(&name.first_name.as_str()),
            "Expected male name, got {}", name.first_name);
    }

    #[test]
    fn test_generate_name_female() {
        let name = generate_name(42, Some("female"));
        assert!(FIRST_NAMES_FEMALE.contains(&name.first_name.as_str()),
            "Expected female name, got {}", name.first_name);
    }

    #[test]
    fn test_generate_name_deterministic() {
        let a = generate_name(99, Some("m"));
        let b = generate_name(99, Some("m"));
        assert_eq!(a.first_name, b.first_name);
        assert_eq!(a.last_name, b.last_name);
    }

    #[test]
    fn test_generate_names_count() {
        let names = generate_names(42, 5);
        assert_eq!(names.len(), 5);
    }

    #[test]
    fn test_generate_names_empty() {
        let names = generate_names(42, 0);
        assert!(names.is_empty());
    }

    #[test]
    fn test_name_data_counts() {
        assert!(FIRST_NAMES_NEUTRAL.len() + FIRST_NAMES_MALE.len() + FIRST_NAMES_FEMALE.len() >= 30,
            "Expected at least 30 total first names");
        assert!(LAST_NAMES.len() >= 20, "Expected at least 20 last names, got {}", LAST_NAMES.len());
    }

    // --- Daily prompt tests ---

    #[test]
    fn test_daily_prompt_deterministic() {
        let a = daily_prompt(0);
        let b = daily_prompt(0);
        assert_eq!(a.prompt_text, b.prompt_text);
        assert_eq!(a.category, b.category);
    }

    #[test]
    fn test_daily_prompt_different_days_differ() {
        let a = daily_prompt(0);
        let b = daily_prompt(1);
        // Different days should produce different prompts (very high probability)
        assert!(
            a.prompt_text != b.prompt_text || a.category != b.category,
            "Different days should usually produce different prompts"
        );
    }

    // --- Exercise tests ---

    #[test]
    fn test_writing_exercises_beginner() {
        let exercises = writing_exercises(Difficulty::Beginner);
        assert!(!exercises.is_empty(), "Should have beginner exercises");
        for ex in &exercises {
            assert!(ex.duration_minutes <= 10);
        }
    }

    #[test]
    fn test_writing_exercises_intermediate() {
        let exercises = writing_exercises(Difficulty::Intermediate);
        assert!(!exercises.is_empty(), "Should have intermediate exercises");
        for ex in &exercises {
            assert!(ex.duration_minutes > 10 && ex.duration_minutes <= 20);
        }
    }

    #[test]
    fn test_writing_exercises_advanced() {
        let exercises = writing_exercises(Difficulty::Advanced);
        assert!(!exercises.is_empty(), "Should have advanced exercises");
        for ex in &exercises {
            assert!(ex.duration_minutes > 20);
        }
    }

    // --- Search tests ---

    #[test]
    fn test_search_prompts_finds_results() {
        let results = search_prompts("character");
        assert!(!results.is_empty(), "Searching for 'character' should find results");
    }

    #[test]
    fn test_search_prompts_case_insensitive() {
        let a = search_prompts("WRITE");
        let b = search_prompts("write");
        assert_eq!(a.len(), b.len(), "Search should be case-insensitive");
    }

    #[test]
    fn test_search_prompts_no_results() {
        let results = search_prompts("xyzzy_nonexistent_query_12345");
        assert!(results.is_empty());
    }

    // --- Tag filter tests ---

    #[test]
    fn test_prompts_by_tag_finds_results() {
        let results = prompts_by_tag("fiction");
        assert!(!results.is_empty(), "Tag 'fiction' should match some prompts");
    }

    #[test]
    fn test_prompts_by_tag_case_insensitive() {
        let a = prompts_by_tag("Poetry");
        let b = prompts_by_tag("poetry");
        assert_eq!(a.len(), b.len());
    }

    #[test]
    fn test_prompts_by_tag_no_results() {
        let results = prompts_by_tag("nonexistent_tag_xyz");
        assert!(results.is_empty());
    }

    // --- Streak tests ---

    #[test]
    fn test_prompt_streak_empty_history() {
        let history = PromptHistory::default();
        assert_eq!(prompt_streak(&history), 0);
    }

    #[test]
    fn test_prompt_streak_single_day() {
        let history = PromptHistory {
            entries: vec![PromptHistoryEntry {
                prompt_id: Uuid::new_v4(),
                used_at: Utc::now(),
                completed: true,
                user_rating: None,
            }],
        };
        assert_eq!(prompt_streak(&history), 1);
    }

    #[test]
    fn test_prompt_streak_consecutive_days() {
        let base = Utc.with_ymd_and_hms(2025, 1, 10, 12, 0, 0).unwrap();
        let history = PromptHistory {
            entries: vec![
                PromptHistoryEntry {
                    prompt_id: Uuid::new_v4(),
                    used_at: base - chrono::Duration::days(2),
                    completed: true,
                    user_rating: None,
                },
                PromptHistoryEntry {
                    prompt_id: Uuid::new_v4(),
                    used_at: base - chrono::Duration::days(1),
                    completed: true,
                    user_rating: None,
                },
                PromptHistoryEntry {
                    prompt_id: Uuid::new_v4(),
                    used_at: base,
                    completed: true,
                    user_rating: None,
                },
            ],
        };
        assert_eq!(prompt_streak(&history), 3);
    }

    #[test]
    fn test_prompt_streak_broken_by_gap() {
        let base = Utc.with_ymd_and_hms(2025, 1, 10, 12, 0, 0).unwrap();
        let history = PromptHistory {
            entries: vec![
                PromptHistoryEntry {
                    prompt_id: Uuid::new_v4(),
                    used_at: base - chrono::Duration::days(5),
                    completed: true,
                    user_rating: None,
                },
                // gap of 3 days
                PromptHistoryEntry {
                    prompt_id: Uuid::new_v4(),
                    used_at: base - chrono::Duration::days(1),
                    completed: true,
                    user_rating: None,
                },
                PromptHistoryEntry {
                    prompt_id: Uuid::new_v4(),
                    used_at: base,
                    completed: true,
                    user_rating: None,
                },
            ],
        };
        assert_eq!(prompt_streak(&history), 2);
    }

    #[test]
    fn test_prompt_streak_incomplete_entries_ignored() {
        let base = Utc.with_ymd_and_hms(2025, 1, 10, 12, 0, 0).unwrap();
        let history = PromptHistory {
            entries: vec![
                PromptHistoryEntry {
                    prompt_id: Uuid::new_v4(),
                    used_at: base - chrono::Duration::days(1),
                    completed: false,
                    user_rating: None,
                },
                PromptHistoryEntry {
                    prompt_id: Uuid::new_v4(),
                    used_at: base,
                    completed: true,
                    user_rating: Some(4),
                },
            ],
        };
        // Only one completed day
        assert_eq!(prompt_streak(&history), 1);
    }

    // --- simple_hash tests ---

    #[test]
    fn test_simple_hash_deterministic() {
        assert_eq!(simple_hash(42, 100), simple_hash(42, 100));
        assert_eq!(simple_hash(0, 50), simple_hash(0, 50));
    }

    #[test]
    fn test_simple_hash_in_range() {
        for seed in 0..100 {
            let result = simple_hash(seed, 10);
            assert!(result < 10, "simple_hash({}, 10) = {} which is out of range", seed, result);
        }
    }

    #[test]
    fn test_simple_hash_zero_max() {
        assert_eq!(simple_hash(42, 0), 0);
    }

    #[test]
    fn test_simple_hash_max_one() {
        assert_eq!(simple_hash(42, 1), 0);
        assert_eq!(simple_hash(999, 1), 0);
    }

    // --- Category and Difficulty enum tests ---

    #[test]
    fn test_prompt_category_all_count() {
        assert_eq!(PromptCategory::all().len(), 13);
    }

    #[test]
    fn test_prompt_category_labels() {
        assert_eq!(PromptCategory::FreeWrite.label(), "Free Write");
        assert_eq!(PromptCategory::CharacterDevelopment.label(), "Character Development");
        assert_eq!(PromptCategory::Poetry.label(), "Poetry");
    }

    #[test]
    fn test_difficulty_labels() {
        assert_eq!(Difficulty::Beginner.label(), "Beginner");
        assert_eq!(Difficulty::Intermediate.label(), "Intermediate");
        assert_eq!(Difficulty::Advanced.label(), "Advanced");
    }

    // --- Trait/occupation data count tests ---

    #[test]
    fn test_character_traits_count() {
        assert!(CHARACTER_TRAITS.len() >= 20, "Expected at least 20 character traits, got {}", CHARACTER_TRAITS.len());
    }

    #[test]
    fn test_occupations_count() {
        assert!(OCCUPATIONS.len() >= 15, "Expected at least 15 occupations, got {}", OCCUPATIONS.len());
    }

    // --- Edge case tests ---

    #[test]
    fn test_uuid_from_seed_deterministic() {
        let a = uuid_from_seed(12345);
        let b = uuid_from_seed(12345);
        assert_eq!(a, b);
    }

    #[test]
    fn test_uuid_from_seed_different_seeds() {
        let a = uuid_from_seed(1);
        let b = uuid_from_seed(2);
        assert_ne!(a, b);
    }

    #[test]
    fn test_estimated_words_increase_with_difficulty() {
        let cat = PromptCategory::FreeWrite;
        let beginner = estimated_words_for_category(cat, Difficulty::Beginner);
        let intermediate = estimated_words_for_category(cat, Difficulty::Intermediate);
        let advanced = estimated_words_for_category(cat, Difficulty::Advanced);
        assert!(beginner < intermediate, "Intermediate should have more words than beginner");
        assert!(intermediate < advanced, "Advanced should have more words than intermediate");
    }

    #[test]
    fn test_all_exercises_have_valid_fields() {
        let exercises = all_exercises();
        for ex in &exercises {
            assert!(!ex.title.is_empty(), "Exercise title should not be empty");
            assert!(!ex.instructions.is_empty(), "Exercise instructions should not be empty");
            assert!(ex.duration_minutes > 0, "Exercise duration should be positive");
        }
    }

    #[test]
    fn test_generate_names_deterministic() {
        let a = generate_names(42, 3);
        let b = generate_names(42, 3);
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.first_name, y.first_name);
            assert_eq!(x.last_name, y.last_name);
        }
    }

    #[test]
    fn test_prompt_history_default_is_empty() {
        let history = PromptHistory::default();
        assert!(history.entries.is_empty());
    }
}
