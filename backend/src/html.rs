use biome_css_syntax::{
    AnyCssDimension, AnyCssSelector, AnyCssSubSelector, AnyCssValue, CssComplexSelector,
    CssCompoundSelector, CssDeclarationWithSemicolon, CssGenericProperty, CssIdentifier,
    CssPercentage, CssQualifiedRule, CssRegularDimension, CssRoot, CssSyntaxKind,
};

use crate::parse_utils::get_combinator_type;

pub fn render_value(value: String) -> String {
    format!(
        "<div data-value=\"{}\" contenteditable>{}</div>",
        value, value
    )
}

pub trait Render {
    fn render_html(&self) -> String;
}

impl Render for AnyCssSelector {
    fn render_html(&self) -> String {
        match self {
            AnyCssSelector::CssBogusSelector(_) => todo!(),
            AnyCssSelector::CssComplexSelector(s) => s.render_html(),
            AnyCssSelector::CssCompoundSelector(s) => s.render_html(),
        }
    }
}

impl Render for CssComplexSelector {
    fn render_html(&self) -> String {
        let left = self.left().unwrap();
        let right = self.right().unwrap();
        let combinator = self.combinator().unwrap();

        format!(
            "
            <div data-kind=\"complex-selector\" data-combinator-type=\"{}\">
                <div data-attr=\"left\">{}</div>
                <div data-attr=\"right\">{}</div>
            </div>",
            get_combinator_type(combinator.kind()),
            left.render_html(),
            right.render_html()
        )
    }
}

impl Render for CssCompoundSelector {
    fn render_html(&self) -> String {
        assert!(self.sub_selectors().into_iter().len() == 1);
        let selector = self.sub_selectors().into_iter().next().unwrap();
        let (name, kind) = match selector {
            AnyCssSubSelector::CssAttributeSelector(_) => todo!(),
            AnyCssSubSelector::CssBogusSubSelector(_) => todo!(),
            AnyCssSubSelector::CssClassSelector(class) => {
                (class.name().unwrap().to_string(), "class")
            }
            AnyCssSubSelector::CssIdSelector(_) => todo!(),
            AnyCssSubSelector::CssPseudoClassSelector(_) => todo!(),
            AnyCssSubSelector::CssPseudoElementSelector(_) => todo!(),
        };

        format!(
            "<div data-kind=\"{}\">{}</div>",
            kind,
            render_value(name.trim().to_string())
        )
    }
}

impl Render for CssRegularDimension {
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

impl Render for CssPercentage {
    fn render_html(&self) -> String {
        let value = self.value_token().unwrap().to_string();
        format!(
            "<div data-kind=\"unit\" data-unit-type=\"percentage\">{}</div>",
            render_value(value)
        )
    }
}

impl Render for AnyCssDimension {
    fn render_html(&self) -> String {
        match self {
            AnyCssDimension::CssPercentage(node) => node.render_html(),
            AnyCssDimension::CssRegularDimension(node) => node.render_html(),
            AnyCssDimension::CssUnknownDimension(_) => todo!(),
        }
    }
}

impl Render for CssIdentifier {
    fn render_html(&self) -> String {
        self.value_token().unwrap().text_trimmed().to_string()
    }
}

impl Render for AnyCssValue {
    fn render_html(&self) -> String {
        match self {
            AnyCssValue::AnyCssDimension(dim) => dim.render_html(),
            AnyCssValue::AnyCssFunction(_) => todo!(),
            AnyCssValue::CssColor(_) => todo!(),
            AnyCssValue::CssCustomIdentifier(_) => todo!(),
            AnyCssValue::CssDashedIdentifier(_) => todo!(),
            AnyCssValue::CssIdentifier(id) => id.render_html(),
            AnyCssValue::CssNumber(_) => todo!(),
            AnyCssValue::CssRatio(_) => todo!(),
            AnyCssValue::CssString(_) => todo!(),
        }
    }
}

impl Render for CssGenericProperty {
    fn render_html(&self) -> String {
        let name = self.name().unwrap().to_string();
        assert!(self.value().into_iter().into_iter().len() == 1);
        let value = self.value().into_iter().next().unwrap();
        let value = value.as_any_css_value().unwrap();
        let property_kind = if name.starts_with("--") {
            "variable"
        } else {
            "property"
        };

        format!(
            "<div data-kind=\"property\" data-property-kind=\"{}\"><div data-attr=\"name\">{}</div><div data-attr=\"value\">{}</div></div>",
            property_kind,
            render_value(name.to_string().trim().to_string()),
            value.render_html()
        )
    }
}

impl Render for CssDeclarationWithSemicolon {
    fn render_html(&self) -> String {
        self.declaration()
            .unwrap()
            .property()
            .unwrap()
            .as_css_generic_property()
            .unwrap()
            .render_html()
    }
}

impl Render for CssQualifiedRule {
    fn render_html(&self) -> String {
        assert!(self.prelude().into_iter().collect::<Vec<_>>().len() == 1);
        let selector = self.prelude().into_iter().next().unwrap().unwrap();

        let b = self.block().unwrap();
        let items = b.as_css_declaration_or_rule_block().unwrap().items();
        let properties = items
            .into_iter()
            .map(|item| {
                item.as_css_declaration_with_semicolon()
                    .unwrap()
                    .render_html()
            })
            .collect::<String>();

        println!("{:?}", selector);

        let selector = format!(
            "<div data-attr=\"selector\">{}</div>",
            selector.render_html()
        );
        let properties = format!("<div data-attr=\"properties\">{}</div>", properties);

        format!("<div data-kind=\"rule\">{}{}</div>", selector, properties)
    }
}

impl Render for CssRoot {
    fn render_html(&self) -> String {
        self.rules()
            .into_iter()
            .map(|r| r.as_css_qualified_rule().unwrap().render_html())
            .collect()
    }
}
