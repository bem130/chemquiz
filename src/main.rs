#![cfg(target_arch = "wasm32")]

use chemquiz::{DEMO_OPTION_COUNT, QuizItem, QuizMode, demo_compounds, generate_quiz};
use leptos::*;
use rand::SeedableRng;
use wasm_bindgen::prelude::wasm_bindgen;

fn set_body_theme(theme: &str) {
    if let Some(document) = leptos::window().document() {
        if let Some(body) = document.body() {
            let _ = body.set_attribute("data-theme", theme);
        }
    }
}

fn demo_quiz() -> Result<QuizItem, String> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    generate_quiz(
        &mut rng,
        &demo_compounds(),
        QuizMode::NameToStructure,
        DEMO_OPTION_COUNT,
    )
    .map_err(|error| error.to_string())
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

    create_effect(move |_| {
        set_body_theme(&theme.get());
    });

    let regenerate = move |_| match demo_quiz() {
        Ok(item) => {
            set_error(None);
            set_quiz(Some(item));
        }
        Err(message) => {
            set_quiz(None);
            set_error(Some(message));
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
                        "Generate demo quiz"
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
                    <p class="value">Six curated compounds with bilingual labels</p>
                </div>
                <div class="status-item">
                    <p class="label">Deployment path</p>
                    <p class="value">Public URL: /chemquiz/</p>
                </div>
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
                            <p class="lede">Generate a sample quiz to verify wasm bindings and styling.</p>
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
