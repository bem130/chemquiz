#![cfg(target_arch = "wasm32")]

use crate::{
    CatalogLeaf, CatalogManifest, CatalogNode, Compound, DEMO_OPTION_COUNT, QuizItem, QuizMode,
    demo_compounds, generate_quiz,
};
use gloo_net::http::Request;
use js_sys::Reflect;
use leptos::{html, *};
use rand::SeedableRng;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlDivElement, HtmlElement};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct CompoundList {
    compounds: Vec<Compound>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scene {
    Menu,
    Game,
    Result,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct SessionScore {
    total: usize,
    correct: usize,
}

#[derive(Clone)]
struct ResultView {
    quiz: QuizItem,
    dataset: Vec<Compound>,
    selected: usize,
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

fn find_by_name(dataset: &[Compound], label: &str) -> Option<Compound> {
    dataset
        .iter()
        .find(|compound| compound.display_name() == label)
        .cloned()
}

fn find_by_structure(dataset: &[Compound], label: &str) -> Option<Compound> {
    dataset
        .iter()
        .find(|compound| compound.display_structure() == label)
        .cloned()
}

fn katex_render_available() -> Option<js_sys::Function> {
    let global = js_sys::global();
    let katex = Reflect::get(&global, &JsValue::from_str("katex")).ok()?;
    if katex.is_undefined() {
        return None;
    }

    Reflect::get(&katex, &JsValue::from_str("render"))
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
}

fn render_formula_into(element: HtmlElement, formula: &str) {
    if let Some(render) = katex_render_available() {
        let content = format!("\\ce{{{}}}", formula);
        let options = js_sys::Object::new();
        let _ = Reflect::set(
            &options,
            &JsValue::from_str("throwOnError"),
            &JsValue::FALSE,
        );

        if render
            .call3(
                &js_sys::global(),
                &JsValue::from_str(&content),
                &element.clone().into(),
                &options,
            )
            .is_ok()
        {
            return;
        }
    }

    element.set_text_content(Some(formula));
}

#[wasm_bindgen(inline_js = r#"
let rdkitModulePromise = null;

function waitForRdkit() {
    if (!rdkitModulePromise) {
        rdkitModulePromise = new Promise((resolve, reject) => {
            let attempts = 0;
            const check = () => {
                if (typeof initRDKitModule === 'function') {
                    initRDKitModule().then(resolve).catch(reject);
                } else if (attempts < 50) {
                    attempts += 1;
                    setTimeout(check, 40);
                } else {
                    reject(new Error('RDKit.js not loaded'));
                }
            };
            check();
        });
    }

    return rdkitModulePromise;
}

function ensureKekule() {
    if (typeof Kekule === 'undefined' || !Kekule.IO || !Kekule.Render) {
        return Promise.reject(new Error('Kekule.js not loaded'));
    }
    return Promise.resolve(Kekule);
}

function copySkeletalCanvas(offscreen, target, theme) {
    const width = offscreen.width;
    const height = offscreen.height;
    const rawContext = offscreen.getContext('2d');
    const visibleContext = target.getContext('2d');
    const img = rawContext.getImageData(0, 0, width, height);

    if (theme === 'dark') {
        const data = img.data;
        for (let i = 0; i < data.length; i += 4) {
            const r = data[i];
            const g = data[i + 1];
            const b = data[i + 2];

            if (r > 245 && g > 245 && b > 245) {
                data[i + 3] = 0;
                continue;
            }

            const maxRGB = Math.max(r, g, b);
            const minRGB = Math.min(r, g, b);
            const isNearGray = (maxRGB - minRGB) < 10;

            if (maxRGB < 60 && isNearGray) {
                data[i] = 255;
                data[i + 1] = 255;
                data[i + 2] = 255;
            }
        }

        visibleContext.clearRect(0, 0, width, height);
        visibleContext.putImageData(img, 0, 0);
    } else {
        visibleContext.fillStyle = '#ffffff';
        visibleContext.fillRect(0, 0, width, height);
        visibleContext.putImageData(img, 0, 0);
    }
}

function buildKekuleViewer(container, width, height, theme) {
    const V = Kekule.ChemWidget;
    const R = Kekule.Render;

    while (container.firstChild) {
        container.removeChild(container.firstChild);
    }

    const viewer = new V.Viewer(container);
    viewer.setRenderType(R.RendererType.R2D);
    viewer.setMoleculeDisplayType(R.Molecule2DDisplayType.CONDENSED);
    viewer.setEnableToolbar(false);
    viewer.setEnableDirectInteraction(false);
    viewer.setEnableEdit(false);
    viewer.setAutoSize(false);
    viewer.setAutofit(true);
    viewer.setPadding(12);
    viewer.setDimension(width, height);

    const moleculeConfigs = viewer.getRenderConfigs().getMoleculeDisplayConfigs();
    moleculeConfigs.setDefHydrogenDisplayLevel(R.HydrogenDisplayLevel.ALL);
    moleculeConfigs.setDefNodeDisplayMode(R.NodeLabelDisplayMode.SHOWN);

    const colorConfigs = viewer.getRenderConfigs().getColorConfigs();
    if (theme === 'dark') {
        colorConfigs.setAtomColor('#ffffff');
        colorConfigs.setBondColor('#ffffff');
        colorConfigs.setLabelColor('#ffffff');
    } else {
        colorConfigs.setAtomColor('#000000');
        colorConfigs.setBondColor('#000000');
        colorConfigs.setLabelColor('#000000');
    }
    viewer.setBackgroundColor('transparent');

    return viewer;
}

export async function renderStructure(smiles, theme, skeletalCanvas, fullDiv) {
    const [rdkit] = await Promise.all([waitForRdkit(), ensureKekule()]);
    let molecule = null;

    try {
        molecule = rdkit.get_mol(smiles);
    } catch (err) {
        return { ok: false, message: `RDKit get_mol failed: ${err}` };
    }

    const width = skeletalCanvas.clientWidth || (skeletalCanvas.parentElement && skeletalCanvas.parentElement.clientWidth) || 260;
    const height = skeletalCanvas.clientHeight || 190;
    skeletalCanvas.width = width;
    skeletalCanvas.height = height;

    const offscreen = document.createElement('canvas');
    offscreen.width = width;
    offscreen.height = height;

    try {
        molecule.draw_to_canvas(offscreen, -1, -1);
    } catch (err) {
        molecule.delete();
        return { ok: false, message: `RDKit draw_to_canvas failed: ${err}` };
    }

    copySkeletalCanvas(offscreen, skeletalCanvas, theme);

    let molblock = '';
    try {
        molblock = molecule.get_molblock();
    } catch (err) {
        molblock = '';
    }
    molecule.delete();

    if (molblock && fullDiv) {
        const targetWidth = fullDiv.clientWidth || width;
        const targetHeight = fullDiv.clientHeight || height;
        const viewer = buildKekuleViewer(fullDiv, targetWidth, targetHeight, theme);

        try {
            const molObj = Kekule.IO.loadFormatData(molblock, 'mol');
            if (!molObj) {
                return { ok: false, message: 'Kekule failed to load Molfile' };
            }
            viewer.setChemObj(molObj);
            viewer.requestRepaint();
        } catch (err) {
            return { ok: false, message: `Kekule render failed: ${err}` };
        }
    }

    return { ok: true };
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = renderStructure)]
    async fn render_structure(
        smiles: &str,
        theme: &str,
        skeletal: HtmlCanvasElement,
        full: HtmlDivElement,
    ) -> JsValue;
}

fn render_error_from(result: JsValue) -> Option<String> {
    match Reflect::get(&result, &JsValue::from_str("ok")) {
        Ok(flag) if flag.as_bool().unwrap_or(false) => None,
        _ => Reflect::get(&result, &JsValue::from_str("message"))
            .ok()
            .and_then(|message| message.as_string()),
    }
}

fn generate_from_dataset(dataset: &[Compound], mode: QuizMode) -> Result<QuizItem, String> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    generate_quiz(&mut rng, dataset, mode, DEMO_OPTION_COUNT).map_err(|error| error.to_string())
}

#[component]
fn NameTile(compound: Compound) -> impl IntoView {
    view! {
        <div class="name-tile">
            <p class="iupac">{compound.iupac_name}</p>
            {compound
                .common_name
                .as_ref()
                .map(|common| view! { <p class="common-name">{common.clone()}</p> })}
            {compound
                .local_name
                .as_ref()
                .map(|local| view! { <p class="local-name">{local.clone()}</p> })}
        </div>
    }
}

#[component]
fn FormulaBadge(formula: String) -> impl IntoView {
    let node_ref = create_node_ref::<html::Div>();

    create_effect(move |_| {
        if let Some(element) = node_ref.get() {
            // Clone the underlying HtmlDivElement, then cast the clone to web_sys::HtmlElement
            render_formula_into(
                <HtmlDivElement as Clone>::clone(&element).unchecked_into::<HtmlElement>(),
                &formula,
            );
        }
    });

    view! { <div class="formula-badge" node_ref=node_ref></div> }
}

#[component]
fn StructureTile(compound: Compound, theme: ReadSignal<String>) -> impl IntoView {
    let skeletal_ref = create_node_ref::<html::Canvas>();
    let full_ref = create_node_ref::<html::Div>();
    let (render_message, set_render_message) = create_signal::<Option<String>>(None);

    let smiles = compound.smiles.clone();
    let effect_smiles = smiles.clone();
    let iupac_name = compound.iupac_name.clone();
    let skeletal_formula = compound.skeletal_formula.clone();
    let molecular_formula = compound.molecular_formula.clone();

    let badge = if molecular_formula.is_empty() {
        None
    } else {
        Some(view! { <FormulaBadge formula=molecular_formula.clone() /> })
    };

    create_effect(move |_| {
        let current_theme = theme.get();
        set_render_message.set(None);

        if let (Some(smiles_value), Some(canvas), Some(full)) =
            (effect_smiles.clone(), skeletal_ref.get(), full_ref.get())
        {
            let status = set_render_message;
            let skeletal_element: HtmlCanvasElement = (*canvas).clone().unchecked_into();
            let full_element: HtmlDivElement = (*full).clone().unchecked_into();

            spawn_local(async move {
                let result = render_structure(
                    &smiles_value,
                    &current_theme,
                    skeletal_element,
                    full_element,
                )
                .await;
                status.set(render_error_from(result));
            });
        }
    });

    let visuals = smiles.map(|_| {
        view! {
            <div class="structure-visual">
                <div class="structure-frame with-badge">
                    <p class="structure-label">Skeletal formula</p>
                    <canvas
                        node_ref=skeletal_ref
                        class="structure-canvas"
                        role="img"
                        aria-label=format!("Skeletal depiction for {}", iupac_name.clone())
                    ></canvas>
                    {badge.clone().unwrap_or_default()}
                </div>
                <div class="structure-frame">
                    <p class="structure-label">Structural formula</p>
                    <div
                        node_ref=full_ref
                        class="full-structure"
                        role="img"
                        aria-label=format!("Full structural formula for {}", iupac_name.clone())
                    ></div>
                </div>
            </div>
        }
        .into_view()
    });

    view! {
        <div class="structure-tile">
            {visuals.unwrap_or_else(|| {
                view! { <p class="structure-fallback">Structure preview is unavailable for this entry.</p> }
                    .into_view()
            })}
            {move || {
                render_message
                    .get()
                    .map(|message| view! { <p class="structure-fallback">{message}</p> })
            }}
            <div class="structure-meta">
                <p class="skeletal">{format!("Skeletal: {}", skeletal_formula.clone())}</p>
                <p class="molecular">{format!("Molecular: {}", molecular_formula.clone())}</p>
            </div>
        </div>
    }
}

#[component]
fn CatalogTreeNode(
    node: CatalogNode,
    prefix: Vec<String>,
    on_select: Callback<CatalogLeaf>,
) -> impl IntoView {
    let mut children = node.children.clone();
    children.sort_by(|left, right| left.label.cmp(&right.label));

    let mut path = prefix.clone();
    path.push(node.label.clone());

    let trigger = node.file.as_ref().map(|file| {
        let leaf = CatalogLeaf {
            path: path.clone(),
            file: file.clone(),
        };
        let callback = on_select.clone();

        view! {
            <button class="pill" on:click=move |_| callback.call(leaf.clone())>
                "Select"
            </button>
        }
    });

    view! {
        <li class="tree-item">
            <div class="tree-row">
                <span class="tree-label">{node.label.clone()}</span>
                {trigger}
            </div>
            {(!children.is_empty())
                .then(|| {
                    view! {
                        <ul class="tree-children">
                            {children
                                .into_iter()
                                .map(|child| {
                                    view! { <CatalogTreeNode node=child prefix=path.clone() on_select=on_select.clone() /> }
                                })
                                .collect_view()}
                        </ul>
                    }
                })}
        </li>
    }
}

#[component]
fn CatalogTree(manifest: CatalogManifest, on_select: Callback<CatalogLeaf>) -> impl IntoView {
    let mut roots = manifest.roots.clone();
    roots.sort_by(|left, right| left.label.cmp(&right.label));

    view! {
        <ul class="catalog-tree">
            {roots
                .into_iter()
                .map(|root| {
                    view! { <CatalogTreeNode node=root prefix=Vec::new() on_select=on_select.clone() /> }
                })
                .collect_view()}
        </ul>
    }
}

#[component]
fn QuizCard(
    quiz: QuizItem,
    dataset: Vec<Compound>,
    theme: ReadSignal<String>,
    selected: Option<usize>,
    reveal: bool,
    on_select: Option<Callback<usize>>,
) -> impl IntoView {
    let mode_label = match quiz.mode {
        QuizMode::NameToStructure => "Name → Structure",
        QuizMode::StructureToName => "Structure → Name",
    };

    let prompt_compound = match quiz.mode {
        QuizMode::NameToStructure => find_by_name(&dataset, &quiz.prompt),
        QuizMode::StructureToName => find_by_structure(&dataset, &quiz.prompt),
    };

    view! {
        <section class="quiz-card">
            <div class="quiz-card-header">
                <div>
                    <p class="eyebrow">Quiz mode</p>
                    <p class="lede">{mode_label}</p>
                </div>
                <div class="pill muted">
                    {format!("{} options", quiz.options.len())}
                </div>
            </div>

            <div class="quiz-layout">
                <div class="prompt-card card-surface">
                    <p class="eyebrow">Prompt</p>
                    {prompt_compound
                        .map(|compound| match quiz.mode {
                            QuizMode::NameToStructure => view! { <NameTile compound=compound /> }.into_view(),
                            QuizMode::StructureToName => {
                                view! { <StructureTile compound=compound theme=theme /> }.into_view()
                            }
                        })
                        .unwrap_or_else(|| view! { <p class="prompt">{quiz.prompt.clone()}</p> }.into_view())}
                </div>

                <div class="options-grid">
                    {quiz
                        .options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| {
                            let is_selected = selected == Some(index);
                            let is_correct = index == quiz.correct_index;

                            let mut classes = vec!["option-card".to_string()];
                            if reveal {
                                if is_correct {
                                    classes.push("correct".to_string());
                                } else if is_selected {
                                    classes.push("wrong".to_string());
                                } else {
                                    classes.push("muted".to_string());
                                }
                            } else if is_selected {
                                classes.push("active".to_string());
                            }

                            let compound = match quiz.mode {
                                QuizMode::NameToStructure => find_by_structure(&dataset, option),
                                QuizMode::StructureToName => find_by_name(&dataset, option),
                            };

                            let click_handler = {
                                let on_select = on_select.clone();
                                let idx = index;

                                move |_| {
                                    if let Some(ref callback) = on_select {
                                        callback.call(idx);
                                    }
                                }
                            };

                            view! {
                                <button
                                    class=classes.join(" ")
                                    aria-pressed=is_selected
                                    on:click=click_handler
                                    disabled=reveal || on_select.is_none()
                                >
                                    <span class="option-index">{(index + 1).to_string()}</span>
                                    {compound
                                        .map(|compound| match quiz.mode {
                                            QuizMode::NameToStructure => {
                                                view! { <StructureTile compound=compound theme=theme /> }.into_view()
                                            }
                                            QuizMode::StructureToName => view! { <NameTile compound=compound /> }.into_view(),
                                        })
                                        .unwrap_or_else(|| view! { <p class="option-body">{option.clone()}</p> }.into_view())}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </section>
    }
}

#[component]
fn App() -> impl IntoView {
    let (theme, set_theme) = create_signal(String::from("dark"));
    let (mode, set_mode) = create_signal(QuizMode::NameToStructure);
    let (quiz, set_quiz) = create_signal::<Option<QuizItem>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (selected_leaf, set_selected_leaf) = create_signal::<Option<CatalogLeaf>>(None);
    let (compounds, set_compounds) = create_signal::<Option<Vec<Compound>>>(None);
    let (active_dataset, set_active_dataset) = create_signal::<Vec<Compound>>(demo_compounds());
    let (scene, set_scene) = create_signal(Scene::Menu);
    let (selected_option, set_selected_option) = create_signal::<Option<usize>>(None);
    let (score, set_score) = create_signal(SessionScore::default());
    let (result_view, set_result_view) = create_signal::<Option<ResultView>>(None);

    let manifest = create_resource(|| (), |_| async { fetch_manifest().await });

    create_effect(move |_| set_body_theme(&theme.get()));

    let regenerate = move || -> bool {
        set_selected_option.set(None);
        set_result_view.set(None);

        let dataset = compounds.get().unwrap_or_else(|| active_dataset.get());

        match generate_from_dataset(&dataset, mode.get()) {
            Ok(item) => {
                set_error.set(None);
                set_active_dataset.set(dataset.clone());
                set_quiz.set(Some(item));
                true
            }
            Err(message) => {
                set_quiz.set(None);
                set_error.set(Some(message));
                false
            }
        }
    };

    let toggle_theme = move |_| {
        let next = if theme.get() == "dark" {
            "light"
        } else {
            "dark"
        };
        set_theme.set(String::from(next));
    };

    let start_game = move |_| {
        set_score.set(SessionScore::default());
        if regenerate() {
            set_scene.set(Scene::Game);
        }
    };

    let next_question = move |_| {
        if regenerate() {
            set_scene.set(Scene::Game);
        }
    };

    let return_to_menu = move |_| {
        set_scene.set(Scene::Menu);
    };

    let choose_option = Callback::new(move |index: usize| {
        if scene.get() != Scene::Game || selected_option.get().is_some() {
            return;
        }

        if let Some(item) = quiz.get() {
            set_selected_option.set(Some(index));
            set_score.update(|state| {
                state.total += 1;
                if index == item.correct_index {
                    state.correct += 1;
                }
            });
            set_result_view.set(Some(ResultView {
                quiz: item.clone(),
                dataset: active_dataset.get(),
                selected: index,
            }));
            set_scene.set(Scene::Result);
        }
    });

    let handle_selection = Callback::new(move |leaf: CatalogLeaf| {
        set_selected_leaf.set(Some(leaf.clone()));
        set_error.set(None);
        set_quiz.set(None);
        set_scene.set(Scene::Menu);
        set_compounds.set(None);

        let setter = set_compounds.clone();
        let error_setter = set_error.clone();
        spawn_local(async move {
            match fetch_compound_file(&leaf.file).await {
                Ok(list) => setter.set(Some(list)),
                Err(message) => error_setter.set(Some(message)),
            }
        });
    });

    view! {
        <main class="page">
            <header class="page-header">
                <div>
                    <p class="eyebrow">Chemistry practice</p>
                    <h1 class="headline">{"Name ⇄ Structure quiz"}</h1>
                    <p class="lede">Pick a catalog folder and generate four-choice prompts for IUPAC names and skeletal formulas.</p>
                </div>
                <div class="header-actions">
                    <button class="pill" on:click=toggle_theme>
                        {move || if theme.get() == "dark" { "Switch to light" } else { "Switch to dark" }}
                    </button>
                    <div class="pill muted">
                        {move || format!("Score {} / {}", score.get().correct, score.get().total)}
                    </div>
                </div>
            </header>
            <Show when=move || scene.get() == Scene::Menu>
                <section class="control-panel">
                    <div class="control-group">
                        <p class="label">Quiz mode</p>
                        <div class="segmented">
                            <button
                                class:active=move || mode.get() == QuizMode::NameToStructure
                                on:click=move |_| set_mode.set(QuizMode::NameToStructure)
                            >
                                "Name → Structure"
                            </button>
                            <button
                                class:active=move || mode.get() == QuizMode::StructureToName
                                on:click=move |_| set_mode.set(QuizMode::StructureToName)
                            >
                                "Structure → Name"
                            </button>
                        </div>
                    </div>
                    <div class="status-group">
                        <div>
                            <p class="label">Selected path</p>
                            <p class="value">{move || selected_leaf.get().map(|leaf| format_path(&leaf.path)).unwrap_or_else(|| "Not selected".to_string())}</p>
                        </div>
                        <div>
                            <p class="label">Loaded compounds</p>
                            <p class="value">{move || compounds.get().as_ref().map(|items| items.len().to_string()).unwrap_or_else(|| active_dataset.get().len().to_string())}</p>
                        </div>
                        <div>
                            <p class="label">Options per quiz</p>
                            <p class="value">{DEMO_OPTION_COUNT.to_string()}</p>
                        </div>
                    </div>
                    <div class="menu-actions">
                        <button class="primary" on:click=start_game>
                            "Start quiz"
                        </button>
                    </div>
                </section>

                <section class="dataset-panel">
                    <div class="panel-header">
                        <p class="eyebrow">Catalog</p>
                        <h2 class="headline">Browse compound folders</h2>
                        <p class="lede">Tap or click through the tree to load a JSON dataset before generating a quiz.</p>
                    </div>
                    {move || match manifest.get() {
                        Some(Ok(listing)) => view! { <CatalogTree manifest=listing.clone() on_select=handle_selection.clone() /> }
                            .into_view(),
                        Some(Err(message)) => view! { <p class="error-body">{message}</p> }.into_view(),
                        None => view! { <p class="lede">Loading catalog index...</p> }.into_view(),
                    }}
                </section>
            </Show>

            <Show when=move || scene.get() == Scene::Game>
                <section class="quiz-shell">
                    {move || {
                        if let Some(item) = quiz.get() {
                            view! {
                                <QuizCard
                                    quiz=item
                                    dataset=active_dataset.get()
                                    theme=theme
                                    selected=selected_option.get()
                                    reveal=false
                                    on_select=Some(choose_option.clone())
                                />
                            }
                            .into_view()
                        } else if let Some(message) = error.get() {
                            view! {
                                <section class="error-card card-surface">
                                    <p class="eyebrow">Generator error</p>
                                    <p class="error-body">{message}</p>
                                </section>
                            }
                            .into_view()
                        } else {
                            view! {
                                <section class="placeholder-card card-surface">
                                    <p class="eyebrow">Awaiting prompt</p>
                                    <p class="lede">Load a catalog entry and start a quiz from the menu.</p>
                                </section>
                            }
                            .into_view()
                        }
                    }}
                </section>
                <div class="action-row">
                    <button class="pill" on:click=return_to_menu>"Back to menu"</button>
                </div>
            </Show>

            <Show when=move || scene.get() == Scene::Result>
                {move || {
                    result_view
                        .get()
                        .map(|result| {
                            let is_correct = result.selected == result.quiz.correct_index;

                            view! {
                                <section class="result-card card-surface">
                                    <p class="eyebrow">Result</p>
                                    <p class="lede">{if is_correct { "Correct answer" } else { "Not quite" }}</p>
                                    <p class="score-line">{move || format!("Score {} / {}", score.get().correct, score.get().total)}</p>
                                    <div class="action-row">
                                        <button class="primary" on:click=next_question>"Next question"</button>
                                        <button class="pill" on:click=return_to_menu>"Back to menu"</button>
                                    </div>
                                </section>
                                <section class="quiz-shell">
                                    <QuizCard
                                        quiz=result.quiz.clone()
                                        dataset=result.dataset.clone()
                                        theme=theme
                                        selected=Some(result.selected)
                                        reveal=true
                                        on_select=None
                                    />
                                </section>
                            }
                            .into_view()
                        })
                        .unwrap_or_else(|| {
                            view! {
                                <section class="placeholder-card card-surface">
                                    <p class="eyebrow">Awaiting result</p>
                                    <p class="lede">Answer a quiz to see the outcome.</p>
                                </section>
                            }
                            .into_view()
                        })
                }}
            </Show>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
