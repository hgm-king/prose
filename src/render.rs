mod markdown;
use web_sys::{console, Node};
use yew::virtual_dom::VNode;
use yew::{Component, ComponentLink, Html, ShouldRender};
use yew::prelude::*;

struct Model {
    link: ComponentLink<Self>,
    markdown: String,
}

enum Msg {
    Render,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            markdown: String::from("<h1>Input your markdown here!</h1>"),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => self.value = markdown::markdown::markdown("# hey there\n")
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
            div.set_inner_html(&self.value);
            console::log_1(&div);
            div
        };
        eprintln!("js_svg: {:?}", rendered_markdown);
        let node = Node::from(rendered_markdown);
        let vnode = VNode::VRef(node);
        
        eprintln!("svg: {:?}", vnode);
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                { vnode }
            </div>
        }

    }
}
