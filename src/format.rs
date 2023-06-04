use super::{entry::Entry, Result};
use chrono::prelude::*;
use colored::*;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output, RenderContext,
};
use std::collections::BTreeMap;

pub struct Format<'a> {
    renderer: Handlebars<'a>,
    data: BTreeMap<&'static str, String>,
}

impl<'a> Format<'a> {
    pub fn with_template(template: &str) -> Result<Self> {
        let mut renderer = Handlebars::new();
        renderer.set_strict_mode(true);
        renderer.register_escape_fn(|s| s.trim().to_owned());
        renderer.register_template_string("template", template)?;
        renderer.register_helper("indent", Box::new(IndentHelper {}));
        renderer.register_helper("strftime", Box::new(StrftimeHelper {}));
        renderer.register_helper("color", Box::new(ColorHelper {}));
        renderer.register_helper("markdown", Box::new(MarkdownHelper {}));

        Ok(Format {
            renderer,
            data: BTreeMap::new(),
        })
    }

    pub fn format_entry(&mut self, entry: &Entry) -> Result<String> {
        self.data.clear();

        self.data.insert("datetime", entry.datetime().to_rfc3339());
        self.data.insert("message", entry.message().to_owned());

        Ok(self.renderer.render("template", &self.data)?)
    }
}

struct IndentHelper {}

impl HelperDef for IndentHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).unwrap();
        Ok(out.write(&textwrap::indent(&param.value().render(), "│ "))?)
    }
}

struct StrftimeHelper {}

impl HelperDef for StrftimeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let date_str = h.param(1).unwrap().value().render();
        let date = DateTime::parse_from_rfc3339(&date_str)
            .map_err(|_| handlebars::RenderError::new("couldn't parse date"))?;
        let local_date = date.with_timezone(&Local);

        let format_str = h.param(0).unwrap().value().render();

        Ok(out.write(&local_date.format(&format_str).to_string())?)
    }
}

struct ColorHelper {}

impl HelperDef for ColorHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let color = h.param(0).unwrap().value().render();
        let s = h.param(1).unwrap().value().render();
        Ok(out.write(&format!("{}", s.color(color)))?)
    }
}

struct MarkdownHelper {}

impl HelperDef for MarkdownHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let s = h.param(0).unwrap().value().render();
        Ok(out.write(&format!("{}", termimad::text(&s)))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("{{ message }}" => "hello world")]
    #[test_case("{{ color \"blue\" message }}" => "hello world".blue().to_string())]
    #[test_case("{{ indent message }}" => "│ hello world")]
    #[test_case("{{ strftime \"%Y-%m-%d %H:%M:%S\" datetime }}" => "2020-01-02 03:04:05")]
    fn test_format(template: &str) -> String {
        Format::with_template(template)
            .unwrap()
            .format_entry(&Entry::new(
                Utc.ymd(2020, 1, 2).and_hms(3, 4, 5).into(),
                "hello world".to_owned(),
            ))
            .unwrap()
    }
}
