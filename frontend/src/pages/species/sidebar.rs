//! Left filter panel: text searches, functional-group range and multi-select categories.

use leptos::prelude::*;

use super::Ctx;
use super::model::{self, CANOPIES, Catalog, Category, GROUP_MAX, GROUP_MIN, IUCN, LIFE_FORMS, ORIGINS};

const INPUT: &str = "w-full border border-gray-300 rounded-lg py-1.5 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary-500";

/// Options of a category: (code, label, optional color swatch classes).
fn options(cat: Category) -> Vec<(&'static str, &'static str, Option<&'static str>)> {
    match cat {
        Category::FlowerColor => model::FLOWER_COLORS.iter().map(|(c, l, s)| (*c, *l, Some(*s))).collect(),
        Category::LifeForm => LIFE_FORMS.iter().map(|(c, l)| (*c, *l, None)).collect(),
        Category::Iucn => IUCN.iter().map(|(c, l, ..)| (*c, *l, None)).collect(),
        Category::Origin => ORIGINS.iter().map(|(c, l)| (*c, *l, None)).collect(),
        Category::Canopy => CANOPIES.iter().map(|(c, l)| (*c, *l, None)).collect(),
    }
}

const SECTIONS: [(Category, &str, bool); 5] = [
    (Category::LifeForm, "Habit", true),
    (Category::Iucn, "Conservation status (IUCN)", true),
    (Category::Origin, "Origin", true),
    (Category::FlowerColor, "Flower color", false),
    (Category::Canopy, "Canopy shape", false),
];

#[component]
pub(super) fn FilterSidebar(catalog: Catalog, ctx: Ctx) -> impl IntoView {
    let f = ctx.filters;

    let group_select = move |is_min: bool| {
        view! {
            <select
                class="flex-1 border border-gray-300 rounded-lg py-1.5 px-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-primary-500"
                prop:value=move || f.with(|f| if is_min { f.group_min } else { f.group_max }.to_string())
                on:change=move |ev| {
                    let v: u8 = event_target_value(&ev).parse().unwrap_or(if is_min { GROUP_MIN } else { GROUP_MAX });
                    f.update(|f| {
                        if is_min {
                            f.group_min = v;
                            f.group_max = f.group_max.max(v);
                        } else {
                            f.group_max = v;
                            f.group_min = f.group_min.min(v);
                        }
                    });
                }
            >
                {(GROUP_MIN..=GROUP_MAX).map(|g| view! { <option value=g.to_string()>{g.to_string()}</option> }).collect_view()}
            </select>
        }
    };

    let sections = SECTIONS
        .into_iter()
        .map(|(cat, title, open)| {
            let items = options(cat)
                .into_iter()
                .map(|(code, label, swatch)| {
                    let count = catalog.iter().filter(|s| cat.value(s) == code).count();
                    view! {
                        <label class="flex items-center gap-2 py-0.5 text-sm text-gray-700 cursor-pointer hover:text-primary-700">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300 accent-primary-600"
                                prop:checked=move || f.with(|f| f.is_selected(cat, code))
                                on:change=move |_| f.update(|f| f.toggle(cat, code))
                            />
                            {swatch.map(|s| view! { <span class=format!("h-3 w-3 rounded-full shrink-0 {s}")></span> })}
                            <span class="flex-1 truncate">{label}</span>
                            <span class="text-xs text-gray-400">{count}</span>
                        </label>
                    }
                })
                .collect_view();
            view! {
                <details class="border-t border-gray-100 pt-3" open=open>
                    <summary class="text-xs font-semibold text-gray-500 uppercase tracking-wider cursor-pointer select-none mb-1">
                        {title}
                    </summary>
                    <div class="mt-1">{items}</div>
                </details>
            }
        })
        .collect_view();

    view! {
        <aside class="w-64 shrink-0 bg-white rounded-lg shadow p-4 overflow-y-auto space-y-3">
            <h2 class="text-sm font-semibold text-gray-800">"Filters"</h2>

            <div class="space-y-1">
                <label class="text-xs font-semibold text-gray-500 uppercase tracking-wider">"Scientific name"</label>
                <input
                    type="text"
                    class=INPUT
                    placeholder="e.g. Ficus"
                    prop:value=move || f.with(|f| f.scientific_name.clone())
                    on:input=move |ev| f.update(|f| f.scientific_name = event_target_value(&ev))
                />
            </div>

            <div class="space-y-1">
                <label class="text-xs font-semibold text-gray-500 uppercase tracking-wider">"Family or genus"</label>
                <input
                    type="text"
                    class=INPUT
                    placeholder="e.g. Fabaceae"
                    prop:value=move || f.with(|f| f.family.clone())
                    on:input=move |ev| f.update(|f| f.family = event_target_value(&ev))
                />
            </div>

            <div class="space-y-1 border-t border-gray-100 pt-3">
                <label class="text-xs font-semibold text-gray-500 uppercase tracking-wider">"Functional group"</label>
                <div class="flex items-center gap-2">
                    {group_select(true)}
                    <span class="text-gray-400">"–"</span>
                    {group_select(false)}
                </div>
                <p class="text-xs text-gray-400">"Narrowing the range hides species without a group."</p>
            </div>

            {sections}
        </aside>
    }
}
