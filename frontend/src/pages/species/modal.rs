//! Species detail sheet (overlay). Closes on backdrop click, the X button or Escape.

use std::sync::Arc;

use leptos::prelude::*;

use super::card::{BookmarkButton, flower_swatch, group_badge, iucn_badge, life_form_icon, svg};
use super::model::{self, Species, TRAITS};
use super::{Ctx, ICON_X};

const ICON_EXTERNAL: &str = "M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14";

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

    let fact = |label: &'static str, value: AnyView| {
        view! {
            <div>
                <dt class="text-xs font-semibold text-gray-500 uppercase tracking-wider">{label}</dt>
                <dd class="mt-0.5 text-sm text-gray-800">{value}</dd>
            </div>
        }
    };

    let link = |href: Option<String>, label: &'static str| {
        href.filter(|h| !h.is_empty()).map(|h| {
            view! {
                <a href=h target="_blank" rel="noopener noreferrer"
                    class="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg border border-primary-600 text-primary-700 text-sm font-medium hover:bg-primary-50">
                    {label}
                    {svg(ICON_EXTERNAL, "h-4 w-4")}
                </a>
            }
        })
    };

    view! {
        <div class="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4" on:click=move |_| close()>
            <div
                class="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto"
                role="dialog"
                aria-modal="true"
                on:click=|ev| ev.stop_propagation()
            >
                <div class="bg-primary-600 text-white px-6 py-4 rounded-t-lg flex items-start justify-between gap-4">
                    <div class="flex items-center gap-4 min-w-0">
                        <div class="h-14 w-14 shrink-0 rounded-full bg-white/15 flex items-center justify-center">
                            {life_form_icon(&s.life_form, "h-8 w-8")}
                        </div>
                        <div class="min-w-0">
                            <h2 class="text-xl font-semibold italic">{s.scientific_name.clone()}</h2>
                            {accepted.map(|a| view! { <p class="text-sm text-primary-100">"Accepted name: " <i>{a}</i></p> })}
                            <p class="text-sm text-primary-100">{format!("{} · {}", s.family(), s.genus.name)}</p>
                        </div>
                    </div>
                    <div class="flex items-center gap-1 shrink-0">
                        <BookmarkButton id=s.id ctx color="text-white" />
                        <button class="p-1 rounded hover:bg-black/10" title="Close" on:click=move |_| close()>
                            {svg(ICON_X, "h-6 w-6")}
                        </button>
                    </div>
                </div>

                <div class="p-6 space-y-6">
                    <dl class="grid grid-cols-2 sm:grid-cols-3 gap-4">
                        {fact("Habit", model::life_form_label(&s.life_form).into_any())}
                        {fact("Origin", model::origin_label(&s.origin).into_any())}
                        {fact("Canopy shape", model::canopy_label(&s.canopy_shape).into_any())}
                        {fact("Flower color", flower_swatch(&s.flower_color).into_any())}
                        {fact("Functional group", group_badge(s.group()).into_any())}
                        {fact("Identified by", s.identified_by.clone().filter(|v| !v.is_empty()).unwrap_or_else(|| "—".into()).into_any())}
                    </dl>

                    <div class="rounded-lg border border-gray-200 p-4">
                        <div class="flex items-center gap-2 mb-1">
                            <span class="text-xs font-semibold text-gray-500 uppercase tracking-wider">"IUCN Red List"</span>
                            {iucn_badge(&s.iucn_status)}
                        </div>
                        <p class="text-sm text-gray-600">{iucn_desc}</p>
                    </div>

                    <div>
                        <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">"Functional-group traits"</h3>
                        {if s.functional_group.is_some() {
                            view! {
                                <div class="grid grid-cols-2 gap-3">
                                    {TRAITS.iter().map(|(ty, label)| view! {
                                        <div class="rounded-lg bg-primary-50 px-4 py-3">
                                            <div class="text-xs text-gray-500">{*label}</div>
                                            <div class="text-lg font-semibold text-primary-900">{s.trait_value(ty)}</div>
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

                    <div class="flex flex-wrap gap-2">
                        {link(s.gbif_url.clone(), "GBIF")}
                        {link(s.tropical_plants_url.clone(), "Tropical Plants")}
                    </div>
                </div>
            </div>
        </div>
    }
}
