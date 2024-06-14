use biome_css_syntax::{
    AnyCssDimension, AnyCssExpression, AnyCssFunction, AnyCssPseudoClass, AnyCssSelector,
    AnyCssSubSelector, AnyCssValue, CssAttributeSelector, CssClassSelector, CssColor,
    CssComplexSelector, CssComponentValueList, CssCompoundSelector, CssDashedIdentifier,
    CssDeclarationWithSemicolon, CssFunction, CssGenericProperty, CssIdentifier, CssNumber,
    CssParameter, CssPercentage, CssPseudoClassIdentifier, CssPseudoClassSelector,
    CssPseudoElementSelector, CssQualifiedRule, CssRegularDimension, CssRoot, CssString,
};

use crate::{parse_utils::get_combinator_type, Property, State};

pub fn render_value(value: &str) -> String {
    format!(
        "<div data-value=\"{}\" contenteditable>{}</div>",
        value, value
    )
}

pub fn render_kind(kind: &str, children: Vec<String>) -> String {
    format!("<div data-kind=\"{}\">{}</div>", kind, children.join(""))
}

pub fn render_attr(name: &str, child: String) -> String {
    format!("<div data-attr=\"{}\">{}</div>", name, child)
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

impl Render for CssClassSelector {
    fn render_html(&self) -> String {
        let name = self.name().unwrap();
        format!(
            "<div data-kind=\"class\">{}</div>",
            render_value(name.to_string().trim())
        )
    }
}

impl Render for CssPseudoClassIdentifier {
    fn render_html(&self) -> String {
        let name = self.name().unwrap();
        format!(
            "<div data-kind=\"pseudo-class-id\">{}</div>",
            render_value(name.to_string().trim())
        )
    }
}

impl Render for CssPseudoClassSelector {
    fn render_html(&self) -> String {
        match self.class().unwrap() {
            AnyCssPseudoClass::CssBogusPseudoClass(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelector(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelectorList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionIdentifier(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionNth(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionRelativeSelectorList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelector(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelectorList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionValueList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassIdentifier(id) => id.render_html(),
        }
    }
}

impl Render for CssAttributeSelector {
    fn render_html(&self) -> String {
        match self.matcher() {
            Some(matcher) => {
                let name = self.name().unwrap();
                assert!(matcher.modifier().is_none());
                let operator = matcher.operator().unwrap();
                let value = matcher.value().unwrap().name();

                format!(
                    "
                <div data-kind=\"attribute-selector\">
                    <div data-attr=\"name\">{}</div>
                    <div data-attr=\"operator\">{}</div>
                    <div data-attr=\"value\">{}</div>
                </div>",
                    render_value(name.to_string().trim()),
                    render_value(operator.text_trimmed()),
                    render_value(value.unwrap().to_string().trim())
                )
            }
            None => {
                let name = self.name().unwrap();
                format!(
                    "
                <div data-kind=\"attribute-selector\">
                    <div data-attr=\"name\">{}</div>
                </div>",
                    render_value(name.to_string().trim())
                )
            }
        }
    }
}

impl Render for CssPseudoElementSelector {
    fn render_html(&self) -> String {
        let element = self.element().unwrap();
        let element = element.as_css_pseudo_element_identifier().unwrap();
        format!(
            "<div data-kind=\"pseudo-element-selector\">{}</div>",
            render_value(element.name().unwrap().to_string().trim())
        )
    }
}

impl Render for AnyCssSubSelector {
    fn render_html(&self) -> String {
        match self {
            AnyCssSubSelector::CssAttributeSelector(attribute_selector) => {
                attribute_selector.render_html()
            }
            AnyCssSubSelector::CssBogusSubSelector(_) => todo!(),
            AnyCssSubSelector::CssClassSelector(class) => class.render_html(),
            AnyCssSubSelector::CssIdSelector(_) => todo!(),
            AnyCssSubSelector::CssPseudoClassSelector(pseudo_class) => pseudo_class.render_html(),
            AnyCssSubSelector::CssPseudoElementSelector(pseudo_element) => {
                pseudo_element.render_html()
            }
        }
    }
}
impl Render for CssCompoundSelector {
    fn render_html(&self) -> String {
        assert!(self.simple_selector().is_none());
        assert!(self.nesting_selector_token().is_none());

        match self
            .sub_selectors()
            .into_iter()
            .collect::<Vec<_>>()
            .as_slice()
        {
            [selector] => selector.render_html(),
            selectors => {
                format!(
                    "<div data-kind=\"compound-selector\">{}</div>",
                    selectors
                        .iter()
                        .map(|selector| selector.render_html())
                        .collect::<String>()
                )
            }
        }
    }
}

impl Render for CssRegularDimension {
    fn render_html(&self) -> String {
        let unit_type = self.unit_token().unwrap().to_string();
        let value = self.value_token().unwrap().to_string();
        format!(
            "<div data-kind=\"unit\" data-unit-type=\"{}\">{}</div>",
            unit_type,
            render_value(value.trim())
        )
    }
}

impl Render for CssPercentage {
    fn render_html(&self) -> String {
        let value = self.value_token().unwrap().to_string();
        format!(
            "<div data-kind=\"unit\" data-unit-type=\"percentage\">{}</div>",
            render_value(value.trim())
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
        let value = self.value_token().unwrap();
        format!(
            "<div data-kind=\"identifier\">{}</div>",
            render_value(value.text_trimmed())
        )
    }
}

impl Render for CssComponentValueList {
    fn render_html(&self) -> String {
        self.into_iter().map(|item| item.render_html()).collect()
    }
}

impl Render for AnyCssExpression {
    fn render_html(&self) -> String {
        match self {
            AnyCssExpression::CssBinaryExpression(_) => todo!(),
            AnyCssExpression::CssListOfComponentValuesExpression(list) => {
                list.css_component_value_list().render_html()
            }
            AnyCssExpression::CssParenthesizedExpression(_) => todo!(),
        }
    }
}

impl Render for CssParameter {
    fn render_html(&self) -> String {
        self.any_css_expression().unwrap().render_html()
    }
}

impl Render for CssFunction {
    fn render_html(&self) -> String {
        let function_name = self.name().unwrap().render_html();
        let args = self
            .items()
            .into_iter()
            .map(|item| item.unwrap().render_html())
            .collect::<String>();

        format!(
            "
        <div data-kind=\"function\">
            <div data-attr=\"name\">{}</div>
            <div data-attr=\"args\">{}</div>
        </div>
        ",
            function_name, args
        )
    }
}

impl Render for AnyCssFunction {
    fn render_html(&self) -> String {
        match self {
            AnyCssFunction::CssFunction(f) => f.render_html(),
            AnyCssFunction::CssUrlFunction(_) => todo!(),
        }
    }
}

impl Render for CssNumber {
    fn render_html(&self) -> String {
        let value = self.value_token().unwrap();
        let value = value.text_trimmed();

        format!("<div data-kind=\"number\">{}</div>", render_value(value))
    }
}

impl Render for CssDashedIdentifier {
    fn render_html(&self) -> String {
        let value = self.value_token().unwrap();
        let value = value.text_trimmed();
        format!("<div data-kind=\"dashed-id\">{}</div>", render_value(value))
    }
}

impl Render for CssColor {
    fn render_html(&self) -> String {
        let hash = self.as_fields().hash_token.unwrap();
        let hash = hash.text_trimmed();
        let value = self.as_fields().value_token.unwrap();
        let value = value.text_trimmed();
        // todo: wtf is this hash?
        format!(
            "<div data-kind=\"color\" data-hash=\"{}\">{}</div>",
            hash,
            render_value(value)
        )
    }
}

impl Render for CssString {
    fn render_html(&self) -> String {
        let str = self.value_token().unwrap();
        let str = str.text();
        format!("<div data-kind=\"string\">{}</div>", render_value(str))
    }
}

impl Render for AnyCssValue {
    fn render_html(&self) -> String {
        match self {
            AnyCssValue::AnyCssDimension(dim) => dim.render_html(),
            AnyCssValue::AnyCssFunction(f) => f.render_html(),
            AnyCssValue::CssColor(color) => color.render_html(),
            AnyCssValue::CssCustomIdentifier(_) => todo!(),
            AnyCssValue::CssDashedIdentifier(id) => id.render_html(),
            AnyCssValue::CssIdentifier(id) => id.render_html(),
            AnyCssValue::CssNumber(num) => num.render_html(),
            AnyCssValue::CssRatio(_) => todo!(),
            AnyCssValue::CssString(css_string) => css_string.render_html(),
        }
    }
}

impl Render for Property {
    fn render_html(&self) -> String {
        let property = self.node.declaration().unwrap().property().unwrap();
        let property = property.as_css_generic_property().unwrap();

        let name = self.name();
        let value = if property.value().into_iter().len() == 1 {
            let value = property.value().into_iter().next().unwrap();
            let value = value.as_any_css_value().unwrap();
            value.render_html()
        } else {
            format!(
                "<div data-kind=\"multi-part-value\">{}</div>",
                property
                    .value()
                    .into_iter()
                    .map(|value| value.as_any_css_value().unwrap().render_html())
                    .collect::<String>()
            )
        };
        let property_kind = if name.starts_with("--") {
            "variable"
        } else {
            "property"
        };

        format!(
            "<div data-kind=\"property\" data-property-kind=\"{}\" data-commented=\"{}\">
                <div data-attr=\"name\">{}</div>
                <div data-attr=\"value\">{}</div>
            </div>",
            property_kind,
            self.state == State::Commented,
            render_value(&name),
            value
        )
    }
}
