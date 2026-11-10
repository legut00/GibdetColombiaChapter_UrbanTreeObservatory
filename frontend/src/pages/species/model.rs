//! Species catalog data: API types, code → label tables and filtering.
//!
//! Filters work on the backend *codes* (`TR`, `NT`, `GL`, …), never on the localized
//! `*_display` strings, which the API returns in Spanish.

use std::collections::HashSet;
use std::sync::Arc;

use futures::future::try_join_all;
use serde::Deserialize;

use crate::api::{API_BASE, get_json};

// ---------------------------------------------------------------------------
// API types (subset of /api/v1/taxonomy/species/)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Species {
    pub id: i64,
    pub scientific_name: String,
    #[serde(default)]
    pub accepted_scientific_name: Option<String>,
    /// Most frequent common name in the species' biodiversity records (annotated by the API).
    #[serde(default)]
    pub common_name: Option<String>,
    pub genus: Genus,
    #[serde(default)]
    pub functional_group: Option<FunctionalGroup>,
    pub origin: String,
    pub iucn_status: String,
    pub life_form: String,
    pub canopy_shape: String,
    pub flower_color: String,
    #[serde(default)]
    pub gbif_url: Option<String>,
    #[serde(default)]
    pub tropical_plants_url: Option<String>,
    #[serde(default)]
    pub identified_by: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Genus {
    pub name: String,
    pub family: Family,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Family {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct FunctionalGroup {
    pub group_id: u8,
    #[serde(default)]
    pub trait_values: Vec<TraitValue>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TraitValue {
    #[serde(rename = "trait")]
    pub trait_: Trait,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Trait {
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Deserialize)]
struct Page {
    count: usize,
    results: Vec<Species>,
}

pub type Catalog = Arc<Vec<Arc<Species>>>;

/// Download every page of the species endpoint (page 1 first, the rest in parallel).
pub async fn fetch_all_species() -> Result<Catalog, String> {
    let base = format!("{API_BASE}/api/v1/taxonomy/species/");
    let first: Page = parse(get_json(&base).await?)?;
    let page_size = first.results.len().max(1);
    let pages = first.count.div_ceil(page_size);

    let rest = try_join_all((2..=pages).map(|p| {
        let url = format!("{base}?page={p}");
        async move { parse::<Page>(get_json(&url).await?) }
    }))
    .await?;

    let mut all: Vec<Arc<Species>> = first.results.into_iter().map(Arc::new).collect();
    for page in rest {
        all.extend(page.results.into_iter().map(Arc::new));
    }
    Ok(Arc::new(all))
}

fn parse<T: for<'de> Deserialize<'de>>(v: serde_json::Value) -> Result<T, String> {
    serde_json::from_value(v).map_err(|e| format!("Unexpected API response: {e}"))
}

// ---------------------------------------------------------------------------
// Derived values
// ---------------------------------------------------------------------------

/// Functional-group traits shown on cards: carbon sequestration, shade, canopy, height.
pub const TRAITS: [(&str, &str); 4] = [
    ("CS", "Carbon sequestration (%)"),
    ("SH", "Shade index"),
    ("CY", "Canopy diameter (m)"),
    ("HX", "Maximum height (m)"),
];

impl Species {
    pub fn common_name(&self) -> Option<&str> {
        self.common_name.as_deref().map(str::trim).filter(|n| !n.is_empty())
    }

    pub fn family(&self) -> &str {
        &self.genus.family.name
    }

    pub fn group(&self) -> Option<u8> {
        self.functional_group.as_ref().map(|g| g.group_id)
    }

    /// Formatted trait range ("51.6 – 77.4"), single value, or "—" when unknown.
    pub fn trait_value(&self, ty: &str) -> String {
        let Some(tv) = self
            .functional_group
            .as_ref()
            .and_then(|g| g.trait_values.iter().find(|t| t.trait_.ty == ty))
        else {
            return "—".into();
        };
        match (tv.min_value, tv.max_value) {
            (Some(a), Some(b)) => {
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                if lo == hi { fmt_num(lo) } else { format!("{} – {}", fmt_num(lo), fmt_num(hi)) }
            }
            (Some(v), None) | (None, Some(v)) => fmt_num(v),
            (None, None) => "—".into(),
        }
    }
}

fn fmt_num(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

// ---------------------------------------------------------------------------
// Code tables (backend codes → English labels), in sidebar display order
// ---------------------------------------------------------------------------

/// (code, label, swatch classes)
pub const FLOWER_COLORS: [(&str, &str, &str); 10] = [
    ("NI", "No information", "border border-gray-400 bg-transparent"),
    ("WH", "White", "border border-gray-300 bg-white"),
    ("YE", "Yellow", "bg-yellow-400"),
    ("PI", "Pink", "bg-pink-400"),
    ("RE", "Red", "bg-red-500"),
    ("VI", "Violet", "bg-purple-500"),
    ("OR", "Orange", "bg-orange-500"),
    ("FU", "Fuchsia", "bg-fuchsia-600"),
    ("BR", "Brown", "bg-amber-800"),
    ("GR", "Green", "bg-green-500"),
];

pub const LIFE_FORMS: [(&str, &str); 4] = [
    ("TR", "Tree"),
    ("SH", "Shrub"),
    ("PT", "Palm"),
    ("OT", "Other"),
];

/// (code, label, badge classes, description)
pub const IUCN: [(&str, &str, &str, &str); 9] = [
    ("LC", "Least concern", "bg-green-600", "Widespread and abundant taxa with a low risk of extinction."),
    ("NT", "Near threatened", "bg-teal-700", "Close to qualifying for a threatened category in the near future."),
    ("VU", "Vulnerable", "bg-amber-500", "Facing a high risk of extinction in the wild."),
    ("EN", "Endangered", "bg-orange-600", "Facing a very high risk of extinction in the wild."),
    ("CR", "Critically endangered", "bg-red-700", "Facing an extremely high risk of extinction in the wild."),
    ("EW", "Extinct in the wild", "bg-gray-900", "Known only to survive in cultivation or outside its natural range."),
    ("CD", "Conservation dependent", "bg-lime-600", "Lower risk, but dependent on conservation measures (older IUCN category)."),
    ("DD", "Data deficient", "bg-gray-500", "Not enough information to assess its risk of extinction."),
    ("NE", "Not evaluated", "bg-gray-400", "Not yet assessed against the IUCN Red List criteria."),
];

pub const ORIGINS: [(&str, &str); 7] = [
    ("NT", "Native"),
    ("CU", "Cultivated"),
    ("NC", "Native-cultivated"),
    ("EN", "Endemic"),
    ("NU", "Naturalized"),
    ("NL", "Naturalized-cultivated"),
    ("NI", "Unidentified"),
];

pub const CANOPIES: [(&str, &str); 12] = [
    ("GL", "Globose"),
    ("SG", "Semiglobose"),
    ("OV", "Oval"),
    ("IR", "Irregular"),
    ("SP", "Spreading"),
    ("PY", "Pyramidal"),
    ("CO", "Columnar"),
    ("BD", "Broad"),
    ("CF", "Fan-shaped caulirosula"),
    ("CR", "Feather-shaped caulirosula"),
    ("CS", "Cespitose caulirosula"),
    ("NI", "No information"),
];

fn lookup2(table: &[(&'static str, &'static str)], code: &str) -> &'static str {
    table.iter().find(|(c, _)| *c == code).map(|(_, l)| *l).unwrap_or("Unknown")
}

pub fn life_form_label(code: &str) -> &'static str { lookup2(&LIFE_FORMS, code) }
pub fn origin_label(code: &str) -> &'static str { lookup2(&ORIGINS, code) }
pub fn canopy_label(code: &str) -> &'static str { lookup2(&CANOPIES, code) }

pub fn flower_label(code: &str) -> &'static str {
    FLOWER_COLORS.iter().find(|(c, _, _)| *c == code).map(|(_, l, _)| *l).unwrap_or("Unknown")
}

/// Card icon colors (fill, text) per flower color code.
pub fn flower_fill(code: &str) -> (&'static str, &'static str) {
    match code {
        "WH" => ("#FFFFFF", "#14532D"),
        "YE" => ("#FACC15", "#14532D"),
        "PI" => ("#F9A8D4", "#14532D"),
        "RE" => ("#EF4444", "#FFFFFF"),
        "VI" => ("#8B5CF6", "#FFFFFF"),
        "OR" => ("#FB923C", "#14532D"),
        "FU" => ("#C026D3", "#FFFFFF"),
        "BR" => ("#92400E", "#FFFFFF"),
        "GR" => ("#4ADE80", "#14532D"),
        _ => ("#E5E7EB", "#374151"),
    }
}

/// IUCN Red List category colors (background, text).
pub fn iucn_colors(code: &str) -> (&'static str, &'static str) {
    match code {
        "LC" => ("#11706B", "#FFFFFF"),
        "NT" => ("#CCE226", "#111827"),
        "CD" => ("#9BBF45", "#111827"),
        "VU" => ("#F9E814", "#111827"),
        "EN" => ("#FC7F3F", "#111827"),
        "CR" => ("#D81E05", "#FFFFFF"),
        "EW" => ("#542344", "#FFFFFF"),
        "DD" => ("#D1D1C6", "#111827"),
        _ => ("#FFFFFF", "#111827"),
    }
}

pub fn iucn_info(code: &str) -> (&'static str, &'static str, &'static str) {
    IUCN.iter()
        .find(|(c, ..)| *c == code)
        .map(|(_, l, b, d)| (*l, *b, *d))
        .unwrap_or(("Unknown", "bg-gray-400", ""))
}

// ---------------------------------------------------------------------------
// Filters
// ---------------------------------------------------------------------------

pub const GROUP_MIN: u8 = 1;
pub const GROUP_MAX: u8 = 18;

/// Multi-select filter categories (each selects backend codes).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    FlowerColor,
    LifeForm,
    Iucn,
    Origin,
    Canopy,
}

impl Category {
    pub fn chip_prefix(self) -> &'static str {
        match self {
            Category::FlowerColor => "Flower",
            Category::LifeForm => "Habit",
            Category::Iucn => "IUCN",
            Category::Origin => "Origin",
            Category::Canopy => "Canopy",
        }
    }

    pub fn value(self, s: &Species) -> &str {
        match self {
            Category::FlowerColor => &s.flower_color,
            Category::LifeForm => &s.life_form,
            Category::Iucn => &s.iucn_status,
            Category::Origin => &s.origin,
            Category::Canopy => &s.canopy_shape,
        }
    }

    pub fn label(self, code: &str) -> &'static str {
        match self {
            Category::FlowerColor => flower_label(code),
            Category::LifeForm => life_form_label(code),
            Category::Iucn => iucn_info(code).0,
            Category::Origin => origin_label(code),
            Category::Canopy => canopy_label(code),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Filters {
    pub scientific_name: String,
    pub family: String,
    pub selected: Vec<(Category, &'static str)>,
    pub group_min: u8,
    pub group_max: u8,
    pub saved_only: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            scientific_name: String::new(),
            family: String::new(),
            selected: Vec::new(),
            group_min: GROUP_MIN,
            group_max: GROUP_MAX,
            saved_only: false,
        }
    }
}

impl Filters {
    pub fn is_selected(&self, cat: Category, code: &str) -> bool {
        self.selected.iter().any(|(c, v)| *c == cat && *v == code)
    }

    pub fn toggle(&mut self, cat: Category, code: &'static str) {
        if let Some(i) = self.selected.iter().position(|(c, v)| *c == cat && *v == code) {
            self.selected.remove(i);
        } else {
            self.selected.push((cat, code));
        }
    }

    pub fn group_range_active(&self) -> bool {
        self.group_min != GROUP_MIN || self.group_max != GROUP_MAX
    }

    pub fn is_active(&self) -> bool {
        *self != Filters::default()
    }

    /// Within a category the selected codes are OR-ed; categories are AND-ed.
    pub fn matches(&self, s: &Species, saved: &HashSet<i64>) -> bool {
        let sci = self.scientific_name.trim().to_lowercase();
        if !sci.is_empty() {
            let hit = s.scientific_name.to_lowercase().contains(&sci)
                || s.common_name().is_some_and(|c| c.to_lowercase().contains(&sci))
                || s.accepted_scientific_name
                    .as_deref()
                    .is_some_and(|a| a.to_lowercase().contains(&sci));
            if !hit {
                return false;
            }
        }

        let fam = self.family.trim().to_lowercase();
        if !fam.is_empty()
            && !s.family().to_lowercase().contains(&fam)
            && !s.genus.name.to_lowercase().contains(&fam)
        {
            return false;
        }

        for cat in [Category::FlowerColor, Category::LifeForm, Category::Iucn, Category::Origin, Category::Canopy] {
            let mut chosen = self.selected.iter().filter(|(c, _)| *c == cat).peekable();
            if chosen.peek().is_some() && !chosen.any(|(_, code)| *code == cat.value(s)) {
                return false;
            }
        }

        // Species without a functional group only pass while the full range is selected.
        if self.group_range_active() {
            match s.group() {
                Some(g) if g >= self.group_min && g <= self.group_max => {}
                _ => return false,
            }
        }

        !self.saved_only || saved.contains(&s.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn species(json: serde_json::Value) -> Species {
        let mut base = serde_json::json!({
            "id": 1, "scientific_name": "Ficus benjamina", "accepted_scientific_name": null,
            "genus": {"name": "Ficus", "family": {"name": "Moraceae"}},
            "functional_group": null, "origin": "CU", "iucn_status": "LC",
            "life_form": "TR", "canopy_shape": "GL", "flower_color": "NI"
        });
        base.as_object_mut().unwrap().extend(json.as_object().unwrap().clone());
        serde_json::from_value(base).unwrap()
    }

    #[test]
    fn trait_ranges_are_sorted_and_formatted() {
        let s = species(serde_json::json!({"functional_group": {"group_id": 6, "trait_values": [
            {"trait": {"type": "CS"}, "min_value": 100.0, "max_value": 63.64},
            {"trait": {"type": "HX"}, "min_value": 15.0, "max_value": 15.0},
            {"trait": {"type": "SH"}, "min_value": null, "max_value": 51.6}
        ]}}));
        assert_eq!(s.trait_value("CS"), "63.64 – 100");
        assert_eq!(s.trait_value("HX"), "15");
        assert_eq!(s.trait_value("SH"), "51.6");
        assert_eq!(s.trait_value("CY"), "—");
    }

    #[test]
    fn filters_or_within_category_and_across() {
        let palm = species(serde_json::json!({"id": 2, "life_form": "PT"}));
        let tree = species(serde_json::json!({"id": 3}));
        let saved = HashSet::from([3]);
        let mut f = Filters::default();
        f.toggle(Category::LifeForm, "PT");
        assert!(f.matches(&palm, &saved) && !f.matches(&tree, &saved));
        f.toggle(Category::LifeForm, "TR");
        assert!(f.matches(&palm, &saved) && f.matches(&tree, &saved));
        f.saved_only = true;
        assert!(!f.matches(&palm, &saved) && f.matches(&tree, &saved));
    }

    #[test]
    fn narrowing_groups_hides_species_without_group() {
        let none = species(serde_json::json!({}));
        let mut f = Filters::default();
        assert!(f.matches(&none, &HashSet::new()));
        f.group_max = 17;
        assert!(!f.matches(&none, &HashSet::new()));
    }
}
