# 🎭 **Prose**
###### **Turning your markdown into lovely HTML!**
Prose is here to let you draft up a document and watch it render in real time.

Prose is here to let you draft up a document and watch it render in real time.
If you want to use this HTML elsewhere, just click the button above to switch the view to raw, unrendered HTML. This way you can copy&paste is anywhere you'd like.
When it is time to save your work, Prose gives you the ability download your document as an `.md` file.

##### Built on the following tech:
- 🦀[Rust](https://www.rust-lang.org/) as your typical programming language
- 🕸[WASM](https://webassembly.org/) to run compiled Rust code in the browser
- 🍟[Nom](https://github.com/Geal/nom) to parse the markdown into a Syntax Tree
- 🌳[Yew](https://yew.rs/docs/) as the web framework

#### How Do I Run This?
Easy! You will need a few things.
1. Install `rust` from the [rust-lang](https://www.rust-lang.org/tools/install) site.
2. Install `wasm-pack` which is a crate from cargo. Just run `cargo install wasm-pack`.
3. Install `npm` from the [npm](https://www.npmjs.com/get-npm) site.
4. Run `npm run serve` in the `www` directory.
5. Navigate to `https://localhost:8080` in your browser!

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
