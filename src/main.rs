#![cfg(target_arch = "wasm32")]

use chemquiz::{
    CatalogLeaf, CatalogManifest, Compound, DEMO_OPTION_COUNT, QuizItem, QuizMode, demo_compounds,
    generate_quiz,
};
use gloo_net::http::Request;
use leptos::*;
use rand::SeedableRng;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct CompoundList {
    compounds: Vec<Compound>,
}

fn set_body_theme(theme: &str) {
    if let Some(document) = leptos::window().document() {
        if let Some(body) = document.body() {
            let _ = body.set_attribute("data-theme", theme);
        }
    }
}

async fn fetch_manifest() -> Result<CatalogManifest, String> {
    Request::get("catalog/index.json")
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json::<CatalogManifest>()
        .await
        .map_err(|error| error.to_string())
}

async fn fetch_compound_file(path: &str) -> Result<Vec<Compound>, String> {
    Request::get(path)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json::<CompoundList>()
        .await
        .map(|list| list.compounds)
        .map_err(|error| error.to_string())
}

fn format_path(path: &[String]) -> String {
    if path.is_empty() {
        "Not selected".to_string()
    } else {
        path.join(" / ")
    }
}

fn generate_from_dataset(dataset: &[Compound]) -> Result<QuizItem, String> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    generate_quiz(
        &mut rng,
        dataset,
        QuizMode::NameToStructure,
        DEMO_OPTION_COUNT,
    )
    .map_err(|error| error.to_string())
}

#[component]
fn CatalogMenu(manifest: CatalogManifest, on_select: Callback<CatalogLeaf>) -> impl IntoView {
    let mut leaves = manifest.leaves();
    leaves.sort_by(|a, b| a.path.cmp(&b.path));

    view! {
        <ul class="catalog-menu">
            {leaves
                .into_iter()
                .map(|leaf| {
                    let label = format_path(&leaf.path);
                    let callback = on_select.clone();
                    let leaf_data = leaf.clone();

                    view! {
                        <li>
                            <button class="pill" on:click=move |_| callback.call(leaf_data.clone())>
                                {label.clone()}
                            </button>
                        </li>
                    }
                })
                .collect_view::<Vec<_>>()}
        </ul>
    }
}

#[component]
fn QuizCard(quiz: QuizItem) -> impl IntoView {
    view! {
        <section class="quiz-card">
            <div class="prompt-area">
                <p class="eyebrow">Prompt</p>
                <p class="prompt">{quiz.prompt}</p>
            </div>
            <div class="options-grid">
                {quiz
                    .options
                    .iter()
                    .enumerate()
                    .map(|(index, option)| {
                        let status = if index == quiz.correct_index {
                            "correct"
                        } else {
                            "distractor"
                        };

                        view! {
                            <button class=format!("option {}", status) aria-pressed={(index == quiz.correct_index).to_string()}>
                                <span class="option-index">{(index + 1).to_string()}</span>
                                <span class="option-body">{option.clone()}</span>
                            </button>
                        }
                    })
                    .collect_view::<Vec<_>>()}
            </div>
        </section>
    }
}

#[component]
fn App() -> impl IntoView {
    let (theme, set_theme) = create_signal(String::from("dark"));
    let (quiz, set_quiz) = create_signal::<Option<QuizItem>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (selected_leaf, set_selected_leaf) = create_signal::<Option<CatalogLeaf>>(None);
    let (compounds, set_compounds) = create_signal::<Option<Vec<Compound>>>(None);

    let manifest = create_resource(|| (), |_| async { fetch_manifest().await });

    create_effect(move |_| {
        set_body_theme(&theme.get());
    });

    let regenerate = move |_| {
        let dataset = compounds.get().unwrap_or_else(demo_compounds);
        match generate_from_dataset(&dataset) {
            Ok(item) => {
                set_error(None);
                set_quiz(Some(item));
            }
            Err(message) => {
                set_quiz(None);
                set_error(Some(message));
            }
        }
    };

    let toggle_theme = move |_| {
        let next = if theme.get() == "dark" {
            "light"
        } else {
            "dark"
        };
        set_theme(String::from(next));
    };

    let handle_selection = Callback::new(move |leaf: CatalogLeaf| {
        set_selected_leaf(Some(leaf.clone()));
        set_error(None);
        set_quiz(None);
        set_compounds(None);

        let setter = set_compounds.clone();
        let error_setter = set_error.clone();
        spawn_local(async move {
            match fetch_compound_file(&leaf.file).await {
                Ok(list) => setter(Some(list)),
                Err(message) => error_setter(Some(message)),
            }
        });
    });

    view! {
        <main class="page">
            <header class="page-header">
                <div>
                    <p class="eyebrow">Chemistry Quiz Sandbox</p>
                    <h1 class="headline">Wasm-ready Leptos frontend</h1>
                    <p class="lede">Interactively preview the quiz core packaged for GitHub Pages.</p>
                </div>
                <div class="header-actions">
                    <button class="pill" on:click=toggle_theme>
                        {move || if theme.get() == "dark" { "Switch to light" } else { "Switch to dark" }}
                    </button>
                    <button class="primary" on:click=regenerate>
                        "Generate quiz"
                    </button>
                </div>
            </header>

            <section class="status-panel">
                <div class="status-item">
                    <p class="label">WASM status</p>
                    <p class="value">Loaded via Trunk for Pages</p>
                </div>
                <div class="status-item">
                    <p class="label">Dataset</p>
                    <p class="value">Select a catalog entry to load JSON compounds</p>
                </div>
                <div class="status-item">
                    <p class="label">Deployment path</p>
                    <p class="value">Public URL: /chemquiz/</p>
                </div>
            </section>

            <section class="status-panel">
                <div class="status-item">
                    <p class="label">Selected path</p>
                    <p class="value">{move || selected_leaf.get().map(|leaf| format_path(&leaf.path)).unwrap_or_else(|| "Not selected".to_string())}</p>
                </div>
                <div class="status-item">
                    <p class="label">Loaded compounds</p>
                    <p class="value">{move || compounds.get().as_ref().map(|items| items.len().to_string()).unwrap_or_else(|| "0".to_string())}</p>
                </div>
                <div class="status-item">
                    <p class="label">Options per quiz</p>
                    <p class="value">{DEMO_OPTION_COUNT.to_string()}</p>
                </div>
            </section>

            <section class="dataset-panel">
                <div class="panel-header">
                    <p class="eyebrow">Catalog</p>
                    <h2 class="headline">Browse compound folders</h2>
                    <p class="lede">Files live in the public catalog directory and load on demand via serde deserialization.</p>
                </div>
                {move || match manifest.read() {
                    Some(Ok(listing)) => view! { <CatalogMenu manifest=listing.clone() on_select=handle_selection.clone() /> },
                    Some(Err(message)) => view! { <p class="error-body">{message}</p> },
                    None => view! { <p class="lede">Loading catalog index...</p> },
                }}
            </section>

            {move || {
                if let Some(item) = quiz.get() {
                    view! { <QuizCard quiz=item /> }
                } else if let Some(message) = error.get() {
                    view! {
                        <section class="error-card">
                            <p class="eyebrow">Generator error</p>
                            <p class="error-body">{message}</p>
                        </section>
                    }
                } else {
                    view! {
                        <section class="placeholder-card">
                            <p class="eyebrow">Awaiting prompt</p>
                            <p class="lede">Load a catalog entry and generate a sample quiz.</p>
                        </section>
                    }
                }
            }}
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
