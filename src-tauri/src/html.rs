use biome_css_syntax::{
    AnyCssDimension, AnyCssExpression, AnyCssFunction, AnyCssGenericComponentValue,
    AnyCssPseudoClass, AnyCssPseudoClassNth, AnyCssPseudoClassNthSelector, AnyCssRelativeSelector,
    AnyCssSelector, AnyCssSimpleSelector, AnyCssSubSelector, AnyCssValue, CssAttributeSelector,
    CssBinaryExpression, CssClassSelector, CssColor, CssComplexSelector, CssComponentValueList,
    CssCompoundSelector, CssCustomIdentifier, CssDashedIdentifier, CssFunction, CssIdSelector,
    CssIdentifier, CssNthOffset, CssNumber, CssParameter, CssParenthesizedExpression,
    CssPercentage, CssPseudoClassFunctionCompoundSelector,
    CssPseudoClassFunctionCompoundSelectorList, CssPseudoClassFunctionIdentifier,
    CssPseudoClassFunctionNth, CssPseudoClassFunctionRelativeSelectorList,
    CssPseudoClassFunctionSelector, CssPseudoClassFunctionSelectorList,
    CssPseudoClassFunctionValueList, CssPseudoClassIdentifier, CssPseudoClassNth,
    CssPseudoClassNthIdentifier, CssPseudoClassNthNumber, CssPseudoClassNthSelector,
    CssPseudoClassSelector, CssPseudoElementSelector, CssRatio, CssRegularDimension,
    CssRelativeSelector, CssSelectorList, CssString, CssTypeSelector, CssUniversalSelector,
    CssUrlFunction,
};
use std::fmt::Display;

use crate::{
    get_combinator_type, parse_utils::parse_property, AtRulePart, CharismaError, Combinator, Frame,
    Part, Property, State,
};

pub fn attr(name: &str, value: &str) -> String {
    format!("{}=\"{}\"", name, value.replace('"', "&quot;").trim())
}

pub fn render_value(value: &str) -> String {
    format!("<div {}>{}</div>", attr("data-value", value), value.trim())
}

pub fn data_string_value(value: &str) -> String {
    attr("data-string-value", value)
}

pub struct RenderOptions {
    pub attrs: Vec<(String, String)>,
}

impl RenderOptions {
    pub fn default() -> Self {
        Self { attrs: vec![] }
    }
}

impl Display for RenderOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.attrs.iter() {
            write!(f, "{}='{}'", key, value)?;
        }
        Ok(())
    }
}

pub trait Render {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError>;
}

impl Render for AnyCssSelector {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        let options = RenderOptions {
            attrs: vec![(
                "data-string-value".to_string().trim().to_string(),
                self.to_string(),
            )],
        };
        match self {
            AnyCssSelector::CssBogusSelector(_) => panic!(),
            AnyCssSelector::CssComplexSelector(s) => s.render_html(&options),
            AnyCssSelector::CssCompoundSelector(s) => s.render_html(&options),
        }
    }
}

impl Render for CssComplexSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let left = self.left().map_err(|_| CharismaError::ParseError)?;
        let right = self.right().map_err(|_| CharismaError::ParseError)?;
        let combinator = self.combinator().map_err(|_| CharismaError::ParseError)?;

        Ok(format!(
            "
            <div data-kind=\"complex-selector\" data-combinator-type=\"{}\" {}>
                <div data-attr=\"left\">{}</div>
                <div data-attr=\"right\">{}</div>
            </div>",
            match get_combinator_type(combinator.kind()) {
                Combinator::Descendant => "descendant",
                Combinator::DirectDescendant => "direct-descendant",
                Combinator::Plus => "next-sibling",
                Combinator::And => panic!(""),
            },
            data_string_value(&self.to_string()),
            left.render_html(options)?,
            right.render_html(options)?
        ))
    }
}

impl Render for CssClassSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let name = self.name().map_err(|_| CharismaError::ParseError)?;
        Ok(format!(
            "<div data-kind=\"class\" {}>{}</div>",
            options,
            render_value(name.to_string().trim())
        ))
    }
}

impl Render for CssPseudoClassIdentifier {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let name = self.name().map_err(|_| CharismaError::ParseError)?;
        Ok(format!(
            "<div data-kind=\"pseudo-class-id\" {}>{}</div>",
            options,
            render_value(name.to_string().trim())
        ))
    }
}

impl Render for CssRelativeSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self.combinator() {
            Some(combinator) => Ok(format!(
                "<div data-kind=\"relative-selector\" data-combinator-type=\"{}\" {}>{}</div>",
                match get_combinator_type(combinator.kind()) {
                    Combinator::Descendant => "descendant",
                    Combinator::DirectDescendant => "direct-descendant",
                    Combinator::Plus => "next-sibling",
                    Combinator::And => todo!(),
                },
                data_string_value(&self.to_string()),
                self.selector()
                    .map_err(|_| CharismaError::ParseError)?
                    .render_html(options)?
            )),
            None => self
                .selector()
                .map_err(|_| CharismaError::ParseError)?
                .render_html(options),
        }
    }
}

impl Render for AnyCssRelativeSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self {
            AnyCssRelativeSelector::CssBogusSelector(_) => Err(CharismaError::ParseError),
            AnyCssRelativeSelector::CssRelativeSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassFunctionRelativeSelectorList {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let name = self.name_token().map_err(|_| CharismaError::ParseError)?;
        assert!(self.relative_selectors().into_iter().count() == 1);

        let selector = match self.relative_selectors().into_iter().next() {
            Some(s) => s.map_err(|_| CharismaError::ParseError)?,
            None => return Err(CharismaError::ParseError),
        };

        Ok(format!(
            "<div data-kind=\"pseudo-class-function\" {}>
                <div data-attr=\"function-name\">{}</div>
                <div data-attr=\"args\">{}</div>
            </div>",
            data_string_value(&self.to_string()),
            render_value(name.text_trimmed()),
            selector.render_html(options)?
        ))
    }
}

impl Render for CssNthOffset {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        let sign = self.sign().unwrap().to_string();
        let value = self.value().unwrap().to_string();

        Ok(format!(
            "
            <div data-kind=\"nth-offset\">
                <div data-attr=\"sign\">{}</div>
                <div data-attr=\"value\">{}</div>
            </div>
            ",
            render_value(sign.trim()),
            render_value(value.trim())
        ))
    }
}

impl Render for CssPseudoClassNth {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let offset = match self.offset() {
            Some(o) => o.render_html(options)?,
            None => String::from(""),
        };

        Ok(format!(
            "
            <div data-kind=\"pseudo-class-nth\" {}>
                <div data-attr=\"value\">{}</div>
                <div data-attr=\"offset\">{}</div>
            </div>
        ",
            data_string_value(&self.to_string()),
            render_value(self.value().unwrap().to_string().trim()),
            offset,
        ))
    }
}

impl Render for CssPseudoClassNthIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}

impl Render for Frame {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let last_part = match self.path.last() {
            Some(p) => p,
            None => return Err(CharismaError::ParseError),
        };
        let selector = match last_part {
            Part::AtRule(AtRulePart::Percentage(pct)) => {
                format!(
                    "<div data-kind=\"keyframes-percentage-selector\">
                        <div data-attr=\"percentage\">{}</div>
                    </div>",
                    render_value(&pct.to_string())
                )
            }
            _ => return Err(CharismaError::ParseError),
        };

        let properties = self
            .properties
            .iter()
            .map(|p| p.render_html(options))
            .collect::<Result<String, _>>()?;

        Ok(format!(
            "<div data-kind=\"frame\">
                <div data-attr=\"selector\">{}</div>
                <div data-attr=\"properties\">{}</div>
            </div>",
            selector, properties
        ))
    }
}

impl Render for CssPseudoClassNthNumber {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        assert!(self.sign().is_none());
        let number = self.value().map_err(|_| CharismaError::ParseError)?;
        Ok(format!(
            "
            <div data-kind=\"pseudo-class-nth-number\" {}>
                {}
            </div>
            ",
            data_string_value(&self.to_string()),
            render_value(&number.to_string())
        ))
    }
}

impl Render for AnyCssPseudoClassNth {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self {
            AnyCssPseudoClassNth::CssPseudoClassNth(s) => s.render_html(options),
            AnyCssPseudoClassNth::CssPseudoClassNthIdentifier(s) => s.render_html(options),
            AnyCssPseudoClassNth::CssPseudoClassNthNumber(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassNthSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        assert!(self.of_selector().is_none());
        let nth = self.nth().map_err(|_| CharismaError::ParseError)?;

        nth.render_html(options)
    }
}

impl Render for AnyCssPseudoClassNthSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self {
            AnyCssPseudoClassNthSelector::CssBogusSelector(_) => panic!(),
            AnyCssPseudoClassNthSelector::CssPseudoClassNthSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassFunctionNth {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let name = self.name().map_err(|_| CharismaError::ParseError)?;
        let selector = self.selector().map_err(|_| CharismaError::ParseError)?;
        Ok(format!(
            "<div data-kind=\"pseudo-class-function-nth\" {}>
                    <div data-attr=\"name\">{}</div>
                    <div data-attr=\"selector\">{}</div>
                </div>",
            data_string_value(&self.to_string()),
            render_value(name.text_trimmed()),
            selector.render_html(options)?
        ))
    }
}

impl Render for CssPseudoClassFunctionCompoundSelector {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionCompoundSelectorList {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionSelector {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionSelectorList {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        let name = self.name().map_err(|_| CharismaError::ParseError)?;

        Ok(format!(
            "
        <div data-kind=\"pseudo-class-function-selector\" {}>
            <div data-attr=\"name\">{}</div>
            <div data-attr=\"selectors\">{}</div>
        </div>
            ",
            data_string_value(&self.to_string()),
            render_value(name.text_trimmed()),
            self.selectors()
                .into_iter()
                .map(|s| s
                    .map_err(|_| CharismaError::ParseError)
                    .and_then(|s| s.render_html(&RenderOptions::default())))
                .collect::<Result<String, _>>()?
        ))
    }
}
impl Render for CssPseudoClassFunctionValueList {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}

impl Render for CssPseudoClassSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self.class().map_err(|_| CharismaError::ParseError)? {
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
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self.matcher() {
            Some(matcher) => {
                let name = self.name().map_err(|_| CharismaError::ParseError)?;
                assert!(matcher.modifier().is_none());
                let operator = matcher.operator().map_err(|_| CharismaError::ParseError)?;
                let value = matcher
                    .value()
                    .map_err(|_| CharismaError::ParseError)?
                    .name();

                Ok(format!(
                    "
                <div data-kind=\"attribute-selector\" {}>
                    <div data-attr=\"name\">{}</div>
                    <div data-attr=\"operator\">{}</div>
                    <div data-attr=\"value\">{}</div>
                </div>",
                    options,
                    render_value(name.to_string().trim()),
                    render_value(operator.text_trimmed()),
                    render_value(
                        value
                            .map_err(|_| CharismaError::ParseError)?
                            .to_string()
                            .trim()
                    )
                ))
            }
            None => {
                let name = self.name().map_err(|_| CharismaError::ParseError)?;
                Ok(format!(
                    "
                <div data-kind=\"attribute-selector\" {}>
                    <div data-attr=\"name\">{}</div>
                </div>",
                    options,
                    render_value(name.to_string().trim())
                ))
            }
        }
    }
}

impl Render for CssPseudoElementSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let element = self.element().map_err(|_| CharismaError::ParseError)?;
        let element = element.as_css_pseudo_element_identifier().unwrap();
        Ok(format!(
            "<div data-kind=\"pseudo-element-selector\" {}>{}</div>",
            options,
            render_value(
                element
                    .name()
                    .map_err(|_| CharismaError::ParseError)?
                    .to_string()
                    .trim()
            )
        ))
    }
}

impl Render for CssIdSelector {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        let name = self.name().map_err(|_| CharismaError::ParseError)?;
        let name = name.value_token().map_err(|_| CharismaError::ParseError)?;

        Ok(format!(
            "<div data-kind=\"id-selector\" {}>
                <div data-attr=\"name\">{}</div>
            </div>",
            data_string_value(&self.to_string()),
            render_value(name.text_trimmed())
        ))
    }
}

impl Render for AnyCssSubSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
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
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        assert!(self.namespace().is_none());
        let name = self.ident().map_err(|_| CharismaError::ParseError)?;
        name.render_html(options)
    }
}

impl Render for CssUniversalSelector {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        Ok("<div data-kind=\"universal-selector\"></div>".to_string())
    }
}

impl Render for AnyCssSimpleSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self {
            AnyCssSimpleSelector::CssTypeSelector(node) => node.render_html(options),
            AnyCssSimpleSelector::CssUniversalSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssCompoundSelector {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        assert!(self.nesting_selector_token().is_none());

        // simple selector is either an element/type selector eg. `body`, or `*` the universal selector
        // I don't understand at the moment why it is separate from compound selector thing, but we're
        // just going to prepend it onto the list of sub_selectors
        let simple_selector_html = self
            .simple_selector()
            .map(|s| s.render_html(options))
            .unwrap_or(Ok(String::from("")))?;

        let sub_selectors = self.sub_selectors().into_iter().collect::<Vec<_>>();

        if sub_selectors.len() == 1 && self.simple_selector().is_none() {
            sub_selectors[0].render_html(options)
        } else if sub_selectors.is_empty() && self.simple_selector().is_some() {
            Ok(simple_selector_html)
        } else {
            Ok(format!(
                "<div data-kind=\"compound-selector\" {}>{}{}</div>",
                options,
                simple_selector_html,
                sub_selectors
                    .iter()
                    .map(|selector| selector.render_html(options))
                    .collect::<Result<String, _>>()?
            ))
        }
    }
}

impl Render for CssRegularDimension {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let unit_type = self
            .unit_token()
            .map_err(|_| CharismaError::ParseError)?
            .to_string();
        let value = self
            .value_token()
            .map_err(|_| CharismaError::ParseError)?
            .to_string();
        Ok(format!(
            "<div data-kind=\"unit\" data-unit-type=\"{}\" {}>{}</div>",
            unit_type.trim(),
            options,
            render_value(value.trim())
        ))
    }
}

impl Render for CssPercentage {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let value = self
            .value_token()
            .map_err(|_| CharismaError::ParseError)?
            .to_string();
        Ok(format!(
            "<div data-kind=\"unit\" data-unit-type=\"percentage\" {}>{}</div>",
            options,
            render_value(value.trim())
        ))
    }
}

impl Render for AnyCssDimension {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self {
            AnyCssDimension::CssPercentage(node) => node.render_html(options),
            AnyCssDimension::CssRegularDimension(node) => node.render_html(options),
            AnyCssDimension::CssUnknownDimension(_) => Err(CharismaError::ParseError),
        }
    }
}

impl Render for CssIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        let value = self.value_token().map_err(|_| CharismaError::ParseError)?;
        Ok(format!(
            "<div data-kind=\"identifier\" {}>{}</div>",
            data_string_value(value.text_trimmed()),
            render_value(value.text_trimmed())
        ))
    }
}

impl Render for CssComponentValueList {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        self.into_iter()
            .map(|item| item.render_html(options))
            .collect()
    }
}

impl Render for CssBinaryExpression {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let left = self.left().map_err(|_| CharismaError::ParseError)?;
        let right = self.right().map_err(|_| CharismaError::ParseError)?;
        let operator = self
            .operator_token()
            .map_err(|_| CharismaError::ParseError)?
            .to_string();
        Ok(format!(
            "<div data-kind=\"binary-expression\">
                <div data-attr=\"left\">{}</div>
                <div data-attr=\"operator\">{}</div>
                <div data-attr=\"right\">{}</div>
            </div>",
            left.render_html(options)?,
            render_value(&operator),
            right.render_html(options)?
        ))
    }
}

impl Render for CssParenthesizedExpression {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}

impl Render for AnyCssExpression {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
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
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        self.any_css_expression().unwrap().render_html(options)
    }
}

impl Render for CssFunction {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let function_name = self
            .name()
            .map_err(|_| CharismaError::ParseError)?
            .render_html(options)?;
        let args = self
            .items()
            .into_iter()
            .map(|item| {
                item.map_err(|_| CharismaError::ParseError)
                    .and_then(|i| i.render_html(options))
            })
            .collect::<Result<String, _>>()?;

        Ok(format!(
            "
        <div data-kind=\"function\" {}>
            <div data-attr=\"name\">{}</div>
            <div data-attr=\"args\">{}</div>
        </div>
        ",
            options, function_name, args
        ))
    }
}

impl Render for CssUrlFunction {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        assert!(self.modifiers().into_iter().len() == 0);
        let name = self.name().map_err(|_| CharismaError::ParseError)?;
        let name = name.text_trimmed();
        let url = self.value().unwrap().to_string();
        Ok(format!(
            "<div data-kind=\"url-function\" {}>
                <div data-attr=\"name\">{}</div>
                <div data-attr=\"value\">{}</div>
            </div>",
            data_string_value(&self.to_string()),
            render_value(name),
            render_value(&url),
        ))
    }
}

impl Render for AnyCssFunction {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self {
            AnyCssFunction::CssFunction(f) => f.render_html(options),
            AnyCssFunction::CssUrlFunction(s) => s.render_html(options),
        }
    }
}

impl Render for CssNumber {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let value = self.value_token().map_err(|_| CharismaError::ParseError)?;
        let value = value.text_trimmed();

        Ok(format!(
            "<div data-kind=\"number\" {}>{}</div>",
            options,
            render_value(value)
        ))
    }
}

impl Render for CssDashedIdentifier {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let value = self.value_token().map_err(|_| CharismaError::ParseError)?;
        let value = value.text_trimmed();
        Ok(format!(
            "<div data-kind=\"dashed-id\" {}>{}</div>",
            options,
            render_value(value)
        ))
    }
}

impl Render for CssColor {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let hash = self.hash_token().map_err(|_| CharismaError::ParseError)?;
        let hash = hash.text_trimmed();
        let value = self.value_token().map_err(|_| CharismaError::ParseError)?;
        let value = value.text_trimmed();
        // todo: wtf is this hash?
        Ok(format!(
            "<div data-kind=\"color\" data-hash=\"{}\" {}>{}</div>",
            hash,
            options,
            render_value(value)
        ))
    }
}

impl Render for CssString {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let str = self.value_token().map_err(|_| CharismaError::ParseError)?;
        let str = str.text();
        Ok(format!(
            "<div data-kind=\"string\" {}>{}</div>",
            options,
            render_value(str)
        ))
    }
}

impl Render for CssRatio {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}

impl Render for CssCustomIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
        todo!()
    }
}

impl Render for AnyCssValue {
    fn render_html(&self, _options: &RenderOptions) -> Result<String, CharismaError> {
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
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        match self {
            AnyCssGenericComponentValue::AnyCssValue(node) => node.render_html(options),
            // these are just commas I believe
            AnyCssGenericComponentValue::CssGenericDelimiter(_) => Ok(String::from("")),
        }
    }
}

impl Render for CssSelectorList {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let string_value = self
            .into_iter()
            .map(|s| s.unwrap().to_string())
            .reduce(|acc, cur| acc + ", " + &cur)
            .unwrap();

        Ok(format!(
            "<div data-kind=\"selector-list\" {}>
                <div data-attr=\"selectors\">{}</div>
            </div>",
            data_string_value(&string_value),
            self.into_iter()
                .map(|s| s
                    .map_err(|_| CharismaError::ParseError)
                    .and_then(|s| s.render_html(options)))
                .collect::<Result<String, _>>()?
        ))
    }
}

impl Render for Property {
    fn render_html(&self, options: &RenderOptions) -> Result<String, CharismaError> {
        let property = parse_property(&format!("{}: {};", self.name, self.value)).unwrap();
        let property = property
            .declaration()
            .map_err(|_| CharismaError::ParseError)?
            .property()
            .map_err(|_| CharismaError::ParseError)?;
        let property = property.as_css_generic_property().unwrap();

        let name = self.name.clone();
        let value = if property.value().into_iter().len() == 1 {
            let value = property.value().into_iter().next().unwrap();
            let value = value.as_any_css_value().unwrap();
            value.render_html(options)?
        } else {
            format!(
                "<div data-kind=\"multi-part-value\" data-len=\"{}\" data-string-value='{}'>
                    <div data-attr=\"args\">{}</div>
                </div>",
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
                    .collect::<Result<String, _>>()?
            )
        };
        let property_kind = if name.starts_with("--") {
            "variable"
        } else {
            "property"
        };

        Ok(format!(
            "<div data-kind=\"property\" data-property-kind=\"{}\" data-commented=\"{}\">
                <div data-attr=\"name\">{}</div>
                <div data-attr=\"value\">{}</div>
            </div>",
            property_kind,
            self.state == State::Commented,
            render_value(&name),
            value
        ))
    }
}
