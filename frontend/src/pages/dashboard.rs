//! Dashboard page: census, taxonomy and climate figures from `GET /api/v1/reports/dashboard/`
//! (one request, aggregated in the database).
//!
//! Blocks without a data source are labeled as such instead of showing sample numbers:
//! CO₂ absorption and oxygen production (no model yet). The former "monthly growth" and
//! "recent activity" blocks were replaced by the height distribution and the census
//! campaigns, because the census has one measurement per tree and no activity log.
//!
//! KNOWN ISSUE: charts are created via `document.getElementById` and never destroyed.

use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::api::{API_BASE, console_error, fmt_thousands, get_json};
use crate::interop::chart::chart_by_id;
use crate::interop::when_connected;
use super::shared::{HEALTH, health_badge, health_label};

// ---------------------------------------------------------------------------
// API types
// ---------------------------------------------------------------------------

#[derive(Clone, Deserialize)]
struct Summary {
    overall: Overall,
    trees_per_capita: PerCapita,
    phytosanitary_status: std::collections::HashMap<String, i64>,
    top_species: Vec<TopSpecies>,
    records_by_year: RecordsByYear,
    height_distribution: Vec<HeightBin>,
    localities: Vec<LocalityStats>,
    functional_groups: Vec<GroupCount>,
    monthly_temperature: Temperature,
    recent_records: Vec<RecentRecord>,
    recent_climate: Vec<ClimateReading>,
}

#[derive(Clone, Deserialize)]
struct Overall {
    total_trees: i64,
    species_count: i64,
    avg_height_m: Option<f64>,
}

#[derive(Clone, Deserialize)]
struct PerCapita {
    population: Option<i64>,
    value: Option<f64>,
}

#[derive(Clone, Deserialize)]
struct TopSpecies {
    scientific_name: String,
    count: i64,
    percent: f64,
}

#[derive(Clone, Deserialize)]
struct RecordsByYear {
    years: Vec<YearCount>,
    invalid_dates: i64,
}

#[derive(Clone, Deserialize)]
struct YearCount {
    year: i32,
    count: i64,
}

#[derive(Clone, Deserialize)]
struct HeightBin {
    from_m: i64,
    to_m: Option<i64>,
    count: i64,
}

#[derive(Clone, Deserialize)]
struct LocalityStats {
    name: String,
    total_trees: i64,
}

#[derive(Clone, Deserialize)]
struct GroupCount {
    group: Option<i64>,
    count: i64,
}

#[derive(Clone, Deserialize)]
struct Temperature {
    months: Vec<String>,
    stations: Vec<StationSeries>,
}

#[derive(Clone, Deserialize)]
struct StationSeries {
    station: String,
    values: Vec<Option<f64>>,
}

/// Line colors for the temperature series (one per station).
const SERIES_COLORS: [&str; 5] = ["#f97316", "#2563eb", "#16a34a", "#9333ea", "#dc2626"];

#[derive(Clone, Deserialize)]
struct RecentRecord {
    id: i64,
    scientific_name: String,
    date: Option<String>,
    height_m: Option<f64>,
    phytosanitary_status: Option<String>,
    neighborhood: String,
    locality: String,
}

#[derive(Clone, Deserialize)]
struct ClimateReading {
    station: String,
    date: String,
    value: f64,
    unit: String,
}

#[derive(Clone)]
enum Load {
    Loading,
    Failed(String),
    Ready(std::sync::Arc<Summary>),
}

const CARD: &str = "bg-white rounded-lg shadow border border-gray-200 p-4";
const TH: &str = "px-2 py-2 text-left text-xs font-medium text-gray-500 uppercase";

fn month_label(ym: &str) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let (y, m) = ym.split_once('-').unwrap_or((ym, "1"));
    let m: usize = m.parse().unwrap_or(1);
    format!("{} {}", MONTHS[(m.clamp(1, 12)) - 1], &y[y.len().saturating_sub(2)..])
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

#[component]
pub fn Dashboard() -> impl IntoView {
    let state = RwSignal::new(Load::Loading);
    let load = move || {
        state.set(Load::Loading);
        spawn_local(async move {
            let url = format!("{API_BASE}/api/v1/reports/dashboard/");
            let res = get_json(&url)
                .await
                .and_then(|v| serde_json::from_value::<Summary>(v).map_err(|e| e.to_string()));
            match res {
                Ok(s) => state.try_set(Load::Ready(std::sync::Arc::new(s))),
                Err(e) => {
                    console_error("Error loading dashboard:", &e);
                    state.try_set(Load::Failed(e))
                }
            };
        });
    };
    load();

    // Charts need their canvases in the document: once the data is rendered, wait for the
    // page to be connected (route views mount asynchronously), then draw.
    let root = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        let ready = state.with(|s| match s {
            Load::Ready(s) => Some(s.clone()),
            _ => None,
        });
        if let (Some(el), Some(summary)) = (root.get(), ready) {
            when_connected(el.into(), move || {
                set_timeout(move || initialize_charts(&summary), Duration::from_millis(100))
            });
        }
    });

    view! {
        <div node_ref=root class="block w-full">
            <div class="bg-white rounded-lg p-6">
                <div class="mb-6">
                    <h1 class="text-2xl font-bold text-green-600">"Ibagué Urban Forest Dashboard"</h1>
                    <p class="text-gray-600 mt-1">"Overview of urban tree census data and environmental impact in Ibagué"</p>
                </div>

                {move || match state.get() {
                    Load::Loading => view! {
                        <div class="py-24 flex items-center justify-center text-gray-500">
                            <div class="h-8 w-8 mr-3 rounded-full border-4 border-green-200 border-t-green-600 animate-spin"></div>
                            "Loading dashboard…"
                        </div>
                    }
                    .into_any(),
                    Load::Failed(err) => view! {
                        <div class="py-24 flex flex-col items-center justify-center gap-3 text-gray-600">
                            <p class="font-medium text-red-600">"Could not load the dashboard data."</p>
                            <p class="text-sm text-gray-500">{err}</p>
                            <button class="px-4 py-2 rounded-lg bg-green-600 text-white hover:bg-green-700" on:click=move |_| load()>
                                "Retry"
                            </button>
                        </div>
                    }
                    .into_any(),
                    Load::Ready(s) => view! {
                        <div class="mb-8">
                            <h2 class="text-xl font-semibold text-gray-800 mb-3">"Overview"</h2>
                            <h3 class="text-lg font-medium text-gray-700 mb-3">"Tree Statistics"</h3>
                            {stat_cards(&s)}
                            {health_and_campaigns(&s)}
                            {impact_and_species(&s)}
                        </div>
                        {analytics_section()}
                        {records_section(&s)}
                    }
                    .into_any(),
                }}
            </div>
        </div>
    }
}

fn stat_cards(s: &Summary) -> impl IntoView + use<> {
    let height = s.overall.avg_height_m.map(|h| format!("{h:.2} m")).unwrap_or_else(|| "N/A".into());
    let per_capita = s.trees_per_capita.value.map(|v| format!("{v:.2}")).unwrap_or_else(|| "N/A".into());
    let population = s
        .trees_per_capita
        .population
        .map(|p| format!("Trees per inhabitant (pop. 2019: {})", fmt_thousands(p)))
        .unwrap_or_else(|| "Trees per inhabitant".into());

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
            <div class="bg-green-50 p-4 rounded-lg shadow border border-green-100">
                <h4 class="text-xs font-medium text-gray-500 uppercase">"Total Trees"</h4>
                <p class="text-3xl font-bold text-green-600">{fmt_thousands(s.overall.total_trees)}</p>
                <p class="text-green-600 text-xs mt-1">"Urban Tree Census"</p>
            </div>
            <div class="bg-blue-50 p-4 rounded-lg shadow border border-blue-100">
                <h4 class="text-xs font-medium text-gray-500 uppercase">"Species Count"</h4>
                <p class="text-3xl font-bold text-blue-600">{fmt_thousands(s.overall.species_count)}</p>
                <p class="text-blue-600 text-xs mt-1">"Total species diversity"</p>
            </div>
            <div class="bg-purple-50 p-4 rounded-lg shadow border border-purple-100">
                <h4 class="text-xs font-medium text-gray-500 uppercase">"Average Height"</h4>
                <p class="text-3xl font-bold text-purple-600">{height}</p>
                <p class="text-purple-600 text-xs mt-1">"Mean total height"</p>
            </div>
            <div class="bg-amber-50 p-4 rounded-lg shadow border border-amber-100">
                <h4 class="text-xs font-medium text-gray-500 uppercase">"Tree-per-capita"</h4>
                <p class="text-3xl font-bold text-amber-600">{per_capita}</p>
                <p class="text-amber-600 text-xs mt-1">{population}</p>
            </div>
        </div>
    }
}

fn health_and_campaigns(s: &Summary) -> impl IntoView + use<> {
    let legend = HEALTH
        .iter()
        .map(|(code, label, dot, _)| {
            let n = s.phytosanitary_status.get(*code).copied().unwrap_or(0);
            view! {
                <div class="text-center">
                    <span class=format!("block w-4 h-4 mx-auto rounded-full {dot}")></span>
                    <span class="text-xs font-medium block mt-1">{*label}</span>
                    <span class="text-xs text-gray-500">{format!("{} trees", fmt_thousands(n))}</span>
                </div>
            }
        })
        .collect_view();

    let max = s.records_by_year.years.iter().map(|y| y.count).max().unwrap_or(1).max(1);
    let years = s
        .records_by_year
        .years
        .iter()
        .map(|y| {
            let width = format!("width: {:.1}%", 100.0 * y.count as f64 / max as f64);
            view! {
                <li>
                    <div class="flex justify-between text-sm">
                        <span class="font-medium text-gray-900">{y.year}</span>
                        <span class="text-gray-600">{format!("{} records", fmt_thousands(y.count))}</span>
                    </div>
                    <div class="mt-1 h-2 rounded bg-gray-100">
                        <div class="h-2 rounded bg-green-500" style=width></div>
                    </div>
                </li>
            }
        })
        .collect_view();
    let invalid = s.records_by_year.invalid_dates;

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
            // Tree Health Distribution (phytosanitary status of the census observations)
            <div class=CARD>
                <h3 class="text-lg font-medium text-gray-700 mb-3">"Tree Health Distribution"</h3>
                <div class="flex justify-center">
                    <div class="h-64 w-64">
                        <canvas id="healthDistributionChart"></canvas>
                    </div>
                </div>
                <div class="flex justify-between mt-4 text-center">{legend}</div>
            </div>

            // Census campaigns (records per year)
            <div class=CARD>
                <h3 class="text-lg font-medium text-gray-700 mb-3">"Census Campaigns"</h3>
                <p class="text-sm text-gray-500 mb-4">"Trees recorded per year (source: Cortolima)"</p>
                <ul class="space-y-4">{years}</ul>
                {(invalid > 0).then(|| view! {
                    <p class="mt-4 text-xs text-gray-400">
                        {format!("{invalid} record(s) with an invalid date are not shown.")}
                    </p>
                })}
            </div>
        </div>
    }
}

fn impact_and_species(s: &Summary) -> impl IntoView + use<> {
    let top = s.top_species.first().map(|t| t.count).unwrap_or(1).max(1);
    let species = s
        .top_species
        .iter()
        .map(|t| {
            let width = format!("width: {:.1}%", 100.0 * t.count as f64 / top as f64);
            view! {
                <div class="flex flex-col">
                    <span class="text-sm italic">{t.scientific_name.clone()}</span>
                    <div class="flex items-center mt-1">
                        <div class="flex-1 h-2 rounded bg-gray-100">
                            <div class="h-2 bg-green-500 rounded" style=width></div>
                        </div>
                        <span class="ml-2 text-xs w-28 text-right">{format!("{} ({:.2}%)", fmt_thousands(t.count), t.percent)}</span>
                    </div>
                </div>
            }
        })
        .collect_view();

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
            // Environmental Impact: no data source yet
            <div class=CARD>
                <h3 class="text-lg font-medium text-gray-700 mb-3">"Environmental Impact"</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="bg-green-50 p-4 rounded-lg border border-green-100">
                        <h4 class="text-xs font-medium text-gray-500 uppercase">"CO₂ Absorption"</h4>
                        <p class="text-2xl font-bold text-gray-400">"N/A"</p>
                        <p class="text-green-600 text-xs mt-1">"Tons"</p>
                    </div>
                    <div class="bg-blue-50 p-4 rounded-lg border border-blue-100">
                        <h4 class="text-xs font-medium text-gray-500 uppercase">"Oxygen Production"</h4>
                        <p class="text-2xl font-bold text-gray-400">"N/A"</p>
                        <p class="text-blue-600 text-xs mt-1">"Tons/year"</p>
                    </div>
                </div>
                <p class="mt-3 text-xs text-gray-400">
                    "Not computed yet: the database has no carbon or oxygen model for the census trees."
                </p>
            </div>

            // Top Species Distribution
            <div class=CARD>
                <h3 class="text-lg font-medium text-gray-700 mb-3">"Top Species Distribution"</h3>
                <div class="space-y-3">{species}</div>
            </div>
        </div>
    }
}

/// Card with a title, the "more" (⋯) button and a `h-64` canvas.
#[component]
fn ChartCard(title: &'static str, canvas_id: &'static str, #[prop(optional)] note: &'static str) -> impl IntoView {
    view! {
        <div class=CARD>
            <div class="flex justify-between mb-3">
                <div>
                    <h3 class="text-lg font-medium text-gray-700">{title}</h3>
                    {(!note.is_empty()).then(|| view! { <p class="text-xs text-gray-400">{note}</p> })}
                </div>
                <button class="text-gray-400 hover:text-gray-600">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M6 10a2 2 0 11-4 0 2 2 0 014 0zM12 10a2 2 0 11-4 0 2 2 0 014 0zM16 12a2 2 0 100-4 2 2 0 000 4z" />
                    </svg>
                </button>
            </div>
            <div class="h-64">
                <canvas id=canvas_id></canvas>
            </div>
        </div>
    }
}

fn analytics_section() -> impl IntoView {
    view! {
        <div>
            <h2 class="text-xl font-semibold text-gray-800 mb-3">"Analytics"</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
                <ChartCard title="Height Distribution" canvas_id="heightDistributionChart" note="Trees per total-height range" />
                <ChartCard title="Geographic Distribution" canvas_id="geoDistributionChart" note="Trees per comuna" />
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
                <ChartCard title="Temperature Trends" canvas_id="temperatureTrendsChart" note="Monthly mean air temperature per station, last 12 months with data" />
                <ChartCard title="Functional Groups" canvas_id="functionalGroupsChart" note="Trees per functional group of their species" />
            </div>
        </div>
    }
}

fn records_section(s: &Summary) -> impl IntoView + use<> {
    let records = s
        .recent_records
        .iter()
        .map(|t| {
            let status = t.phytosanitary_status.clone().unwrap_or_default();
            let badge = health_badge(&status);
            view! {
                <tr>
                    <td class="px-2 py-2 text-sm">{t.id}</td>
                    <td class="px-2 py-2 text-sm italic">{t.scientific_name.clone()}</td>
                    <td class="px-2 py-2 text-sm whitespace-nowrap">
                        {t.height_m.map(|h| format!("{h:.1} m")).unwrap_or_else(|| "—".into())}
                    </td>
                    <td class="px-2 py-2">
                        <span class=format!("px-2 py-0.5 inline-flex text-xs leading-5 font-semibold rounded-full {badge}")>
                            {health_label(&status)}
                        </span>
                    </td>
                    <td class="px-2 py-2 text-sm">{format!("{}, {}", t.neighborhood, t.locality)}</td>
                    <td class="px-2 py-2 text-sm whitespace-nowrap">{t.date.clone().unwrap_or_default()}</td>
                </tr>
            }
        })
        .collect_view();

    let climate = s
        .recent_climate
        .iter()
        .map(|r| view! {
            <tr>
                <td class="px-2 py-2 text-sm">{r.station.clone()}</td>
                <td class="px-2 py-2 text-sm whitespace-nowrap">{r.date.clone()}</td>
                <td class="px-2 py-2 text-sm whitespace-nowrap">{format!("{:.1} {}", r.value, r.unit)}</td>
            </tr>
        })
        .collect_view();

    view! {
        <div>
            <h2 class="text-xl font-semibold text-gray-800 mb-3">"Records"</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="bg-white rounded-lg shadow border border-gray-200 p-4 overflow-x-auto">
                    <h3 class="text-lg font-medium text-gray-700 mb-3">"Recent Tree Records"</h3>
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead>
                            <tr>
                                <th class=TH>"ID"</th>
                                <th class=TH>"Species"</th>
                                <th class=TH>"Height"</th>
                                <th class=TH>"Health"</th>
                                <th class=TH>"Location"</th>
                                <th class=TH>"Date"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-200">{records}</tbody>
                    </table>
                </div>

                <div class="bg-white rounded-lg shadow border border-gray-200 p-4 overflow-x-auto">
                    <h3 class="text-lg font-medium text-gray-700 mb-3">"Latest Climate Readings"</h3>
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead>
                            <tr>
                                <th class=TH>"Station"</th>
                                <th class=TH>"Date"</th>
                                <th class=TH>"Air temperature"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-200">{climate}</tbody>
                    </table>
                    <p class="mt-3 text-xs text-gray-400">"Last reading of each station. The stations only report air temperature."</p>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Charts
// ---------------------------------------------------------------------------

fn initialize_charts(s: &Summary) {
    create_health_distribution_chart(s);
    create_height_distribution_chart(s);
    create_geo_distribution_chart(s);
    create_temperature_trends_chart(s);
    create_functional_groups_chart(s);
}

fn small_ticks_xy() -> Value {
    json!({
        "y": { "beginAtZero": true, "grid": { "display": true, "color": "rgba(0, 0, 0, 0.05)" }, "ticks": { "font": { "size": 10 } } },
        "x": { "grid": { "display": false }, "ticks": { "font": { "size": 10 } } }
    })
}

fn bar_chart(id: &str, labels: Vec<String>, data: Vec<i64>, label: &str, y_title: &str, x_title: &str) {
    chart_by_id(
        id,
        json!({
            "type": "bar",
            "data": {
                "labels": labels,
                "datasets": [{
                    "label": label,
                    "data": data,
                    "backgroundColor": "#2F7B3D",
                    "borderRadius": 4,
                    "borderSkipped": false
                }]
            },
            "options": {
                "responsive": true,
                "maintainAspectRatio": false,
                "scales": {
                    "y": {
                        "beginAtZero": true,
                        "grid": { "display": true, "color": "rgba(0, 0, 0, 0.05)" },
                        "ticks": { "font": { "size": 10 } },
                        "title": { "display": true, "text": y_title }
                    },
                    "x": {
                        "grid": { "display": false },
                        "ticks": { "font": { "size": 10 } },
                        "title": { "display": true, "text": x_title }
                    }
                },
                "plugins": { "legend": { "display": false } }
            }
        }),
        true,
    );
}

fn create_health_distribution_chart(s: &Summary) {
    let labels: Vec<_> = HEALTH.iter().map(|(_, l, ..)| *l).collect();
    let data: Vec<_> = HEALTH.iter().map(|(c, ..)| s.phytosanitary_status.get(*c).copied().unwrap_or(0)).collect();
    let colors: Vec<_> = HEALTH.iter().map(|(.., color)| *color).collect();
    chart_by_id(
        "healthDistributionChart",
        json!({
            "type": "doughnut",
            "data": {
                "labels": labels,
                "datasets": [{ "data": data, "backgroundColor": colors, "borderWidth": 0 }]
            },
            "options": {
                "responsive": true,
                "cutout": "70%",
                "plugins": { "legend": { "display": false }, "tooltip": { "enabled": true } }
            }
        }),
        true,
    );
}

fn create_height_distribution_chart(s: &Summary) {
    let labels = s
        .height_distribution
        .iter()
        .map(|b| match b.to_m {
            Some(to) => format!("{}–{} m", b.from_m, to),
            None => format!("≥ {} m", b.from_m),
        })
        .collect();
    let data = s.height_distribution.iter().map(|b| b.count).collect();
    bar_chart("heightDistributionChart", labels, data, "Trees", "Trees", "Total height");
}

fn create_geo_distribution_chart(s: &Summary) {
    let labels = s.localities.iter().map(|l| l.name.clone()).collect();
    let data = s.localities.iter().map(|l| l.total_trees).collect();
    bar_chart("geoDistributionChart", labels, data, "Total Trees", "Total de árboles", "Comunas");
}

fn create_temperature_trends_chart(s: &Summary) {
    let labels: Vec<_> = s.monthly_temperature.months.iter().map(|m| month_label(m)).collect();
    let datasets: Vec<_> = s
        .monthly_temperature
        .stations
        .iter()
        .enumerate()
        .map(|(i, st)| {
            let color = SERIES_COLORS[i % SERIES_COLORS.len()];
            json!({
                "label": st.station,
                "data": st.values,
                "borderColor": color,
                "backgroundColor": color,
                "tension": 0.3,
                "pointRadius": 3,
                "borderWidth": 2,
                "spanGaps": false
            })
        })
        .collect();
    let mut scales = small_ticks_xy();
    scales["y"]["beginAtZero"] = json!(false);
    scales["y"]["title"] = json!({ "display": true, "text": "°C" });
    chart_by_id(
        "temperatureTrendsChart",
        json!({
            "type": "line",
            "data": { "labels": labels, "datasets": datasets },
            "options": {
                "responsive": true,
                "maintainAspectRatio": false,
                "scales": scales,
                "plugins": { "legend": { "display": true, "position": "bottom", "labels": { "boxWidth": 10, "font": { "size": 10 } } } }
            }
        }),
        true,
    );
}

fn create_functional_groups_chart(s: &Summary) {
    let labels = s
        .functional_groups
        .iter()
        .map(|g| g.group.map(|n| n.to_string()).unwrap_or_else(|| "None".into()))
        .collect();
    let data = s.functional_groups.iter().map(|g| g.count).collect();
    bar_chart("functionalGroupsChart", labels, data, "Trees", "Trees", "Functional group");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_labels() {
        assert_eq!(month_label("2025-04"), "Apr 25");
        assert_eq!(month_label("2024-12"), "Dec 24");
    }
}
