#![recursion_limit = "512"]

// lib.rs
use wasm_bindgen::prelude::*;
use web_sys::{console, Node};
use yew::virtual_dom::VNode;
use yew::{Component, ComponentLink, Html, InputData, ShouldRender};
use yew::prelude::*;
mod markdown;

struct Model {
    link: ComponentLink<Self>,
    markdown: String,
    value: String,
    show: String,
}

enum Msg {
    GotInput(String),
    Reset,
    ToggleShow,
    Clear,
}

const DEFAULT: &str = "### 🎭 **Prose**
###### **Turning your markdown into lovely HTML!**
Prose is here to let you draft up a document and watch it render in real time.
When it is time to save your work, Prose gives you the ability download your document as an `.md` file.
Prose is here to let you draft up a document and watch it render in real time.
If you want to use this HTML elsewhere, just click the button above to switch the view to raw, unrendered HTML. This way you can copy&paste is anywhere you'd like.
When it is time to save your work, Prose gives you the ability download your document as an `.md` file.

##### Built on the following tech:
- 🦀[Rust](https://www.rust-lang.org/) as your typical programming language
- 🕸[WASM](https://webassembly.org/) to run compiled Rust code in the browser
- 🍟[Nom](https://github.com/Geal/nom) to parse the markdown into a Syntax Tree
- 🌳[Yew](https://yew.rs/docs/) as the web framework

#### Support
###### Prose supports the following markdown structures:
1. Headers 1-6
1. Ordered Lists
1. Unordered Lists
1. Codeblocks (no specified language support)
1. **boldtext**
1. *italic text*
1. `inline_code`
1. Links
1. Images


You may be asking: *What makes this better than any other markdown parser?*
Well, this is implemented in a very performant systems programming language and is much much faster.
Using WebAssembly, we have been able to compile this code into a format that runs super fast in the browser's JavaScript engine.
You probably will see above a benchmark on how long it took to parse this. I have done my own measurements and found most tools are 30-50 times slower. Not a good look :(


#### Coming Soon!
- Bugfixes
- Download Feature
- New Markdown Flavors
- In-Page Timer
";

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            show: String::from("MARKDOWN"),
            value: String::from(DEFAULT),
            markdown: markdown::markdown::markdown(DEFAULT),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotInput(new_value) => {
                let _timer = Timer::new("Markdown::render");
                self.markdown = markdown::markdown::markdown(&new_value);
                self.value = new_value;
            }
            Msg::Reset => {
                self.markdown = markdown::markdown::markdown(DEFAULT);
                self.value = DEFAULT.to_string();
            }
            Msg::Clear => {
                self.markdown = "".to_string();
                self.value = "".to_string();
            }
            Msg::ToggleShow => {
                if self.show == "HTML" {
                    self.show = "MARKDOWN".to_string();
                } else {
                    self.show = "HTML".to_string();
                }
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let rendered_markdown = {
            let div = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("div")
                .unwrap();
            div.set_inner_html(&self.markdown);
            div
        };

        eprintln!("rendered_markdown: {:?}", rendered_markdown);
        let node = Node::from(rendered_markdown);
        let vnode = VNode::VRef(node);
        eprintln!("rendered_markdown: {:?}", vnode);

        html! {
            <div class={"main"}>
                <div class={"header"}>
                    <h1>{"🎭 Prose"}</h1>
                    <h6>{"Created by "}<a href={"https://github.com/HGHimself"}>{"HG King"}</a></h6>
                </div>
                <div class={"container"}>
                    <div class={"left"}>
                        <div class={"input"}>
                            <textarea rows=100
                                value=&self.value
                                wrap={"off"}
                                oninput=self.link.callback(|e: InputData| Msg::GotInput(e.value))
                                placeholder="placeholder">
                            </textarea>
                        </div>
                    </div>
                    <div class={"right"}>
                        <div class={"container"}>
                            <button
                                class={"success-button"}
                                onclick=self.link.callback(|_| Msg::Reset)>{ "RESET" }</button>
                            <button
                                class={"info-button"}
                                onclick=self.link.callback(|_| Msg::ToggleShow)>{ &self.show }</button>
                            <button
                                class={"error-button"}
                                onclick=self.link.callback(|_| Msg::Clear)>{ "CLEAR" }</button>
                        </div>
                        <div class={"output"}>
                            <div class={if &self.show == "MARKDOWN" { "x" } else { "hidden" }}>{vnode}</div>
                            <div class={if &self.show == "HTML" { "x" } else { "hidden" }}>{&self.markdown}</div>
                        </div>
                    </div>
                </div>
            </div>
        }

    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
