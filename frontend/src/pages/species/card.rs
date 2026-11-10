//! Grid card, list table and small shared pieces (icons, badges, bookmark button).

use std::sync::Arc;

use leptos::prelude::*;

use super::Ctx;
use super::model::{self, Species, TRAITS};

const TH_SM: &str = "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider";

pub(super) const ICON_BOOKMARK: &str = "M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z";

/// Heroicons-style outline icon.
pub(super) fn svg(d: &'static str, class: &'static str) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class=class fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=d />
        </svg>
    }
}

/// Simple line drawing for each life form (placeholder: the API has no photos).
fn life_form_paths(code: &str) -> &'static str {
    match code {
        // Tree
        "TR" => r#"<circle cx="12" cy="9" r="6"/><path d="M12 15v6M9 21h6"/>"#,
        // Shrub
        "SH" => r#"<path d="M3 18c0-4.5 4-8 9-8s9 3.5 9 8z"/><path d="M8 18c0-2 1.5-4 4-4s4 2 4 4"/><path d="M12 18v3"/>"#,
        // Palm
        "PT" => r#"<path d="M12 21V9"/><path d="M12 9C9.5 5.5 6 5 3 7"/><path d="M12 9c2.5-3.5 6-4 9-2"/><path d="M12 9C11 5.5 8.5 3.5 6 3"/><path d="M12 9c1-3.5 3.5-5.5 6-6"/>"#,
        // Other: sprout
        _ => r#"<path d="M12 21v-8"/><path d="M12 13c-4 0-6.5-3-6.5-7 4 0 6.5 3 6.5 7z"/><path d="M12 13c0-4 2.5-7 6.5-7 0 4-2.5 7-6.5 7z"/>"#,
    }
}

pub(super) fn life_form_icon(code: &str, class: &'static str) -> impl IntoView + use<> {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class=class fill="none" viewBox="0 0 24 24" stroke="currentColor"
            stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" inner_html=life_form_paths(code)>
        </svg>
    }
}

pub(super) fn iucn_badge(code: &str) -> impl IntoView + use<> {
    let (label, bg, desc) = model::iucn_info(code);
    view! {
        <span class=format!("inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold text-white {bg}") title=desc>
            {format!("{code} · {label}")}
        </span>
    }
}

pub(super) fn group_badge(group: Option<u8>) -> impl IntoView + use<> {
    match group {
        Some(g) => view! {
            <span class="px-2 py-0.5 rounded-full text-xs font-semibold bg-primary-600 text-white" title="Functional group">
                {format!("Group {g}")}
            </span>
        }
        .into_any(),
        None => view! {
            <span class="px-2 py-0.5 rounded-full text-xs font-medium bg-gray-200 text-gray-600" title="No functional group assigned">
                "No group"
            </span>
        }
        .into_any(),
    }
}

pub(super) fn flower_swatch(code: &str) -> impl IntoView + use<> {
    let (_, label, swatch) = model::FLOWER_COLORS
        .iter()
        .copied()
        .find(|(c, ..)| *c == code)
        .unwrap_or(("NI", "No information", "border border-gray-400"));
    view! {
        <span class="inline-flex items-center gap-1.5 text-xs text-gray-600" title="Flower color">
            <span class=format!("h-3 w-3 rounded-full {swatch}")></span>
            {label}
        </span>
    }
}

#[component]
pub(super) fn BookmarkButton(id: i64, ctx: Ctx, #[prop(default = "text-primary-700")] color: &'static str) -> impl IntoView {
    let saved = move || ctx.is_saved(id);
    view! {
        <button
            class=format!("p-1 rounded hover:bg-black/10 {color}")
            title=move || if saved() { "Remove from saved" } else { "Save species" }
            on:click=move |ev| {
                ev.stop_propagation();
                ctx.toggle_saved(id);
            }
        >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" stroke="currentColor"
                fill=move || if saved() { "currentColor" } else { "none" }>
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=ICON_BOOKMARK />
            </svg>
        </button>
    }
}

#[component]
pub(super) fn SpeciesCard(species: Arc<Species>, ctx: Ctx) -> impl IntoView {
    let s = species.clone();
    let open = move |_| ctx.selected.set(Some(species.clone()));
    let has_group = s.functional_group.is_some();

    view! {
        <article
            class="bg-white rounded-lg shadow hover:shadow-md transition-shadow cursor-pointer flex flex-col overflow-hidden"
            on:click=open
        >
            <div class="bg-primary-50 px-4 py-3 flex items-start justify-between gap-2">
                <div class="min-w-0">
                    <h3 class="italic font-semibold text-primary-900 truncate" title=s.scientific_name.clone()>
                        {s.scientific_name.clone()}
                    </h3>
                    <p class="text-xs text-gray-500 truncate">{format!("{} · {}", s.family(), s.genus.name)}</p>
                </div>
                <div class="flex items-center gap-1 shrink-0">
                    {group_badge(s.group())}
                    <BookmarkButton id=s.id ctx />
                </div>
            </div>

            <div class="px-4 py-3 flex items-center gap-3 border-b border-gray-100">
                <div class="h-12 w-12 shrink-0 rounded-full bg-primary-100 text-primary-700 flex items-center justify-center">
                    {life_form_icon(&s.life_form, "h-7 w-7")}
                </div>
                <div class="text-sm min-w-0">
                    <div class="font-medium text-gray-800">{model::life_form_label(&s.life_form)}</div>
                    <div class="text-gray-500 truncate">{if s.canopy_shape == "NI" {
                        model::origin_label(&s.origin).to_string()
                    } else {
                        format!("{} · {} canopy", model::origin_label(&s.origin), model::canopy_label(&s.canopy_shape).to_lowercase())
                    }}</div>
                </div>
            </div>

            <div class="px-4 py-3 flex-1 text-sm">
                {if has_group {
                    TRAITS
                        .iter()
                        .map(|(ty, label)| {
                            view! {
                                <div class="flex justify-between gap-2 py-0.5">
                                    <span class="text-gray-500">{*label}</span>
                                    <span class="font-medium text-gray-800 whitespace-nowrap">{s.trait_value(ty)}</span>
                                </div>
                            }
                        })
                        .collect_view()
                        .into_any()
                } else {
                    view! { <p class="text-gray-400 italic">"No functional-group metrics"</p> }.into_any()
                }}
            </div>

            <div class="px-4 py-2 border-t border-gray-100 flex items-center justify-between gap-2">
                {iucn_badge(&s.iucn_status)}
                {flower_swatch(&s.flower_color)}
            </div>
        </article>
    }
}

#[component]
pub(super) fn SpeciesTable(rows: Memo<Vec<Arc<Species>>>, ctx: Ctx) -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th class=TH_SM>"Scientific name"</th>
                        <th class=TH_SM>"Family"</th>
                        <th class=TH_SM>"Habit"</th>
                        <th class=TH_SM>"Origin"</th>
                        <th class=TH_SM>"IUCN"</th>
                        <th class=TH_SM>"Group"</th>
                        <th class=TH_SM>"Max height (m)"</th>
                        <th class=TH_SM><span class="sr-only">"Saved"</span></th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    <For
                        each=move || rows.get()
                        key=|s| s.id
                        children={move |s: Arc<Species>| {
                            let row = s.clone();
                            view! {
                                <tr class="hover:bg-primary-50 cursor-pointer" on:click=move |_| ctx.selected.set(Some(row.clone()))>
                                    <td class="px-4 py-2 whitespace-nowrap italic font-medium text-gray-900">{s.scientific_name.clone()}</td>
                                    <td class="px-4 py-2 whitespace-nowrap text-sm text-gray-600">{s.family().to_string()}</td>
                                    <td class="px-4 py-2 whitespace-nowrap text-sm text-gray-600">{model::life_form_label(&s.life_form)}</td>
                                    <td class="px-4 py-2 whitespace-nowrap text-sm text-gray-600">{model::origin_label(&s.origin)}</td>
                                    <td class="px-4 py-2 whitespace-nowrap">{iucn_badge(&s.iucn_status)}</td>
                                    <td class="px-4 py-2 whitespace-nowrap">{group_badge(s.group())}</td>
                                    <td class="px-4 py-2 whitespace-nowrap text-sm text-gray-600">{s.trait_value("HX")}</td>
                                    <td class="px-4 py-2 whitespace-nowrap text-right"><BookmarkButton id=s.id ctx /></td>
                                </tr>
                            }
                        }}
                    />
                </tbody>
            </table>
        </div>
    }
}
