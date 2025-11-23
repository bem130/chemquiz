#![cfg(target_arch = "wasm32")]

use crate::{
    CatalogLeaf, CatalogManifest, CatalogNode, Compound, DEMO_OPTION_COUNT, QuizItem, QuizMode,
    demo_compounds, generate_quiz,
};
use gloo_net::http::Request;
use js_sys::Reflect;
use leptos::{html, *};
use rand::SeedableRng;
use std::rc::Rc;
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
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Skeletal,
    Full,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PromptContent {
    Name,
    Structure,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OptionContent {
    Name,
    Structure,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutKind {
    StandardVertical,
    WideHorizontal,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct QuestionLayout {
    prompt: PromptContent,
    options: [OptionContent; 4],
    layout_kind: LayoutKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FeedbackKind {
    Neutral,
    Correct,
    Wrong,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StructureViewSize {
    Prompt,
    Option,
}

#[derive(Clone, PartialEq, Eq)]
struct FeedbackState {
    message: String,
    kind: FeedbackKind,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct SessionScore {
    total: usize,
    correct: usize,
}

#[derive(Clone)]
struct AnswerOverlay {
    quiz: QuizItem,
    dataset: Vec<Compound>,
    selected: usize,
    is_correct: bool,
}

impl FeedbackState {
    fn neutral(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FeedbackKind::Neutral,
        }
    }

    fn correct(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FeedbackKind::Correct,
        }
    }

    fn wrong(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FeedbackKind::Wrong,
        }
    }
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
        .find(|compound| compound.english_label() == label)
        .cloned()
}

fn find_by_structure(dataset: &[Compound], label: &str) -> Option<Compound> {
    dataset
        .iter()
        .find(|compound| compound.display_structure() == label)
        .cloned()
}

fn english_label(compound: &Compound) -> String {
    compound.english_label()
}

fn japanese_label(compound: &Compound) -> Option<String> {
    compound.local_name.clone()
}

fn hint_from_compound(compound: &Compound) -> Option<String> {
    if let Some(series) = &compound.series_general_formula {
        return Some(format!("Series formula: {}", series));
    }

    if !compound.functional_groups.is_empty() {
        let groups: Vec<String> = compound
            .functional_groups
            .iter()
            .map(|group| format!("{} ({})", group.name_en, group.pattern))
            .collect();
        return Some(format!("Functional groups: {}", groups.join(", ")));
    }

    if let Some(notes) = &compound.notes {
        return Some(notes.clone());
    }

    if !compound.molecular_formula.is_empty() {
        return Some(format!("Molecular formula: {}", compound.molecular_formula));
    }

    None
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

fn layout_for_quiz(quiz: &QuizItem) -> QuestionLayout {
    match quiz.mode {
        QuizMode::NameToStructure => QuestionLayout {
            prompt: PromptContent::Name,
            options: [OptionContent::Structure; 4],
            layout_kind: LayoutKind::StandardVertical,
        },
        QuizMode::StructureToName => QuestionLayout {
            prompt: PromptContent::Structure,
            options: [OptionContent::Name; 4],
            layout_kind: LayoutKind::StandardVertical,
        },
    }
}

fn compound_for_prompt(dataset: &[Compound], quiz: &QuizItem) -> Option<Compound> {
    match quiz.mode {
        QuizMode::NameToStructure => find_by_name(dataset, &quiz.prompt),
        QuizMode::StructureToName => find_by_structure(dataset, &quiz.prompt),
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

    view! { <div class="molformula-badge" node_ref=node_ref></div> }
}

#[component]
fn StructureTile(
    compound: Compound,
    theme: ReadSignal<String>,
    view_mode: ReadSignal<ViewMode>,
    size: StructureViewSize,
) -> impl IntoView {
    let skeletal_ref = create_node_ref::<html::Canvas>();
    let full_ref = create_node_ref::<html::Div>();
    let (render_message, set_render_message) = create_signal::<Option<String>>(None);

    let smiles = compound.smiles.clone();
    let effect_smiles = smiles.clone();
    let iupac_name = compound.iupac_name.clone();
    let skeletal_formula = compound.skeletal_formula.clone();
    let molecular_formula = compound.molecular_formula.clone();
    let formula_badge = (!molecular_formula.is_empty())
        .then(|| view! { <FormulaBadge formula=molecular_formula.clone() /> });

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

    let container_class = match size {
        StructureViewSize::Prompt => "viewer-inner prompt-large",
        StructureViewSize::Option => "option-structure-inner",
    };

    let visuals = smiles.map(|_| {
        view! {
            <div class=container_class>
                <canvas
                    node_ref=skeletal_ref
                    style=move || {
                        if view_mode.get() == ViewMode::Skeletal {
                            "display:block".to_string()
                        } else {
                            "display:none".to_string()
                        }
                    }
                    role="img"
                    aria-label=format!("Skeletal depiction for {}", iupac_name.clone())
                ></canvas>
                <div
                    node_ref=full_ref
                    class="kekule-container"
                    style=move || {
                        if view_mode.get() == ViewMode::Full {
                            "display:block".to_string()
                        } else {
                            "display:none".to_string()
                        }
                    }
                    role="img"
                    aria-label=format!("Full structural formula for {}", iupac_name.clone())
                ></div>
                {formula_badge.clone()}
            </div>
        }
        .into_view()
    });

    view! {
        {visuals.unwrap_or_else(|| {
            view! {
                <div class=container_class>
                    <p class="prompt-formula-text">{format!("{}", skeletal_formula.clone())}</p>
                    {formula_badge.clone()}
                </div>
            }
            .into_view()
        })}
        {move || {
            render_message
                .get()
                .map(|message| view! { <p class="prompt-formula-text">{message}</p> })
        }}
    }
}

#[component]
fn CatalogTreeNode(
    node: CatalogNode,
    prefix: Vec<String>,
    selected: ReadSignal<Vec<CatalogLeaf>>,
    on_select: Callback<CatalogLeaf>,
) -> impl IntoView {
    // Sort children for stable order
    let mut children = node.children.clone();
    children.sort_by(|left, right| left.label.cmp(&right.label));

    // Build full path like ["Inorganic compounds", "Metals", "Alkali metals"]
    let mut path = prefix.clone();
    path.push(node.label.clone());

    // If this node has a file, it can be selected
    let leaf_for_node = node.file.as_ref().map(|file| CatalogLeaf {
        path: path.clone(),
        file: file.clone(),
    });

    // Button view (None → no button, Some → Select button)
    let button_view = leaf_for_node.clone().map(|leaf| {
        let on_select = on_select.clone();
        let class_leaf = leaf.clone();
        let click_leaf = leaf.clone();

        view! {
            <button
                type="button"
                class=move || {
                    let list = selected.get();
                    if list.iter().any(|current| current == &class_leaf) {
                        "mode-btn active".to_string()
                    } else {
                        "mode-btn".to_string()
                    }
                }
                on:click=move |_| {
                    on_select.call(click_leaf.clone());
                }
            >
                "Select"
            </button>
        }
    });

    // Children list (for tree structure)
    let children_view = if children.is_empty() {
        None
    } else {
        let path_for_children = path.clone();

        Some(view! {
            <ul class="catalog-children">
                {children
                    .into_iter()
                    .map(move |child| {
                        let prefix_for_child = path_for_children.clone();
                        view! {
                            <CatalogTreeNode
                                node=child
                                prefix=prefix_for_child
                                selected=selected
                                on_select=on_select.clone()
                            />
                        }
                    })
                    .collect_view()}
            </ul>
        })
    };

    view! {
        <li class="catalog-node">
            <div class="catalog-row">
                <div class="catalog-label">{node.label.clone()}</div>
                <div class="catalog-actions">
                    {button_view}
                </div>
            </div>
            {children_view}
        </li>
    }
}

#[component]
fn CatalogTree(
    manifest: CatalogManifest,
    selected: ReadSignal<Vec<CatalogLeaf>>,
    on_select: Callback<CatalogLeaf>,
) -> impl IntoView {
    let mut roots = manifest.roots.clone();
    roots.sort_by(|left, right| left.label.cmp(&right.label));

    view! {
        <ul class="catalog-tree">
            {roots
                .into_iter()
                .map(|root| {
                    view! {
                        <CatalogTreeNode
                            node=root
                            prefix=Vec::new()
                            selected=selected
                            on_select=on_select.clone()
                        />
                    }
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
    view_mode: ReadSignal<ViewMode>,
    selected: Option<usize>,
    feedback: FeedbackState,
    hint: Option<String>,
    hint_visible: bool,
    reveal: bool,
    layout: QuestionLayout,
    on_select: Option<Callback<usize>>,
    on_toggle_hint: Callback<()>,
) -> impl IntoView {
    let heading_label = match layout
        .options
        .first()
        .copied()
        .unwrap_or(OptionContent::Name)
    {
        OptionContent::Structure => "Choose the correct structure",
        OptionContent::Name => "Choose the correct name",
    };

    let prompt_compound = compound_for_prompt(&dataset, &quiz);

    let feedback_class = match feedback.kind {
        FeedbackKind::Neutral => "feedback-text feedback-neutral",
        FeedbackKind::Correct => "feedback-text feedback-correct",
        FeedbackKind::Wrong => "feedback-text feedback-wrong",
    };

    let grid_class = match layout.layout_kind {
        LayoutKind::StandardVertical => "grid-main".to_string(),
        LayoutKind::WideHorizontal => "grid-main grid-horizontal".to_string(),
    };

    view! {
        <div class=grid_class>
            <div class="prompt-card">
                <div class="prompt-card-header">
                    <div class="prompt-heading">Prompt</div>
                </div>
                <div class="prompt-body">
                    {prompt_compound
                        .map(|compound| match layout.prompt {
                            PromptContent::Name => {
                                let english = english_label(&compound);
                                let japanese = japanese_label(&compound);
                                let molecular = (!compound.molecular_formula.is_empty())
                                    .then(|| compound.molecular_formula.clone());

                                view! {
                                    <div>
                                        <div class="prompt-name-main">{english}</div>
                                        {japanese
                                            .map(|name| view! { <div class="prompt-name-ja">{name}</div> })}
                                        {molecular
                                            .map(|formula| {
                                                view! { <div class="prompt-formula-text">{formula}</div> }
                                            })}
                                    </div>
                                }
                                .into_view()
                            }
                            PromptContent::Structure => {
                                view! {
                                    <div class="structure-container">
                                        <div class="viewer-card">
                                            <div class="viewer-title-row">
                                                <div class="viewer-label">Skeletal / full structure</div>
                                                <div class="viewer-badge">Prompt</div>
                                            </div>
                                            <StructureTile
                                                compound=compound
                                                theme=theme
                                                view_mode=view_mode
                                                size=StructureViewSize::Prompt
                                            />
                                        </div>
                                    </div>
                                }
                                .into_view()
                            }
                        })
                        .unwrap_or_else(|| view! { <p class="prompt-formula-text">{quiz.prompt.clone()}</p> }.into_view())}
                </div>
            </div>

            <div class="options-card">
                <div class="prompt-card-header" style="margin-bottom:6px;">
                    <div class="prompt-heading">{heading_label}</div>
                </div>
                <div class="options-grid">
                    {quiz
                        .options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| {
                            let is_selected = selected == Some(index);
                            let is_correct = index == quiz.correct_index;

                            let mut classes = vec!["option-btn".to_string()];
                            if reveal {
                                classes.push("option-disabled".to_string());

                                if is_correct {
                                    classes.push("option-correct".to_string());
                                } else if is_selected {
                                    classes.push("option-wrong".to_string());
                                }
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
                                    <div class="option-row-top">
                                        <span class="option-tag">{format!("Option {}", index + 1)}</span>
                                        {is_correct
                                            .then(|| view! { <span class="option-tag">Correct</span> })}
                                    </div>
                                    {compound
                                        .map(|compound| match layout.options[index] {
                                            OptionContent::Structure => {
                                                view! {
                                                    <div class="option-structure-box">
                                                        <StructureTile
                                                            compound=compound
                                                            theme=theme
                                                            view_mode=view_mode
                                                            size=StructureViewSize::Option
                                                        />
                                                    </div>
                                                }
                                                .into_view()
                                            }
                                            OptionContent::Name => {
                                                let english = english_label(&compound);
                                                let japanese = japanese_label(&compound);

                                                view! {
                                                    <div class="option-name-inner">
                                                        <p class="option-name-main">{english}</p>
                                                        {japanese
                                                            .map(|name| view! { <p class="option-name-ja">{name}</p> })}
                                                    </div>
                                                }
                                                .into_view()
                                            }
                                        })
                                        .unwrap_or_else(|| view! { <p class="prompt-formula-text">{option.clone()}</p> }
                                            .into_view())}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>

                <div class="controls-row">
                    <div class=feedback_class>{feedback.message}</div>
                    <div class="controls-buttons">
                        <button
                            class="btn"
                            type="button"
                            on:click=move |_| on_toggle_hint.call(())
                            disabled=hint.is_none()
                        >
                            {if hint_visible { "Hide hint" } else { "Show hint" }}
                        </button>
                    </div>
                </div>

                <Show when=move || hint_visible>
                    <div class="hint-box">
                        "Hint: "
                        {hint.clone().unwrap_or_else(|| "–".to_string())}
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (theme, _set_theme) = create_signal(String::from("dark"));
    let (mode, set_mode) = create_signal(QuizMode::NameToStructure);
    let (active_mode, set_active_mode) = create_signal(QuizMode::NameToStructure);
    let (view_mode, set_view_mode) = create_signal(ViewMode::Skeletal);
    let (quiz, set_quiz) = create_signal::<Option<QuizItem>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (active_dataset, set_active_dataset) = create_signal::<Vec<Compound>>(demo_compounds());
    let (scene, set_scene) = create_signal(Scene::Menu);
    let (selected_option, set_selected_option) = create_signal::<Option<usize>>(None);
    let (score, set_score) = create_signal(SessionScore::default());
    let (answer_overlay, set_answer_overlay) = create_signal::<Option<AnswerOverlay>>(None);
    let (feedback, set_feedback) = create_signal(FeedbackState::neutral(
        "Load a catalog entry and start a quiz.",
    ));
    let (hint, set_hint) = create_signal::<Option<String>>(None);
    let (hint_visible, set_hint_visible) = create_signal(false);

    // Quiz uses “last clicked leaf” only
    let (selected_leaf, set_selected_leaf) = create_signal::<Option<CatalogLeaf>>(None);

    // UI can highlight multiple selected nodes
    let (selected_nodes, set_selected_nodes) = create_signal::<Vec<CatalogLeaf>>(Vec::new());

    let (compounds, set_compounds) = create_signal::<Option<Vec<Compound>>>(None);

    let manifest = create_resource(|| (), |_| async { fetch_manifest().await });

    create_effect(move |_| set_body_theme(&theme.get()));

    let toggle_hint = {
        let hint = hint.clone();
        let set_hint_visible = set_hint_visible.clone();

        Callback::new(move |_| {
            if hint.get().is_some() {
                set_hint_visible.update(|flag| *flag = !*flag);
            }
        })
    };

    let regenerate = {
        let active_mode = active_mode.clone();
        let set_error = set_error.clone();
        let set_hint = set_hint.clone();
        let set_feedback = set_feedback.clone();
        let set_quiz = set_quiz.clone();
        let set_active_dataset = set_active_dataset.clone();
        let set_selected_option = set_selected_option.clone();
        let set_hint_visible = set_hint_visible.clone();
        let set_answer_overlay = set_answer_overlay.clone();
        let compounds = compounds.clone();
        let active_dataset = active_dataset.clone();

        Rc::new(move || -> bool {
            set_selected_option.set(None);
            set_answer_overlay.set(None);
            set_hint_visible.set(false);

            let dataset = compounds.get().unwrap_or_else(|| active_dataset.get());

            match generate_from_dataset(&dataset, active_mode.get()) {
                Ok(item) => {
                    set_error.set(None);
                    set_active_dataset.set(dataset.clone());

                    let prompt_compound = compound_for_prompt(&dataset, &item);

                    set_hint
                        .set(prompt_compound.and_then(|compound| hint_from_compound(&compound)));
                    set_feedback.set(FeedbackState::neutral(
                        "Select an option to submit your answer.",
                    ));
                    set_quiz.set(Some(item));
                    true
                }
                Err(message) => {
                    set_quiz.set(None);
                    set_hint.set(None);
                    set_feedback.set(FeedbackState::wrong(message.clone()));
                    set_error.set(Some(message));
                    false
                }
            }
        })
    };

    let start_game = {
        let regenerate = regenerate.clone();
        let set_scene = set_scene.clone();
        let set_score = set_score.clone();
        let set_active_mode = set_active_mode.clone();
        let set_answer_overlay = set_answer_overlay.clone();

        Callback::new(move |_| {
            set_score.set(SessionScore::default());
            set_active_mode.set(mode.get());
            set_answer_overlay.set(None);
            if regenerate() {
                set_scene.set(Scene::Game);
            }
        })
    };

    let return_to_menu = {
        let set_scene = set_scene.clone();
        let set_quiz = set_quiz.clone();
        let set_selected_option = set_selected_option.clone();
        let set_feedback = set_feedback.clone();
        let set_hint = set_hint.clone();
        let set_hint_visible = set_hint_visible.clone();
        let set_answer_overlay = set_answer_overlay.clone();

        Callback::new(move |_| {
            set_scene.set(Scene::Menu);
            set_quiz.set(None);
            set_selected_option.set(None);
            set_answer_overlay.set(None);
            set_feedback.set(FeedbackState::neutral(
                "Load a catalog entry and start a quiz.",
            ));
            set_hint.set(None);
            set_hint_visible.set(false);
        })
    };

    let choose_option = {
        let scene = scene.clone();
        let selected_option = selected_option.clone();
        let quiz = quiz.clone();
        let set_selected_option = set_selected_option.clone();
        let set_score = set_score.clone();
        let active_dataset = active_dataset.clone();
        let set_feedback = set_feedback.clone();
        let set_answer_overlay = set_answer_overlay.clone();

        Callback::new(move |index: usize| {
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

                let is_correct = index == item.correct_index;
                let feedback_message = if is_correct {
                    FeedbackState::correct("Correct! View the overlay for details.")
                } else {
                    FeedbackState::wrong(
                        "Not quite. Review the highlighted answer and open the overlay.",
                    )
                };
                set_feedback.set(feedback_message);

                set_answer_overlay.set(Some(AnswerOverlay {
                    quiz: item.clone(),
                    dataset: active_dataset.get(),
                    selected: index,
                    is_correct,
                }));
            }
        })
    };

    let next_question = {
        let regenerate = regenerate.clone();
        let set_scene = set_scene.clone();
        let set_answer_overlay = set_answer_overlay.clone();
        let set_selected_option = set_selected_option.clone();

        Callback::new(move |_| {
            set_selected_option.set(None);
            set_answer_overlay.set(None);
            if regenerate() {
                set_scene.set(Scene::Game);
            }
        })
    };

    let handle_selection = {
        let selected_nodes = selected_nodes.clone();
        let set_selected_nodes = set_selected_nodes.clone();
        let set_selected_leaf = set_selected_leaf.clone();
        let set_error = set_error.clone();
        let set_quiz = set_quiz.clone();
        let set_scene = set_scene.clone();
        let set_compounds = set_compounds.clone();
        let set_feedback = set_feedback.clone();
        let set_hint = set_hint.clone();
        let set_hint_visible = set_hint_visible.clone();

        Callback::new(move |leaf: CatalogLeaf| {
            // --- UI: トグル ---
            let mut list = selected_nodes.get();
            if let Some(idx) = list.iter().position(|item| item == &leaf) {
                // すでに選ばれているなら解除
                list.remove(idx);
            } else {
                // 新規に選択
                list.push(leaf.clone());
            }
            set_selected_nodes.set(list);

            // --- クイズ用: 「最後に押したもの」を保存 ---
            // ここでは仕様をシンプルに保つため、
            // 実際に問題を出すのは「最後に押した葉ノード 1つだけ」のままにしています。
            set_selected_leaf.set(Some(leaf.clone()));

            // 以降は今まで通り、leaf.file から 1つの compounds.json を読み込む
            set_error.set(None);
            set_quiz.set(None);
            set_scene.set(Scene::Menu);
            set_compounds.set(None);
            set_hint.set(None);
            set_hint_visible.set(false);
            set_feedback.set(FeedbackState::neutral("Loading selected catalog entry..."));

            let setter = set_compounds.clone();
            let error_setter = set_error.clone();
            let feedback_setter = set_feedback.clone();

            spawn_local(async move {
                match fetch_compound_file(&leaf.file).await {
                    Ok(list) => {
                        feedback_setter.set(FeedbackState::neutral(format!(
                            "Loaded {} compounds.",
                            list.len()
                        )));
                        setter.set(Some(list));
                    }
                    Err(message) => {
                        feedback_setter.set(FeedbackState::wrong(message.clone()));
                        error_setter.set(Some(message));
                    }
                }
            });
        })
    };

    let question_total = move || active_dataset.get().len();
    let question_position = move || match scene.get() {
        Scene::Menu => 0,
        Scene::Game => {
            let answered = score.get().total;
            if answer_overlay.get().is_some() {
                answered
            } else {
                answered + 1
            }
        }
    };
    let progress = move || {
        let total = question_total();
        if total == 0 {
            0.0
        } else {
            let position = question_position().min(total);
            (position as f64 / total as f64) * 100.0
        }
    };
    let incorrect_count = move || score.get().total.saturating_sub(score.get().correct);

    view! {
        <div class="app">
            <Show when=move || scene.get() == Scene::Menu>
                <div>
                    <header class="app-header">
                        <div class="app-title">"Molecular Structure Quiz (compounds.json)"</div>
                        <div class="app-subtitle">
                            "Structure → Name / Name → Structure, using Kekule.js where SMILES are available."
                        </div>
                    </header>

                    <section class="panel">
                        <div class="panel-title">"Quiz settings"</div>
                        <div class="top-row">
                            <div>
                                <div style="font-size:0.72rem;color:var(--text-muted);margin-bottom:3px;">
                                    "Quiz mode"
                                </div>
                                <div class="mode-switch">
                                    <button
                                        class=move || {
                                            if mode.get() == QuizMode::StructureToName {
                                                "mode-btn active".to_string()
                                            } else {
                                                "mode-btn".to_string()
                                            }
                                        }
                                        type="button"
                                        on:click=move |_| set_mode.set(QuizMode::StructureToName)
                                    >
                                        "Structure → Name"
                                    </button>
                                    <button
                                        class=move || {
                                            if mode.get() == QuizMode::NameToStructure {
                                                "mode-btn active".to_string()
                                            } else {
                                                "mode-btn".to_string()
                                            }
                                        }
                                        type="button"
                                        on:click=move |_| set_mode.set(QuizMode::NameToStructure)
                                    >
                                        "Name → Structure"
                                    </button>
                                </div>
                                <div style="display:inline-block;margin-left:10px;vertical-align:middle;">"Structure view"</div>
                                <div class="mode-switch" id="viewModeSwitch" style="--gap:6px;">
                                    <button
                                        class=move || {
                                            if view_mode.get() == ViewMode::Skeletal {
                                                "mode-btn active".to_string()
                                            } else {
                                                "mode-btn".to_string()
                                            }
                                        }
                                        type="button"
                                        on:click=move |_| set_view_mode.set(ViewMode::Skeletal)
                                    >
                                        "Skeletal"
                                    </button>
                                    <button
                                        class=move || {
                                            if view_mode.get() == ViewMode::Full {
                                                "mode-btn active".to_string()
                                            } else {
                                                "mode-btn".to_string()
                                            }
                                        }
                                        type="button"
                                        on:click=move |_| set_view_mode.set(ViewMode::Full)
                                    >
                                        "Full"
                                    </button>
                                </div>
                            </div>
                            <div class="score-badge">
                                <span>"Score:"</span>
                                <strong><span>{move || score.get().correct}</span></strong>
                                <span>"/ "<span>{move || score.get().total}</span></span>
                            </div>
                        </div>
                    </section>

                    <section class="panel">
                        <div class="panel-title">"Catalog & quiz setup"</div>
                        <div class="catalog-grid">
                            <div class="catalog-card">
                                <div class="prompt-heading">"Dataset status"</div>
                                <div class="menu-status">
                                    <div class="menu-chip">
                                        <div class="prompt-heading">"Selected path"</div>
                                        <div>{move || selected_leaf
                                            .get()
                                            .map(|leaf| format_path(&leaf.path))
                                            .unwrap_or_else(|| "Not selected".to_string())}</div>
                                    </div>
                                    <div class="menu-chip">
                                        <div class="prompt-heading">"Loaded compounds"</div>
                                        <div>{move || compounds
                                            .get()
                                            .as_ref()
                                            .map(|items| items.len().to_string())
                                            .unwrap_or_else(|| active_dataset.get().len().to_string())}</div>
                                    </div>
                                    <div class="menu-chip">
                                        <div class="prompt-heading">"Options per quiz"</div>
                                        <div>{DEMO_OPTION_COUNT.to_string()}</div>
                                    </div>
                                </div>
                                <div class="menu-actions">
                                    <button
                                        class="btn btn-primary"
                                        type="button"
                                        on:click=move |_| start_game.call(())
                                    >
                                        "Start quiz"
                                    </button>
                                </div>
                            </div>
                            <div class="catalog-card">
                                <div class="prompt-heading">"Catalog"</div>
                                {move || match manifest.get() {
                                    Some(Ok(listing)) => {
                                        view! {
                                            <CatalogTree
                                                manifest=listing.clone()
                                                selected=selected_nodes
                                                on_select=handle_selection.clone()
                                            />
                                        }.into_view()
                                    }
                                    Some(Err(message)) => view! { <p class="error-body">{message}</p> }.into_view(),
                                    None => view! { <p class="prompt-formula-text">"Loading catalog index..."</p> }.into_view(),
                                }}
                            </div>
                        </div>
                    </section>
                </div>
            </Show>

            <Show when=move || scene.get() == Scene::Game>
                <div class="play-page">
                    <div class="play-main">
                        {move || {
                            if let Some(item) = quiz.get() {
                                let layout = layout_for_quiz(&item);
                                let reveal = selected_option.get().is_some();
                                let on_select = if answer_overlay.get().is_some() {
                                    None
                                } else {
                                    Some(choose_option.clone())
                                };

                                view! {
                                    <section class="panel play-panel">
                                        <QuizCard
                                            quiz=item
                                            dataset=active_dataset.get()
                                            theme=theme
                                            view_mode=view_mode
                                            selected=selected_option.get()
                                            feedback=feedback.get()
                                            hint=hint.get()
                                            hint_visible=hint_visible.get()
                                            reveal=reveal
                                            layout=layout
                                            on_select=on_select
                                            on_toggle_hint=toggle_hint.clone()
                                        />
                                    </section>
                                }
                                .into_view()
                            } else if let Some(message) = error.get() {
                                view! {
                                    <section class="panel play-panel">
                                        <div class="prompt-heading">"Generator error"</div>
                                        <p class="error-body">{message}</p>
                                    </section>
                                }
                                .into_view()
                            } else {
                                view! {
                                    <section class="panel play-panel">
                                        <div class="prompt-heading">"Awaiting prompt"</div>
                                        <p class="prompt-formula-text">"Load a catalog entry and start a quiz from the menu."</p>
                                    </section>
                                }
                                .into_view()
                            }
                        }}
                    </div>

                    <div class="play-progress-overlay">
                        <div class="progress-bar">
                            <div
                                class="progress-bar-inner"
                                style=move || format!("width: {:.2}%;", progress())
                            ></div>
                        </div>
                        <div class="progress-meta">
                            <span>
                                "Question "
                                <span>{question_position}</span>
                                "/"
                                <span>{question_total}</span>
                            </span>
                            <span style="font-size:0.72rem;color:var(--text-muted);">
                                {move || format!(
                                    "Correct: {} | Incorrect: {}",
                                    score.get().correct,
                                    incorrect_count()
                                )}
                            </span>
                        </div>
                    </div>

                    <Show when=move || answer_overlay.get().is_some()>
                        {move || {
                            answer_overlay.get().map(|overlay| {
                                let prompt_compound = compound_for_prompt(&overlay.dataset, &overlay.quiz);
                                let subtitle = prompt_compound
                                    .as_ref()
                                    .map(|compound| english_label(compound))
                                    .unwrap_or_else(|| "Summary".to_string());
                                let message = if overlay.is_correct {
                                    "Correct"
                                } else {
                                    "Incorrect"
                                };

                                view! {
                                    <div class="result-overlay" role="dialog" aria-modal="true">
                                        <div class="result-popup">
                                            <h2 class=if overlay.is_correct { "result-title correct" } else { "result-title incorrect" }>
                                                {message}
                                            </h2>
                                            <p class="result-subtitle">{subtitle}</p>
                                            <div class="result-body">
                                                <p class="prompt-formula-text">{"Review the highlighted options. The question layout stays fixed while this overlay is visible."}</p>
                                            </div>
                                            <div class="overlay-actions">
                                                <button class="btn btn-primary" type="button" on:click=move |_| next_question.call(())>
                                                    "Next question"
                                                </button>
                                                <button class="btn" type="button" on:click=move |_| return_to_menu.call(())>
                                                    "Back to menu"
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                }
                                .into_view()
                            })
                        }}
                    </Show>
                </div>
            </Show>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
