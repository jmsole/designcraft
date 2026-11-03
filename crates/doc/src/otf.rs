//! OpenType features as the Character panel's OpenType menu offers them, kept in a character
//! format's `otf_features` tag list (`"dlig"` on, `"-calt"` off, `"aalt=2"` the value 2: the
//! second alternate).

/// On/off features: (tag, menu label, IDML attribute).
pub const TOGGLES: &[(&str, &str, &str)] = &[
    ("dlig", "Discretionary Ligatures", "OTFDiscretionaryLigature"),
    ("frac", "Fractions", "OTFFraction"),
    ("ordn", "Ordinal", "OTFOrdinal"),
    ("swsh", "Swash", "OTFSwash"),
    ("titl", "Titling Alternates", "OTFTitling"),
    ("calt", "Contextual Alternates", "OTFContextualAlternate"),
    ("zero", "Slashed Zero", "OTFSlashedZero"),
    ("palt", "Proportional Metrics", "OTFProportionalMetrics"),
    ("hkna", "Horizontal Kana", "OTFHVKana"),
    ("locl", "Localized Forms", "OTFLocale"),
    ("mark", "Mark Positioning", "OTFMark"),
    ("hist", "Historical Forms", "OTFHistorical"),
    ("salt", "Stylistic Alternates", "OTFStylisticAlternate"),
    ("jalt", "Justification Alternates", "OTFJustificationAlternate"),
    ("ital", "Roman Italics", "OTFRomanItalics"),
];

/// Figure styles: (id, menu label, tags, IDML value).
pub const FIGURES: &[(&str, &str, &[&str], &str)] = &[
    ("tabularLining", "Tabular Lining", &["tnum", "lnum"], "TabularLining"),
    ("proportionalOldstyle", "Proportional Oldstyle", &["pnum", "onum"], "ProportionalOldstyle"),
    ("proportionalLining", "Proportional Lining", &["pnum", "lnum"], "ProportionalLining"),
    ("tabularOldstyle", "Tabular Oldstyle", &["tnum", "onum"], "TabularOldstyle"),
    ("default", "Default Figure Style", &[], "Default"),
];

/// Features shaping turns on by itself: "off" is an explicit `-tag`.
fn default_on(tag: &str) -> bool {
    matches!(tag, "calt" | "liga" | "locl" | "mark")
}

pub fn is_on(list: &[String], tag: &str) -> bool {
    if default_on(tag) { !list.iter().any(|t| t.strip_prefix('-') == Some(tag)) } else { list.iter().any(|t| t == tag) }
}

/// The feature tag of a list entry: `"-calt"` → `"calt"`, `"aalt=2"` → `"aalt"`.
pub fn tag_of(entry: &str) -> &str {
    let t = entry.strip_prefix('-').unwrap_or(entry);
    t.split_once('=').map_or(t, |(t, _)| t)
}

/// The value a list entry gives its feature: 0 off, 1 on, N the Nth alternate.
pub fn value_of(entry: &str) -> Option<u32> {
    match entry.strip_prefix('-') {
        Some(_) => Some(0),
        None => entry.split_once('=').map_or(Some(1), |(_, n)| n.parse().ok()),
    }
}

pub fn set(list: &mut Vec<String>, tag: &str, on: bool) {
    list.retain(|t| tag_of(t) != tag);
    if on != default_on(tag) {
        list.push(if on { tag.to_string() } else { format!("-{tag}") });
    }
}

/// Sets a feature to a value as IDML's `OpenTypeFeatures` list stores it: 0 off, 1 on, N > 1 the
/// Nth alternate (`"aalt=2"`).
pub fn set_value(list: &mut Vec<String>, tag: &str, value: u32) {
    if value > 1 {
        list.retain(|t| tag_of(t) != tag);
        list.push(format!("{tag}={value}"));
    } else {
        set(list, tag, value == 1);
    }
}

/// The figure style id of a tag list ("default" when none matches).
pub fn figures(list: &[String]) -> &'static str {
    FIGURES.iter().find(|(_, _, tags, _)| !tags.is_empty() && tags.iter().all(|t| list.iter().any(|x| x == t))).map_or("default", |f| f.0)
}

pub fn set_figures(list: &mut Vec<String>, id: &str) -> bool {
    let Some((_, _, tags, _)) = FIGURES.iter().find(|f| f.0 == id) else { return false };
    list.retain(|t| !matches!(t.as_str(), "tnum" | "pnum" | "lnum" | "onum"));
    list.extend(tags.iter().map(ToString::to_string));
    true
}

/// Stylistic sets 1–20 as a bit mask, IDML's `OTFStylisticSets`: ssNN is bit NN (ss01 = 2), as in
/// InDesign; bit 0 is unused.
pub fn stylistic_sets(list: &[String]) -> u32 {
    list.iter().filter_map(|t| t.strip_prefix("ss")?.parse::<u32>().ok()).filter(|n| (1..=20).contains(n)).fold(0, |m, n| m | 1 << n)
}

pub fn set_stylistic_sets(list: &mut Vec<String>, mask: u32) {
    list.retain(|t| !(t.len() == 4 && t.starts_with("ss") && t[2..].parse::<u32>().is_ok()));
    list.extend((1..=20).filter(|n| mask & (1 << n) != 0).map(|n| format!("ss{n:02}")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_figures_and_sets() {
        let mut l: Vec<String> = vec![];
        assert!(is_on(&l, "calt") && !is_on(&l, "dlig"));
        set(&mut l, "dlig", true);
        set(&mut l, "calt", false);
        assert_eq!(l, ["dlig", "-calt"]);
        set(&mut l, "calt", true);
        assert_eq!(l, ["dlig"]);
        assert!(set_figures(&mut l, "tabularOldstyle"));
        assert_eq!(figures(&l), "tabularOldstyle");
        set_figures(&mut l, "default");
        assert_eq!(figures(&l), "default");
        set_stylistic_sets(&mut l, 0b1011);
        assert_eq!(stylistic_sets(&l), 0b1010, "bit 0 sets nothing");
        assert!(l.contains(&"ss01".to_string()) && l.contains(&"ss03".to_string()) && !l.contains(&"ss02".to_string()));
    }

    #[test]
    fn feature_values() {
        let mut l: Vec<String> = vec![];
        set_value(&mut l, "ss01", 1);
        set_value(&mut l, "aalt", 2);
        set_value(&mut l, "calt", 0);
        set_value(&mut l, "liga", 1);
        assert_eq!(l, ["ss01", "aalt=2", "-calt"]);
        assert_eq!(l.iter().map(|t| (tag_of(t), value_of(t))).collect::<Vec<_>>(), [("ss01", Some(1)), ("aalt", Some(2)), ("calt", Some(0))]);
        // A later value replaces the earlier one, in either direction.
        set_value(&mut l, "aalt", 3);
        set(&mut l, "ss01", false);
        assert_eq!(l, ["-calt", "aalt=3"]);
        set(&mut l, "aalt", true);
        assert_eq!(l, ["-calt", "aalt"]);
    }
}
