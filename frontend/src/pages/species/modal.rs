//! Species detail sheet (overlay). Closes on backdrop click, the X button or Escape.

use std::sync::Arc;

use leptos::prelude::*;

use super::card::{
    BookmarkButton, NAME_BAR, SCI_BAR, flower_swatch, group_tag, iucn_badge, life_form_icon,
    photo_placeholder, svg,
};
use super::model::{self, Species, TRAITS};
use super::{Ctx, ICON_X};

const ICON_EXTERNAL: &str = "M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14";
const LABEL: &str = "text-xs font-semibold text-gray-500 uppercase tracking-wider";

#[component]
pub(super) fn SpeciesModal(species: Arc<Species>, ctx: Ctx) -> impl IntoView {
    let close = move || ctx.selected.set(None);
    let handle = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            close();
        }
    });
    on_cleanup(move || handle.remove());

    let s = species;
    let (_, _, iucn_desc) = model::iucn_info(&s.iucn_status);
    let accepted = s
        .accepted_scientific_name
        .clone()
        .filter(|a| !a.is_empty() && *a != s.scientific_name);
    let common = s.common_name().map(str::to_string);

    let fact = |label: &'static str, value: AnyView| {
        view! {
            <div>
                <dt class=LABEL>{label}</dt>
                <dd class="mt-0.5 text-sm text-gray-900">{value}</dd>
            </div>
        }
    };

    let link = |href: Option<String>, label: &'static str| {
        href.filter(|h| !h.is_empty()).map(|h| {
            view! {
                <a href=h target="_blank" rel="noopener noreferrer"
                    class=format!("inline-flex items-center gap-1.5 px-4 py-2 rounded-lg {SCI_BAR} text-white text-sm font-medium hover:opacity-90")>
                    {label}
                    {svg(ICON_EXTERNAL, "h-4 w-4")}
                </a>
            }
        })
    };

    view! {
        <div class="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4" on:click=move |_| close()>
            <div
                class="bg-white rounded-lg shadow-xl w-full max-w-3xl max-h-[90vh] overflow-y-auto p-5 space-y-4"
                role="dialog"
                aria-modal="true"
                on:click=|ev| ev.stop_propagation()
            >
                // Common name bar (template header) + actions
                <div class="flex items-center gap-2">
                    <div class=format!("{NAME_BAR} flex-1 min-w-0 rounded flex items-center gap-2 pl-3")>
                        <h2 class="flex-1 min-w-0 text-center text-2xl font-bold text-gray-900 truncate py-1">
                            {common.unwrap_or_else(|| "—".into())}
                        </h2>
                        {group_tag(s.group())}
                    </div>
                    <BookmarkButton id=s.id ctx />
                    <button class="p-1 rounded hover:bg-black/10 text-gray-700" title="Close" on:click=move |_| close()>
                        {svg(ICON_X, "h-6 w-6")}
                    </button>
                </div>

                <div class="grid gap-4 sm:grid-cols-[2fr_3fr]">
                    <div class="bg-[#DFE1E6] h-48 sm:h-auto flex items-center justify-center">
                        {photo_placeholder()}
                    </div>
                    <div class="space-y-3">
                        <div class=format!("{SCI_BAR} rounded px-3 py-1.5 text-white")>
                            <h3 class="text-xl font-semibold italic">{s.scientific_name.clone()}</h3>
                            {accepted.map(|a| view! { <p class="text-sm text-gray-200">"Accepted name: " <i>{a}</i></p> })}
                        </div>
                        <dl class="grid grid-cols-2 gap-3">
                            {fact("Family", s.family().to_string().into_any())}
                            {fact("Genus", view! { <i>{s.genus.name.clone()}</i> }.into_any())}
                            {fact("Habit", view! {
                                <span class="inline-flex items-center gap-1">
                                    {life_form_icon(&s.life_form, "h-6 w-6")}
                                    {model::life_form_label(&s.life_form)}
                                </span>
                            }.into_any())}
                            {fact("Origin", model::origin_label(&s.origin).into_any())}
                            {fact("Canopy shape", model::canopy_label(&s.canopy_shape).into_any())}
                            {fact("Flower color", flower_swatch(&s.flower_color).into_any())}
                        </dl>
                    </div>
                </div>

                <div class="rounded-lg border border-gray-200 p-4">
                    <div class="flex items-center gap-2 mb-1">
                        <span class=LABEL>"IUCN Red List"</span>
                        {iucn_badge(&s.iucn_status)}
                    </div>
                    <p class="text-sm text-gray-700">{iucn_desc}</p>
                </div>

                <div>
                    <h4 class=format!("{LABEL} mb-2")>
                        {match s.group() {
                            Some(g) => format!("Functional group {g} — traits"),
                            None => "Functional-group traits".to_string(),
                        }}
                    </h4>
                    {if s.functional_group.is_some() {
                        view! {
                            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                                {TRAITS.iter().map(|(ty, label)| view! {
                                    <div class="rounded-lg border border-gray-200 px-3 py-2">
                                        <div class="text-xs text-gray-500">{*label}</div>
                                        <div class="text-lg font-semibold text-gray-900">{s.trait_value(ty)}</div>
                                    </div>
                                }).collect_view()}
                            </div>
                            <p class="mt-2 text-xs text-gray-400">"Ranges are the values recorded for the species' functional group."</p>
                        }
                        .into_any()
                    } else {
                        view! { <p class="text-sm text-gray-400 italic">"This species has no functional group assigned yet."</p> }.into_any()
                    }}
                </div>

                <div class="flex flex-wrap items-center justify-between gap-3 border-t border-gray-100 pt-4">
                    <div class="flex flex-wrap gap-2">
                        {link(s.gbif_url.clone(), "GBIF")}
                        {link(s.tropical_plants_url.clone(), "Tropical Plants")}
                    </div>
                    <span class="text-xs text-gray-400">
                        {s.identified_by.clone().filter(|v| !v.is_empty()).map(|v| format!("Identified by {v}"))}
                    </span>
                </div>
            </div>
        </div>
    }
}
