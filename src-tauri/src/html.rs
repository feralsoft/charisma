use biome_css_syntax::{
    AnyCssDimension, AnyCssExpression, AnyCssFunction, AnyCssGenericComponentValue,
    AnyCssPseudoClass, AnyCssPseudoClassNth, AnyCssPseudoClassNthSelector, AnyCssRelativeSelector,
    AnyCssSelector, AnyCssSimpleSelector, AnyCssSubSelector, AnyCssValue, CssAttributeSelector,
    CssBinaryExpression, CssClassSelector, CssColor, CssComplexSelector, CssComponentValueList,
    CssCompoundSelector, CssCustomIdentifier, CssDashedIdentifier, CssFunction, CssIdSelector,
    CssIdentifier, CssNumber, CssParameter, CssParenthesizedExpression, CssPercentage,
    CssPseudoClassFunctionCompoundSelector, CssPseudoClassFunctionCompoundSelectorList,
    CssPseudoClassFunctionIdentifier, CssPseudoClassFunctionNth,
    CssPseudoClassFunctionRelativeSelectorList, CssPseudoClassFunctionSelector,
    CssPseudoClassFunctionSelectorList, CssPseudoClassFunctionValueList, CssPseudoClassIdentifier,
    CssPseudoClassNth, CssPseudoClassNthIdentifier, CssPseudoClassNthNumber,
    CssPseudoClassNthSelector, CssPseudoClassSelector, CssPseudoElementSelector, CssRatio,
    CssRegularDimension, CssRelativeSelector, CssSelectorList, CssString, CssTypeSelector,
    CssUniversalSelector, CssUrlFunction,
};

use crate::{get_combinator_type, parse_utils::parse_property, Combinator, Property, State};

pub fn render_value(value: &str) -> String {
    format!(
        "<div data-value=\"{}\">{}</div>",
        value.trim(),
        value.trim()
    )
}

pub struct RenderOptions {
    pub attrs: Vec<(String, String)>,
}

impl RenderOptions {
    pub fn default() -> Self {
        Self { attrs: vec![] }
    }
}

impl ToString for RenderOptions {
    fn to_string(&self) -> String {
        self.attrs
            .iter()
            .map(|(key, value)| format!("{}='{}'", key, value))
            .collect()
    }
}

pub trait Render {
    fn render_html(&self, options: &RenderOptions) -> String;
}

impl Render for AnyCssSelector {
    fn render_html(&self, _options: &RenderOptions) -> String {
        let options = RenderOptions {
            attrs: vec![("data-string-value".to_string(), self.to_string())],
        };
        match self {
            AnyCssSelector::CssBogusSelector(_) => panic!(),
            AnyCssSelector::CssComplexSelector(s) => s.render_html(&options),
            AnyCssSelector::CssCompoundSelector(s) => s.render_html(&options),
        }
    }
}

impl Render for CssComplexSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        let left = self.left().unwrap();
        let right = self.right().unwrap();
        let combinator = self.combinator().unwrap();

        format!(
            "
            <div data-kind=\"complex-selector\" data-combinator-type=\"{}\" data-string-value='{}'>
                <div data-attr=\"left\">{}</div>
                <div data-attr=\"right\">{}</div>
            </div>",
            match get_combinator_type(combinator.kind()) {
                Combinator::Descendant => "descendant",
                Combinator::DirectDescendant => "direct-descendant",
                Combinator::Plus => "next-sibling",
                Combinator::And => panic!(""),
            },
            self.to_string().trim(),
            left.render_html(options),
            right.render_html(options)
        )
    }
}

impl Render for CssClassSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        let name = self.name().unwrap();
        format!(
            "<div data-kind=\"class\" {}>{}</div>",
            options.to_string(),
            render_value(name.to_string().trim())
        )
    }
}

impl Render for CssPseudoClassIdentifier {
    fn render_html(&self, options: &RenderOptions) -> String {
        let name = self.name().unwrap();
        format!(
            "<div data-kind=\"pseudo-class-id\" {}>{}</div>",
            options.to_string(),
            render_value(name.to_string().trim())
        )
    }
}

impl Render for CssRelativeSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self.combinator() {
            Some(combinator) => {
                format!(
                    "<div data-kind=\"relative-selector\" data-combinator-type=\"{}\" data-string-value=\"{}\">{}</div>",
                    match get_combinator_type(combinator.kind()) {
                        Combinator::Descendant => "descendant",
                        Combinator::DirectDescendant => "direct-descendant",
                        Combinator::Plus => "next-sibling",
                        Combinator::And => todo!(),
                    },
                    self.to_string(),
                    self.selector().unwrap().render_html(options)
                )
            }
            None => self.selector().unwrap().render_html(options),
        }
    }
}

impl Render for AnyCssRelativeSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssRelativeSelector::CssBogusSelector(_) => panic!(),
            AnyCssRelativeSelector::CssRelativeSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassFunctionRelativeSelectorList {
    fn render_html(&self, options: &RenderOptions) -> String {
        let name = self.name_token().unwrap();
        assert!(self.relative_selectors().into_iter().count() == 1);

        let selector = self
            .relative_selectors()
            .into_iter()
            .next()
            .unwrap()
            .unwrap();

        format!(
            "<div data-kind=\"pseudo-class-function\" data-string-value=\"{}\">
                <div data-attr=\"function-name\">{}</div>
                <div data-attr=\"args\">{}</div>
            </div>",
            self.to_string(),
            render_value(name.text_trimmed()),
            selector.render_html(options)
        )
    }
}

impl Render for CssPseudoClassNth {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}

impl Render for CssPseudoClassNthIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}

impl Render for CssPseudoClassNthNumber {
    fn render_html(&self, _options: &RenderOptions) -> String {
        assert!(self.sign().is_none());
        let number = self.value().unwrap();
        format!(
            "
            <div data-kind=\"pseudo-class-nth-number\" data-string-value=\"{}\">
                {}
            </div>
            ",
            self.to_string(),
            render_value(&number.to_string())
        )
    }
}

impl Render for AnyCssPseudoClassNth {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssPseudoClassNth::CssPseudoClassNth(s) => s.render_html(options),
            AnyCssPseudoClassNth::CssPseudoClassNthIdentifier(s) => s.render_html(options),
            AnyCssPseudoClassNth::CssPseudoClassNthNumber(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassNthSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        assert!(self.of_selector().is_none());
        let nth = self.nth().unwrap();

        nth.render_html(options)
    }
}

impl Render for AnyCssPseudoClassNthSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssPseudoClassNthSelector::CssBogusSelector(_) => panic!(),
            AnyCssPseudoClassNthSelector::CssPseudoClassNthSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassFunctionNth {
    fn render_html(&self, options: &RenderOptions) -> String {
        let name = self.name().unwrap();
        let selector = self.selector().unwrap();
        format!(
            "<div data-kind=\"pseudo-class-function-nth\" data-string-value='{}'>
                    <div data-attr=\"name\">{}</div>
                    <div data-attr=\"selector\">{}</div>
                </div>",
            self.to_string(),
            render_value(name.text_trimmed()),
            selector.render_html(options)
        )
    }
}

impl Render for CssPseudoClassFunctionCompoundSelector {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionCompoundSelectorList {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionSelector {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionSelectorList {
    fn render_html(&self, _options: &RenderOptions) -> String {
        let name = self.name().unwrap();
        format!(
            "
        <div data-kind=\"pseudo-class-function-selector\" data-string-value=\"{}\">
            <div data-attr=\"name\">{}</div>
            <div data-attr=\"selectors\">{}</div>
        </div>
            ",
            self.to_string(),
            render_value(name.text_trimmed()),
            self.selectors()
                .into_iter()
                .map(|s| s.unwrap().render_html(&RenderOptions::default()))
                .collect::<String>()
        )
    }
}
impl Render for CssPseudoClassFunctionValueList {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}

impl Render for CssPseudoClassSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self.class().unwrap() {
            AnyCssPseudoClass::CssBogusPseudoClass(s) => {
                panic!("bogus pseudo class = {:?}", s.items())
            }
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelector(s) => s.render_html(options),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelectorList(s) => {
                s.render_html(options)
            }
            AnyCssPseudoClass::CssPseudoClassFunctionIdentifier(s) => s.render_html(options),
            AnyCssPseudoClass::CssPseudoClassFunctionNth(s) => s.render_html(options),
            AnyCssPseudoClass::CssPseudoClassFunctionRelativeSelectorList(s) => {
                s.render_html(options)
            }
            AnyCssPseudoClass::CssPseudoClassFunctionSelector(s) => s.render_html(options),
            AnyCssPseudoClass::CssPseudoClassFunctionSelectorList(s) => s.render_html(options),
            AnyCssPseudoClass::CssPseudoClassFunctionValueList(s) => s.render_html(options),
            AnyCssPseudoClass::CssPseudoClassIdentifier(id) => id.render_html(options),
        }
    }
}

impl Render for CssAttributeSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self.matcher() {
            Some(matcher) => {
                let name = self.name().unwrap();
                assert!(matcher.modifier().is_none());
                let operator = matcher.operator().unwrap();
                let value = matcher.value().unwrap().name();

                format!(
                    "
                <div data-kind=\"attribute-selector\" {}>
                    <div data-attr=\"name\">{}</div>
                    <div data-attr=\"operator\">{}</div>
                    <div data-attr=\"value\">{}</div>
                </div>",
                    options.to_string(),
                    render_value(name.to_string().trim()),
                    render_value(operator.text_trimmed()),
                    render_value(value.unwrap().to_string().trim())
                )
            }
            None => {
                let name = self.name().unwrap();
                format!(
                    "
                <div data-kind=\"attribute-selector\" {}>
                    <div data-attr=\"name\">{}</div>
                </div>",
                    options.to_string(),
                    render_value(name.to_string().trim())
                )
            }
        }
    }
}

impl Render for CssPseudoElementSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        let element = self.element().unwrap();
        let element = element.as_css_pseudo_element_identifier().unwrap();
        format!(
            "<div data-kind=\"pseudo-element-selector\" {}>{}</div>",
            options.to_string(),
            render_value(element.name().unwrap().to_string().trim())
        )
    }
}

impl Render for CssIdSelector {
    fn render_html(&self, _options: &RenderOptions) -> String {
        let name = self.name().unwrap();
        let name = name.value_token().unwrap();

        format!(
            "<div data-kind=\"id-selector\" data-string-value=\"{}\">
                <div data-attr=\"name\">{}</div>
            </div>",
            self.to_string(),
            render_value(name.text_trimmed())
        )
    }
}

impl Render for AnyCssSubSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssSubSelector::CssAttributeSelector(attribute_selector) => {
                attribute_selector.render_html(options)
            }
            AnyCssSubSelector::CssBogusSubSelector(_) => panic!(),
            AnyCssSubSelector::CssClassSelector(class) => class.render_html(options),
            AnyCssSubSelector::CssIdSelector(s) => s.render_html(options),
            AnyCssSubSelector::CssPseudoClassSelector(pseudo_class) => {
                pseudo_class.render_html(options)
            }
            AnyCssSubSelector::CssPseudoElementSelector(pseudo_element) => {
                pseudo_element.render_html(options)
            }
        }
    }
}

impl Render for CssTypeSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        assert!(self.namespace().is_none());
        let name = self.ident().unwrap();
        name.render_html(options)
    }
}

impl Render for CssUniversalSelector {
    fn render_html(&self, _options: &RenderOptions) -> String {
        format!("<div data-kind=\"universal-selector\"></div>")
    }
}

impl Render for AnyCssSimpleSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssSimpleSelector::CssTypeSelector(node) => node.render_html(options),
            AnyCssSimpleSelector::CssUniversalSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssCompoundSelector {
    fn render_html(&self, options: &RenderOptions) -> String {
        assert!(self.nesting_selector_token().is_none());

        // simple selector is either an element/type selector eg. `body`, or `*` the universal selector
        // I don't understand at the moment why it is separate from compound selector thing, but we're
        // just going to prepend it onto the list of sub_selectors
        let simple_selector_html = self
            .simple_selector()
            .map(|s| s.render_html(options))
            .unwrap_or(String::from(""));

        let sub_selectors = self.sub_selectors().into_iter().collect::<Vec<_>>();

        if sub_selectors.len() == 1 && self.simple_selector().is_none() {
            sub_selectors[0].render_html(options)
        } else if sub_selectors.is_empty() && self.simple_selector().is_some() {
            simple_selector_html
        } else {
            format!(
                "<div data-kind=\"compound-selector\" {}>{}{}</div>",
                options.to_string(),
                simple_selector_html,
                sub_selectors
                    .iter()
                    .map(|selector| selector.render_html(options))
                    .collect::<String>()
            )
        }
    }
}

impl Render for CssRegularDimension {
    fn render_html(&self, options: &RenderOptions) -> String {
        let unit_type = self.unit_token().unwrap().to_string();
        let value = self.value_token().unwrap().to_string();
        format!(
            "<div data-kind=\"unit\" data-unit-type=\"{}\" {}>{}</div>",
            unit_type,
            options.to_string(),
            render_value(value.trim())
        )
    }
}

impl Render for CssPercentage {
    fn render_html(&self, options: &RenderOptions) -> String {
        let value = self.value_token().unwrap().to_string();
        format!(
            "<div data-kind=\"unit\" data-unit-type=\"percentage\" {}>{}</div>",
            options.to_string(),
            render_value(value.trim())
        )
    }
}

impl Render for AnyCssDimension {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssDimension::CssPercentage(node) => node.render_html(options),
            AnyCssDimension::CssRegularDimension(node) => node.render_html(options),
            AnyCssDimension::CssUnknownDimension(_) => panic!(),
        }
    }
}

impl Render for CssIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> String {
        let value = self.value_token().unwrap();
        format!(
            "<div data-kind=\"identifier\" data-string-value=\"{}\">{}</div>",
            value.text_trimmed(),
            render_value(value.text_trimmed())
        )
    }
}

impl Render for CssComponentValueList {
    fn render_html(&self, options: &RenderOptions) -> String {
        self.into_iter()
            .map(|item| item.render_html(options))
            .collect()
    }
}

impl Render for CssBinaryExpression {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}

impl Render for CssParenthesizedExpression {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}

impl Render for AnyCssExpression {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssExpression::CssBinaryExpression(s) => s.render_html(options),
            AnyCssExpression::CssListOfComponentValuesExpression(list) => {
                list.css_component_value_list().render_html(options)
            }
            AnyCssExpression::CssParenthesizedExpression(s) => s.render_html(options),
        }
    }
}

impl Render for CssParameter {
    fn render_html(&self, options: &RenderOptions) -> String {
        self.any_css_expression().unwrap().render_html(options)
    }
}

impl Render for CssFunction {
    fn render_html(&self, options: &RenderOptions) -> String {
        let function_name = self.name().unwrap().render_html(options);
        let args = self
            .items()
            .into_iter()
            .map(|item| item.unwrap().render_html(options))
            .collect::<String>();

        format!(
            "
        <div data-kind=\"function\" {}>
            <div data-attr=\"name\">{}</div>
            <div data-attr=\"args\">{}</div>
        </div>
        ",
            options.to_string(),
            function_name,
            args
        )
    }
}

impl Render for CssUrlFunction {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}

impl Render for AnyCssFunction {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssFunction::CssFunction(f) => f.render_html(options),
            AnyCssFunction::CssUrlFunction(s) => s.render_html(options),
        }
    }
}

impl Render for CssNumber {
    fn render_html(&self, options: &RenderOptions) -> String {
        let value = self.value_token().unwrap();
        let value = value.text_trimmed();

        format!(
            "<div data-kind=\"number\" {}>{}</div>",
            options.to_string(),
            render_value(value)
        )
    }
}

impl Render for CssDashedIdentifier {
    fn render_html(&self, options: &RenderOptions) -> String {
        let value = self.value_token().unwrap();
        let value = value.text_trimmed();
        format!(
            "<div data-kind=\"dashed-id\" {}>{}</div>",
            options.to_string(),
            render_value(value)
        )
    }
}

impl Render for CssColor {
    fn render_html(&self, options: &RenderOptions) -> String {
        let hash = self.as_fields().hash_token.unwrap();
        let hash = hash.text_trimmed();
        let value = self.as_fields().value_token.unwrap();
        let value = value.text_trimmed();
        // todo: wtf is this hash?
        format!(
            "<div data-kind=\"color\" data-hash=\"{}\" {}>{}</div>",
            hash,
            options.to_string(),
            render_value(value)
        )
    }
}

impl Render for CssString {
    fn render_html(&self, options: &RenderOptions) -> String {
        let str = self.value_token().unwrap();
        let str = str.text();
        format!(
            "<div data-kind=\"string\" {}>{}</div>",
            options.to_string(),
            render_value(str)
        )
    }
}

impl Render for CssRatio {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}

impl Render for CssCustomIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> String {
        todo!()
    }
}

impl Render for AnyCssValue {
    fn render_html(&self, _options: &RenderOptions) -> String {
        let options = RenderOptions {
            attrs: vec![("data-string-value".to_string(), self.to_string())],
        };
        match self {
            AnyCssValue::AnyCssDimension(dim) => dim.render_html(&options),
            AnyCssValue::AnyCssFunction(f) => f.render_html(&options),
            AnyCssValue::CssColor(color) => color.render_html(&options),
            AnyCssValue::CssCustomIdentifier(s) => s.render_html(&options),
            AnyCssValue::CssDashedIdentifier(id) => id.render_html(&options),
            AnyCssValue::CssIdentifier(id) => id.render_html(&options),
            AnyCssValue::CssNumber(num) => num.render_html(&options),
            AnyCssValue::CssRatio(s) => s.render_html(&options),
            AnyCssValue::CssString(css_string) => css_string.render_html(&options),
        }
    }
}

impl Render for AnyCssGenericComponentValue {
    fn render_html(&self, options: &RenderOptions) -> String {
        match self {
            AnyCssGenericComponentValue::AnyCssValue(node) => node.render_html(options),
            // these are just commas I believe
            AnyCssGenericComponentValue::CssGenericDelimiter(_) => String::from(""),
        }
    }
}

impl Render for CssSelectorList {
    fn render_html(&self, options: &RenderOptions) -> String {
        let string_value = self
            .into_iter()
            .map(|s| s.unwrap().to_string())
            .reduce(|acc, cur| acc + ", " + &cur)
            .unwrap();

        format!(
            "<div data-kind=\"selector-list\" data-string-value='{}'>
                <div data-attr=\"selectors\">{}</div>
            </div>",
            string_value,
            self.into_iter()
                .map(|s| s.unwrap().render_html(options))
                .collect::<String>()
        )
    }
}

impl Render for Property {
    fn render_html(&self, options: &RenderOptions) -> String {
        let property = parse_property(&format!("{}: {};", self.name, self.value)).unwrap();
        let property = property.declaration().unwrap().property().unwrap();
        let property = property.as_css_generic_property().unwrap();

        let name = self.name.clone();
        let value = if property.value().into_iter().len() == 1 {
            let value = property.value().into_iter().next().unwrap();
            let value = value.as_any_css_value().unwrap();
            value.render_html(options)
        } else {
            format!(
                "<div data-kind=\"multi-part-value\" data-len=\"{}\" data-string-value='{}'>{}</div>",
                property.value().into_iter().count(),
                property
                    .value()
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect::<String>(),
                property
                    .value()
                    .into_iter()
                    .map(|value| value.render_html(options))
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
