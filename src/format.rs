use super::{entry::Entry, Result};
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output, RenderContext,
};
use std::collections::BTreeMap;

pub struct Format<'a> {
    renderer: Handlebars<'a>,
}

impl<'a> Format<'a> {
    pub fn with_template(template: &str) -> Result<Self> {
        let mut renderer = Handlebars::new();
        renderer.set_strict_mode(true);
        renderer.register_escape_fn(|s| s.trim().to_owned());
        renderer.register_template_string("template", template)?;
        renderer.register_helper("indent", Box::new(IndentHelper::new()));

        Ok(Format { renderer })
    }

    pub fn format_entry(&self, entry: &Entry) -> Result<String> {
        let mut data = BTreeMap::new();

        data.insert("raw".to_owned(), entry.to_csv_row()?);
        data.insert("datetime".to_owned(), entry.datetime().to_rfc3339());
        data.insert("message".to_owned(), entry.message().to_owned());

        Ok(self.renderer.render("template", &data)?)
    }
}

struct IndentHelper<'a> {
    wrapper: textwrap::Wrapper<'a, textwrap::HyphenSplitter>,
}

impl<'a> IndentHelper<'a> {
    fn new() -> Self {
        let wrapper = textwrap::Wrapper::with_termwidth()
            .initial_indent("| ")
            .subsequent_indent("| ");
        IndentHelper { wrapper }
    }
}

impl<'a> HelperDef for IndentHelper<'a> {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).unwrap();
        Ok(out.write(&self.wrapper.fill(&param.value().render()))?)
    }
}
