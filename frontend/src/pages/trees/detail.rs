//! Full record of one tree (overlay): species, location, every measurement with its date
//! (latest per metric first, older ones as history) and every census observation field.
//! Closes on backdrop click, the X button or Escape.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use serde_json::Value;

use super::super::shared::{health_badge, health_label};
use super::super::species::model::{self, fmt_num};
use super::inventory_url;
use crate::api::{console_error, get_json};

const SECTION: &str = "text-sm font-semibold text-gray-900 uppercase tracking-wider border-b border-gray-200 pb-1 mb-2";
const DT: &str = "text-xs font-medium text-gray-500";
const DD: &str = "text-sm text-gray-900";
const TH: &str = "px-2 py-1.5 text-left text-xs font-medium text-gray-500 uppercase";
const TD: &str = "px-2 py-1.5 text-sm whitespace-nowrap";
const ICON_X: &str = "M6 18L18 6M6 6l12 12";

/// Observation fields shown under "Condition"; the rest are census form codes.
const CONDITION: [&str; 7] = [
    "phytosanitary_status",
    "physical_condition",
    "foliage_density",
    "aesthetic_value",
    "growth_phase",
    "reproductive_condition",
    "standing",
];
/// Free-text / provenance fields, shown under "Notes".
const NOTES: [&str; 4] = ["field_notes", "recorded_by", "accompanying_collectors", "date"];

#[derive(Clone, Deserialize)]
struct Tree {
    id: i64,
    uuid: String,
    common_name: Option<String>,
    species: SpeciesInfo,
    neighborhood: String,
    locality: String,
    site: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    elevation_m: Option<f64>,
    recorded_by: Option<String>,
    date: Option<String>,
    measurements: Vec<Measurement>,
    observations: Vec<Observation>,
}

#[derive(Clone, Deserialize)]
struct SpeciesInfo {
    scientific_name: String,
    accepted_scientific_name: Option<String>,
    family: String,
    genus: String,
    life_form: String,
    origin: String,
    iucn_status: String,
}

#[derive(Clone, Deserialize)]
struct Measurement {
    attribute: String,
    label: String,
    value: f64,
    unit: String,
    method: String,
    date: Option<String>,
}

#[derive(Clone, Deserialize)]
struct Observation {
    date: Option<String>,
    fields: Vec<ObsField>,
}

#[derive(Clone, Deserialize)]
struct ObsField {
    field: String,
    label: String,
    value: Value,
    display: Option<String>,
}

impl ObsField {
    fn text(&self) -> String {
        let raw = match &self.value {
            Value::Null => String::new(),
            Value::String(s) => s.clone(),
            v => v.to_string(),
        };
        let shown = self.display.clone().unwrap_or(raw);
        if shown.is_empty() || shown == "NR" { "—".into() } else { capitalize(&shown) }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn dash(v: Option<String>) -> String {
    v.filter(|s| !s.is_empty()).map(|s| s.replace('_', " ")).unwrap_or_else(|| "—".into())
}

#[derive(Clone)]
enum Load {
    Loading,
    Failed,
    Ready(Box<Tree>),
}

#[component]
pub(super) fn TreeDetail(id: i64, on_close: Callback<()>) -> impl IntoView {
    let close = move || on_close.run(());
    let handle = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            close();
        }
    });
    on_cleanup(move || handle.remove());

    let data = RwSignal::new(Load::Loading);
    spawn_local(async move {
        let res = get_json(&format!("{}{id}/", inventory_url()))
            .await
            .and_then(|v| serde_json::from_value::<Tree>(v).map_err(|e| e.to_string()));
        match res {
            Ok(t) => data.try_set(Load::Ready(Box::new(t))),
            Err(e) => {
                console_error("Error loading the tree:", &e);
                data.try_set(Load::Failed)
            }
        };
    });

    let content = move || match data.get() {
        Load::Loading => view! { <p class="py-16 text-center text-sm text-gray-500">"Loading tree…"</p> }.into_any(),
        Load::Failed => view! { <p class="py-16 text-center text-sm text-red-600">"Could not load this tree."</p> }.into_any(),
        Load::Ready(t) => tree_view(*t).into_any(),
    };

    view! {
        <div class="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4" on:click=move |_| close()>
            <div
                class="relative bg-white rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] overflow-y-auto p-6"
                role="dialog"
                aria-modal="true"
                aria-label=format!("Tree {id}")
                on:click=|ev| ev.stop_propagation()
            >
                <button class="absolute top-3 right-3 p-1 rounded hover:bg-gray-100 text-gray-600" title="Close" on:click=move |_| close()>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=ICON_X />
                    </svg>
                </button>
                {content}
            </div>
        </div>
    }
}

fn fact(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div>
            <dt class=DT>{label}</dt>
            <dd class=DD>{value}</dd>
        </div>
    }
}

fn tree_view(t: Tree) -> impl IntoView {
    let s = &t.species;
    let accepted = s.accepted_scientific_name.clone().filter(|a| !a.is_empty() && *a != s.scientific_name);
    let (iucn_label, ..) = model::iucn_info(&s.iucn_status);
    let (iucn_bg, iucn_fg) = model::iucn_colors(&s.iucn_status);

    let latest_obs = t.observations.first().cloned();
    let field = |name: &str| latest_obs.as_ref().and_then(|o| o.fields.iter().find(|f| f.field == name).cloned());
    let status = field("phytosanitary_status")
        .and_then(|f| f.value.as_str().map(str::to_string))
        .unwrap_or_else(|| "NR".into());
    let last_measured = t.measurements.iter().filter_map(|m| m.date.clone()).max();
    let coords = match (t.latitude, t.longitude) {
        (Some(lat), Some(lon)) => format!("{lat:.6}, {lon:.6}"),
        _ => "—".into(),
    };

    view! {
        <div class="space-y-6">
            // Title
            <div class="pr-10">
                <p class="text-xs font-medium text-gray-500">{format!("Tree #{}", t.id)}</p>
                <h2 class="text-2xl font-bold text-gray-900">{t.common_name.clone().unwrap_or_else(|| "—".into())}</h2>
                <p class="text-lg italic text-gray-700">{s.scientific_name.clone()}</p>
                <div class="mt-2 flex flex-wrap gap-2 text-xs font-semibold">
                    <span class=format!("px-2 py-0.5 rounded-full {}", health_badge(&status))>{health_label(&status)}</span>
                    <span class="px-2 py-0.5 rounded-full" style=format!("background:{iucn_bg};color:{iucn_fg}")>
                        {format!("IUCN: {iucn_label}")}
                    </span>
                    {last_measured.clone().map(|d| view! {
                        <span class="px-2 py-0.5 rounded-full bg-blue-50 text-blue-800">{format!("Last measured {d}")}</span>
                    })}
                </div>
            </div>

            <div class="grid gap-6 md:grid-cols-2">
                <section>
                    <h3 class=SECTION>"Species"</h3>
                    <dl class="grid grid-cols-2 gap-x-4 gap-y-2">
                        {fact("Family", s.family.clone())}
                        {fact("Genus", s.genus.clone())}
                        {fact("Life form", capitalize(&s.life_form))}
                        {fact("Origin", capitalize(&s.origin))}
                        {accepted.map(|a| view! { <div class="col-span-2">{fact("Accepted name", a)}</div> })}
                    </dl>
                </section>
                <section>
                    <h3 class=SECTION>"Location & record"</h3>
                    <dl class="grid grid-cols-2 gap-x-4 gap-y-2">
                        {fact("Neighborhood", t.neighborhood.clone())}
                        {fact("Comuna", t.locality.clone())}
                        {fact("Site", dash(t.site.clone()))}
                        {fact("Elevation", t.elevation_m.map(|e| format!("{} m", fmt_num(e))).unwrap_or_else(|| "—".into()))}
                        {fact("Coordinates (lat, lon)", coords)}
                        {fact("Recorded", dash(t.date.clone()))}
                        {fact("Recorded by", dash(t.recorded_by.clone()))}
                        <div class="col-span-2">{fact("UUID", t.uuid.clone())}</div>
                    </dl>
                </section>
            </div>

            <section>
                <h3 class=SECTION>"Measurements"</h3>
                {measurements_table(&t.measurements)}
            </section>

            {match latest_obs {
                Some(o) => observation_view(o, t.observations.len(), t.date.clone()).into_any(),
                None => view! {
                    <section>
                        <h3 class=SECTION>"Condition"</h3>
                        <p class="text-sm text-gray-500">"No observations recorded."</p>
                    </section>
                }.into_any(),
            }}
        </div>
    }
}

fn measurements_table(rows: &[Measurement]) -> AnyView {
    if rows.is_empty() {
        return view! { <p class="text-sm text-gray-500">"No measurements recorded."</p> }.into_any();
    }
    // Rows come grouped by metric, newest first: the first of each metric is current.
    let mut seen = std::collections::HashSet::new();
    let body = rows
        .iter()
        .map(|m| {
            let latest = seen.insert(m.attribute.clone());
            let cls = if latest { "text-gray-900" } else { "text-gray-400" };
            view! {
                <tr class=cls>
                    <td class=TD>
                        {if latest { capitalize(&m.label) } else { "↳ earlier".to_string() }}
                    </td>
                    <td class=format!("{TD} text-right font-medium")>{fmt_num(m.value)}</td>
                    <td class=TD>{m.unit.clone()}</td>
                    <td class=TD>{capitalize(&m.method)}</td>
                    <td class=TD>{dash(m.date.clone())}</td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
                <thead>
                    <tr>
                        <th class=TH>"Metric"</th>
                        <th class=format!("{TH} text-right")>"Value"</th>
                        <th class=TH>"Unit"</th>
                        <th class=TH>"Method"</th>
                        <th class=TH>"Measured on"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-100">{body}</tbody>
            </table>
        </div>
    }
    .into_any()
}

fn observation_view(o: Observation, total: usize, record_date: Option<String>) -> impl IntoView {
    let pick = |names: &[&str]| {
        names
            .iter()
            .filter_map(|n| o.fields.iter().find(|f| f.field == *n).cloned())
            .collect::<Vec<_>>()
    };
    let condition = pick(&CONDITION);
    let notes: Vec<_> = pick(&NOTES).into_iter().filter(|f| f.field != "date").collect();
    let codes: Vec<_> = o
        .fields
        .iter()
        .filter(|f| !CONDITION.contains(&f.field.as_str()) && !NOTES.contains(&f.field.as_str()) && f.field != "photo_url")
        .cloned()
        .collect();
    let photo = o
        .fields
        .iter()
        .find(|f| f.field == "photo_url")
        .and_then(|f| f.value.as_str().map(str::to_string))
        .filter(|u| !u.is_empty() && u != "NR");
    let observed = match (o.date.clone(), record_date) {
        (Some(d), _) => format!("Observed on {d}"),
        (None, Some(d)) => format!("No observation date; tree recorded on {d}"),
        (None, None) => "No observation date".into(),
    };
    let older = (total > 1).then(|| format!(" · latest of {total} observations"));

    let grid = |fields: Vec<ObsField>, upper: bool| {
        fields
            .into_iter()
            .map(|f| {
                let label = if upper { f.label.to_uppercase() } else { capitalize(&f.label) };
                view! {
                    <div>
                        <dt class=DT>{label}</dt>
                        <dd class=DD>{f.text()}</dd>
                    </div>
                }
            })
            .collect_view()
    };

    view! {
        <section>
            <h3 class=SECTION>"Condition"</h3>
            <p class="text-xs text-gray-500 mb-2">{observed}{older}</p>
            <dl class="grid grid-cols-2 sm:grid-cols-4 gap-x-4 gap-y-2">{grid(condition, false)}</dl>
        </section>
        <section>
            <h3 class=SECTION>"Other census fields"</h3>
            <p class="text-xs text-gray-500 mb-2">"Codes from the Cortolima census form, shown as recorded."</p>
            <dl class="grid grid-cols-3 sm:grid-cols-6 gap-x-4 gap-y-2">{grid(codes, true)}</dl>
        </section>
        <section>
            <h3 class=SECTION>"Notes"</h3>
            <dl class="grid grid-cols-2 sm:grid-cols-4 gap-x-4 gap-y-2">
                {grid(notes, false)}
                <div>
                    <dt class=DT>"Census photo"</dt>
                    <dd class=DD>
                        {match photo {
                            Some(url) => view! {
                                <a href=url target="_blank" rel="noopener noreferrer" class="text-primary-700 hover:underline">"Open (Cortolima)"</a>
                            }.into_any(),
                            None => "—".into_any(),
                        }}
                    </dd>
                </div>
            </dl>
        </section>
    }
}
