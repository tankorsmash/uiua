use gloo::{events::EventListener, utils::document};
use uiua::{compile::Compiler, format::format, ops::Primitive, UiuaResult};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

const DEFAULT_CODE: &str = r#"⌗=¯1≡/-🗗2≤'A'∾' '."Um, I um...arrays""#;

const EXAMPLES: &[&str] = &[
    DEFAULT_CODE,
    r#"↯~𝆱/×.2_3_4"#,
    r#"◱⥀(\+⇌)10 0_1"#,
    r#"⊟⠿(≅⇌.).["uiua" "racecar" "wow" "cool!"]"#,
    r#"⌗⠿(=2/+=⌊.÷~+1𝆱.).+1𝆱60"#,
    r#"🏳️‍⚧️↯4_4[...1 .2 .3 ...4 .5 .6] # Yes, the transpose operator is the trans flag"#,
    r#"⠿(⊡~(·|:|÷|≡|⍋)⁅÷~23)^×.-~10𝆱20"#,
];

#[function_component]
fn App() -> Html {
    let (_, default_output) = run_code(DEFAULT_CODE).unwrap_throw();

    let example = use_state(|| 0);
    let code = use_state(|| DEFAULT_CODE.to_string());
    let output = use_state(move || default_output);
    let error = use_state(String::new);

    // Run the code
    let run = {
        let code = code.clone();
        let output = output.clone();
        let error = error.clone();
        move || {
            output.set(String::new());
            error.set(String::new());
            let code_string = code_element().value();
            match run_code(&code_string) {
                Ok((formatted, stack)) => {
                    code_element().set_value(&formatted);
                    code.set(formatted);
                    output.set(stack);
                }
                Err(e) => error.set(e.to_string()),
            }
        }
    };

    // Run the code when the button is clicked
    let run_click = {
        let run = run.clone();
        move |_| run()
    };

    // Go to the next example
    let next_example_click = {
        let code = code.clone();
        let run = run.clone();
        move |_| {
            let next = (*example + 1) % EXAMPLES.len();
            example.set(next);
            code.set(EXAMPLES[next].to_string());
            code_element().set_value(EXAMPLES[next]);
            run();
        }
    };

    // Run the code when Ctrl+Enter or Shift+Enter is pressed
    use_effect(move || {
        let listener = EventListener::new(&document(), "keydown", move |event| {
            let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
            if event.key() == "Enter" && (event.ctrl_key() || event.shift_key()) {
                run();
            }
        });
        || drop(listener)
    });

    // Update the code when the textarea is changed
    let oninput = {
        let code = code.clone();
        move |e: InputEvent| {
            let event: Event = e.dyn_into().unwrap_throw();
            let text_area: HtmlTextAreaElement =
                event.target().unwrap_throw().dyn_into().unwrap_throw();
            code.set(text_area.value());
        }
    };

    let mut glyph_buttons: Vec<_> = Primitive::ALL
        .iter()
        .filter_map(|p| {
            let name = p.name();
            let text = name
                .unicode
                .map(Into::into)
                .or_else(|| name.ascii.map(|s| s.to_string()))?;
            let extra = name
                .unicode
                .is_some()
                .then(|| name.ascii.map(|s| s.to_string()))
                .flatten()
                .unwrap_or_default();
            let title = format!(
                "{}{}",
                name.ident.unwrap_or_default(),
                format_args!("\n{extra}")
            );
            let onclick = {
                let code = code.clone();
                move |_| {
                    let elem = code_element();
                    if let (Ok(Some(start)), Ok(Some(end))) =
                        (elem.selection_start(), elem.selection_end())
                    {
                        let (start, end) = (start.min(end) as usize, start.max(end) as usize);
                        let text = code
                            .chars()
                            .take(start)
                            .chain(p.to_string().chars())
                            .chain(code.chars().skip(end))
                            .collect();
                        code.set(text);
                    }
                }
            };
            let class = format!(
                "glyph-button {}",
                if p.is_modifier() {
                    "modifier-button"
                } else {
                    match p.args() {
                        Some(1) => "monadic-function-button",
                        Some(2) => "dyadic-function-button",
                        Some(3) => "triadic-function-button",
                        Some(4) => "tetradic-function-button",
                        _ => "noadic-function-button",
                    }
                }
            );
            Some(html! {
                <button {class} {title} {onclick}>{ text }</button>
            })
        })
        .collect();

    for (glyph, title) in [("#", "comment"), ("=", "binding")] {
        glyph_buttons.push(html! {
            <button class="glyph-button" {title}>{glyph}</button>
        });
    }

    html! {
        <div id="top" class="centered-div">
            <h1>{ "Uiua" }</h1>
            <div id="subtitle">
                <p>{ "A stack-oriented array programming language" }</p>
                <p><a href="https://github.com/kaikalii/uiua">{"GitHub"}</a></p>
            </div>
            <div id="editor">
                <div id="glyph-buttons">{ glyph_buttons }</div>
                <textarea class="code" id="code" spellcheck="false" {oninput} value={ (*code).clone() }/>
                <div id="output" class="code">
                    <div id="code-buttons">
                        <button id="run-button" class="code-button" onclick={run_click}>{ "Run" }</button>
                        <button id="next-example"
                            class="code-button"
                            onclick={next_example_click}
                            title="Next example">{ "⏵" }
                        </button>
                    </div>
                    { (*output).clone() }
                </div>
                <p id="error" class="code">{ (*error).clone() }</p>
            </div>
            <br/>
            { main_text() }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

fn element<T: JsCast>(id: &str) -> T {
    document()
        .get_element_by_id(id)
        .unwrap_throw()
        .dyn_into::<T>()
        .unwrap_throw()
}

fn code_element() -> HtmlTextAreaElement {
    element("code")
}

fn run_code(code: &str) -> UiuaResult<(String, String)> {
    let formatted = format(code, "")?;
    let mut compiler = Compiler::new();
    compiler.load(code, "")?;
    let assembly = compiler.finish();
    let values = assembly.run()?;
    let mut s = String::new();
    for val in values.into_iter().rev() {
        s.push_str(&val.show());
    }
    Ok((formatted, s))
}

fn main_text() -> Html {
    html! {<div>
        <p>{"Uiua ("}<i>{"wee-wuh"}</i>{") is a stack-oriented array programming language with a focus on tacit code. Its semantics and primitives (and this site) are largely inspired by "}<a href="https://mlochbaum.github.io/BQN/">{"BQN"}</a>{", but it combines the array paradigm with the stack-oriented paradigm to make writing point-free code more workable."}</p>
        <hr/>
        <h3>{"How is Uiua like other array languages?"}</h3>
        <p>{"Like APL, J, BQN, and many other array languages, Uiua's primary data type is the array. Arrays are multidimensional and rank-polymorphic, meaning that many operation automatically apply to every element."}</p>
        <p>{"Uiua features:"}</p>
        <ul>
            <li>{"A rich set of primitives"}</li>
            <li>{"Arrays following the "}<a href="https://mlochbaum.github.io/BQN/doc/based.html">{"Based Array Model"}</a>{"."}</li>
            <li>{"Basic IO facilities"}</li>
            <li>{"Compile-time code evaluation"}</li>
        </ul>
        <hr/>
        <h3>{"How is Uiua different from other array languages?"}</h3>
        <p>{"Uiua does away with the infix notation of most array languages for a more general prefix notation. While functions still take arguments, they pull those arguments from the stack and push their results back on. This enables a lot of cool stuff, particularly in the realm of tacit code."}</p>
        <p>{"Uiua also features:"}</p>
        <ul>
            <li>{"Hook and fork constructs without parenthesis"}</li>
            <li>{"Selector literals, or combinators that only rearrange arguments"}</li>
            <li>{"Stack-to-array syntax for building arrays with stack operations"}</li>
            <li>{"Special syntax for constructing lists of functions"}</li>
            <li>{"Functions that work with lists of functions to form complex combinators"}</li>
        </ul>
    </div>}
}
