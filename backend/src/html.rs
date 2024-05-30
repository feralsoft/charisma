pub fn render_value(value: String) -> String {
    format!(
        "<div data-value=\"{}\" contenteditable>{}</div>",
        value, value
    )
}

pub trait Render {
    fn render_html(&self) -> String;
}

impl Render for biome_css_syntax::CssCompoundSelector {
    fn render_html(&self) -> String {
        assert!(self.sub_selectors().into_iter().len() == 1);
        let selector = self.sub_selectors().into_iter().next().unwrap();
        let (name, kind) = match selector {
            biome_css_syntax::AnyCssSubSelector::CssAttributeSelector(_) => todo!(),
            biome_css_syntax::AnyCssSubSelector::CssBogusSubSelector(_) => todo!(),
            biome_css_syntax::AnyCssSubSelector::CssClassSelector(class) => {
                (class.name().unwrap().to_string(), "class")
            }
            biome_css_syntax::AnyCssSubSelector::CssIdSelector(_) => todo!(),
            biome_css_syntax::AnyCssSubSelector::CssPseudoClassSelector(_) => todo!(),
            biome_css_syntax::AnyCssSubSelector::CssPseudoElementSelector(_) => todo!(),
        };

        format!(
            "<div data-kind=\"{}\">{}</div>",
            kind,
            render_value(name.trim().to_string())
        )
    }
}

impl Render for biome_css_syntax::CssRegularDimension {
    fn render_html(&self) -> String {
        let unit_type = self.unit_token().unwrap().to_string();
        let value = self.value_token().unwrap().to_string();
        format!(
            "<div data-kind=\"unit\" data-unit-type=\"{}\">{}</div>",
            unit_type,
            render_value(value)
        )
    }
}

impl Render for biome_css_syntax::CssPercentage {
    fn render_html(&self) -> String {
        let value = self.value_token().unwrap().to_string();
        format!(
            "<div data-kind=\"unit\" data-unit-type=\"percentage\">{}</div>",
            render_value(value)
        )
    }
}

impl Render for biome_css_syntax::AnyCssDimension {
    fn render_html(&self) -> String {
        match self {
            biome_css_syntax::AnyCssDimension::CssPercentage(node) => node.render_html(),
            biome_css_syntax::AnyCssDimension::CssRegularDimension(node) => node.render_html(),
            biome_css_syntax::AnyCssDimension::CssUnknownDimension(_) => todo!(),
        }
    }
}

impl Render for biome_css_syntax::CssGenericProperty {
    fn render_html(&self) -> String {
        let name = self.name().unwrap().to_string();
        assert!(self.value().into_iter().into_iter().len() == 1);
        let value = self.value().into_iter().next().unwrap();
        let number = value
            .as_any_css_value()
            .unwrap()
            .as_any_css_dimension()
            .unwrap();

        format!(
            "<div data-kind=\"property\"><div data-attr=\"name\">{}</div><div data-attr=\"value\">{}</div></div>",
            render_value(name.trim().to_string()),
            number.render_html()
        )
    }
}

impl Render for biome_css_syntax::CssQualifiedRule {
    fn render_html(&self) -> String {
        assert!(self.prelude().into_iter().collect::<Vec<_>>().len() == 1);
        let selector = self.prelude().into_iter().next().unwrap().unwrap();

        let b = self.block().unwrap();
        let block = b.as_css_declaration_or_rule_block().unwrap().items();
        assert!(block.clone().into_iter().len() == 1);
        let p = block.into_iter().next().unwrap();
        let properties = p
            .as_css_declaration_with_semicolon()
            .unwrap()
            .declaration()
            .unwrap();

        let selector = format!(
            "<div data-attr=\"selector\">{}</div>",
            selector.as_css_compound_selector().unwrap().render_html()
        );
        let properties = format!(
            "<div data-attr=\"properties\">{}</div>",
            properties
                .property()
                .unwrap()
                .as_css_generic_property()
                .unwrap()
                .render_html()
        );

        format!("<div data-kind=\"rule\">{}{}</div>", selector, properties)
    }
}

impl Render for biome_css_syntax::CssRoot {
    fn render_html(&self) -> String {
        self.rules()
            .into_iter()
            .map(|r| r.as_css_qualified_rule().unwrap().render_html())
            .collect()
    }
}
