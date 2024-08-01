use biome_css_syntax::{
    AnyCssDimension, AnyCssExpression, AnyCssFunction, AnyCssGenericComponentValue,
    AnyCssPseudoClass, AnyCssPseudoClassNth, AnyCssPseudoClassNthSelector, AnyCssRelativeSelector,
    AnyCssSelector, AnyCssSimpleSelector, AnyCssSubSelector, AnyCssValue, CssAttributeSelector,
    CssBinaryExpression, CssBogusPseudoClass, CssClassSelector, CssColor, CssComplexSelector,
    CssComponentValueList, CssCompoundSelector, CssCustomIdentifier, CssDashedIdentifier,
    CssFunction, CssIdSelector, CssIdentifier, CssNthOffset, CssNumber, CssParameter,
    CssParenthesizedExpression, CssPercentage, CssPseudoClassFunctionCompoundSelector,
    CssPseudoClassFunctionCompoundSelectorList, CssPseudoClassFunctionIdentifier,
    CssPseudoClassFunctionNth, CssPseudoClassFunctionRelativeSelectorList,
    CssPseudoClassFunctionSelector, CssPseudoClassFunctionSelectorList,
    CssPseudoClassFunctionValueList, CssPseudoClassIdentifier, CssPseudoClassNth,
    CssPseudoClassNthIdentifier, CssPseudoClassNthNumber, CssPseudoClassNthSelector,
    CssPseudoClassSelector, CssPseudoElementSelector, CssRatio, CssRegularDimension,
    CssRelativeSelector, CssSelectorList, CssString, CssSyntaxKind, CssSyntaxNode, CssTypeSelector,
    CssUniversalSelector, CssUrlFunction,
};
use serde::Serialize;
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

pub fn render_error_node(value: &str) -> String {
    format!(
        "<div data-kind=\"error-node\" {}>{}</div>",
        data_string_value(value),
        value
    )
}

pub fn data_string_value(value: &str) -> String {
    attr("data-string-value", value)
}

// TODO: remove this
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

#[derive(Serialize, Debug)]
pub struct RenderResult {
    pub html: String,
    pub errors: Vec<CharismaError>,
}

pub trait Render {
    fn render_html(&self, options: &RenderOptions) -> RenderResult;
}

impl Render for AnyCssSelector {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
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
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let left = match self.left() {
            Ok(l) => {
                let r = l.render_html(options);
                errors.extend(r.errors);
                r.html
            }
            Err(err) => {
                errors.push(CharismaError::ParseError(err.to_string()));
                String::from("")
            }
        };
        let right = match self.right() {
            Ok(r) => {
                let r = r.render_html(options);
                errors.extend(r.errors);
                r.html
            }
            Err(err) => {
                errors.push(CharismaError::ParseError(err.to_string()));
                String::from("")
            }
        };
        let combinator = match self.combinator() {
            Ok(r) => render_combinator_type(r.kind()),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                RenderResult {
                    html: "".into(),
                    errors: vec![CharismaError::ParseError(e.to_string())],
                }
            }
        };

        RenderResult {
            html: format!(
                "
            <div data-kind=\"complex-selector\" data-combinator-type=\"{}\" {}>
                <div data-attr=\"left\">{}</div>
                <div data-attr=\"right\">{}</div>
            </div>",
                combinator.html,
                data_string_value(&self.to_string()),
                left,
                right,
            ),
            errors: [errors, combinator.errors].concat(),
        }
    }
}

impl Render for CssClassSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];

        let name = match self.name() {
            Ok(n) => n.to_string(),
            Err(er) => {
                errors.push(CharismaError::ParseError(er.to_string()));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"class\" {}>{}</div>",
                options,
                render_value(name.trim())
            ),
            errors,
        }
    }
}

impl Render for CssPseudoClassIdentifier {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let name = match self.name() {
            Ok(name) => name.to_string(),
            Err(err) => {
                errors.push(CharismaError::ParseError(err.to_string()));
                String::from("")
            }
        };
        RenderResult {
            html: format!(
                "<div data-kind=\"pseudo-class-id\" {}>{}</div>",
                options,
                render_value(name.trim())
            ),
            errors,
        }
    }
}

fn render_combinator_type(kind: CssSyntaxKind) -> RenderResult {
    match get_combinator_type(kind) {
        Ok(Combinator::Descendant) => RenderResult {
            html: "descendant".to_string(),
            errors: vec![],
        },
        Ok(Combinator::DirectDescendant) => RenderResult {
            html: "direct-descendant".to_string(),
            errors: vec![],
        },
        Ok(Combinator::Plus) => RenderResult {
            html: "next-sibling".to_string(),
            errors: vec![],
        },
        Ok(Combinator::And) => RenderResult {
            html: "".to_string(),
            errors: vec![CharismaError::AssertionError("unexpected `And`".into())],
        },
        Err(e) => RenderResult {
            html: "".to_owned(),
            errors: vec![e],
        },
    }
}

impl Render for CssRelativeSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        match self.combinator() {
            Some(combinator) => {
                let mut errors: Vec<CharismaError> = vec![];
                let selector = match self.selector() {
                    Ok(s) => {
                        let result = s.render_html(options);
                        errors.extend(result.errors);
                        result.html
                    }
                    Err(err) => {
                        errors.push(CharismaError::ParseError(err.to_string()));
                        String::from("")
                    }
                };

                let combinator_type = render_combinator_type(combinator.kind());

                RenderResult {
                    html: format!(
                        "<div data-kind=\"relative-selector\" data-combinator-type=\"{}\" {}>{}</div>",
                        combinator_type.html,
                        data_string_value(&self.to_string()),
                        selector
                    ),
                    errors: [errors, combinator_type.errors].concat(),
                }
            }
            None => match self.selector() {
                Ok(s) => s.render_html(options),

                Err(err) => RenderResult {
                    errors: vec![CharismaError::ParseError(err.to_string())],
                    html: String::from(""),
                },
            },
        }
    }
}

impl Render for AnyCssRelativeSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        match self {
            AnyCssRelativeSelector::CssBogusSelector(s) => RenderResult {
                html: String::from(""),
                errors: vec![CharismaError::ParseError(format!("{:?}", s))],
            },
            AnyCssRelativeSelector::CssRelativeSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassFunctionRelativeSelectorList {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let name = match self.name_token() {
            Ok(n) => format!("{}", n),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };
        assert!(self.relative_selectors().into_iter().count() == 1);

        let selector = match self.relative_selectors().into_iter().next() {
            Some(Ok(s)) => {
                let r = s.render_html(options);
                errors.extend(r.errors);
                r.html
            }
            Some(Err(e)) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
            None => {
                errors.push(CharismaError::ParseError(String::from("expected selector")));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"pseudo-class-function\" {}>
                <div data-attr=\"function-name\">{}</div>
                <div data-attr=\"args\">{}</div>
            </div>",
                data_string_value(&self.to_string()),
                render_value(name.trim()),
                selector
            ),
            errors,
        }
    }
}
impl Render for CssNthOffset {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        let sign = self.sign().unwrap().to_string();
        let value = self.value().unwrap().to_string();

        RenderResult {
            html: format!(
                "
            <div data-kind=\"nth-offset\">
                <div data-attr=\"sign\">{}</div>
                <div data-attr=\"value\">{}</div>
            </div>
            ",
                render_value(sign.trim()),
                render_value(value.trim())
            ),
            errors: vec![],
        }
    }
}

impl Render for CssPseudoClassNth {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let offset = match self.offset() {
            Some(o) => {
                let r = o.render_html(options);
                errors.extend(r.errors);
                r.html
            }
            None => String::from(""),
        };

        RenderResult {
            html: format!(
                "
            <div data-kind=\"pseudo-class-nth\" {}>
                <div data-attr=\"value\">{}</div>
                <div data-attr=\"offset\">{}</div>
            </div>
        ",
                data_string_value(&self.to_string()),
                render_value(self.value().unwrap().to_string().trim()),
                offset,
            ),
            errors,
        }
    }
}

impl Render for CssPseudoClassNthIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        todo!()
    }
}

impl Render for Frame {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let selector = match self.path.last() {
            Some(Part::AtRule(AtRulePart::Percentage(pct))) => {
                format!(
                    "<div data-kind=\"keyframes-percentage-selector\">
                        <div data-attr=\"percentage\">{}</div>
                    </div>",
                    render_value(&pct.to_string())
                )
            }
            _ => {
                errors.push(CharismaError::ParseError(String::from(
                    "keyframes selector broken",
                )));
                String::from("")
            }
        };

        let properties = self
            .properties
            .iter()
            .map(|p| p.render_html(options))
            .reduce(|acc, RenderResult { html, errors }| RenderResult {
                errors: [acc.errors, errors].concat(),
                html: acc.html + &html,
            })
            .unwrap_or(RenderResult {
                html: String::from(""),
                errors: vec![],
            });

        RenderResult {
            html: format!(
                "<div data-kind=\"frame\">
                <div data-attr=\"selector\">{}</div>
                <div data-attr=\"properties\">{}</div>
            </div>",
                selector, properties.html
            ),
            errors: [errors, properties.errors].concat(),
        }
    }
}

impl Render for CssPseudoClassNthNumber {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        assert!(self.sign().is_none());
        let number = match self.value() {
            Ok(v) => v.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };
        RenderResult {
            html: format!(
                "
            <div data-kind=\"pseudo-class-nth-number\" {}>
                {}
            </div>
            ",
                data_string_value(&self.to_string()),
                render_value(number.trim())
            ),
            errors,
        }
    }
}

impl Render for AnyCssPseudoClassNth {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        match self {
            AnyCssPseudoClassNth::CssPseudoClassNth(s) => s.render_html(options),
            AnyCssPseudoClassNth::CssPseudoClassNthIdentifier(s) => s.render_html(options),
            AnyCssPseudoClassNth::CssPseudoClassNthNumber(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassNthSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        assert!(self.of_selector().is_none());
        match self.nth() {
            Ok(nth) => nth.render_html(options),
            Err(er) => RenderResult {
                html: String::from(""),
                errors: vec![CharismaError::ParseError(er.to_string())],
            },
        }
    }
}

impl Render for AnyCssPseudoClassNthSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        match self {
            AnyCssPseudoClassNthSelector::CssBogusSelector(_) => panic!(),
            AnyCssPseudoClassNthSelector::CssPseudoClassNthSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssPseudoClassFunctionNth {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let name = match self.name() {
            Ok(name) => format!("{}", name),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };
        let selector = match self.selector() {
            Ok(s) => {
                let r = s.render_html(options);
                errors.extend(r.errors);
                r.html
            }
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };
        RenderResult {
            html: format!(
                "<div data-kind=\"pseudo-class-function-nth\" {}>
                    <div data-attr=\"name\">{}</div>
                    <div data-attr=\"selector\">{}</div>
                </div>",
                data_string_value(&self.to_string()),
                render_value(name.trim()),
                selector,
            ),
            errors,
        }
    }
}

impl Render for CssPseudoClassFunctionCompoundSelector {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionCompoundSelectorList {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionSelector {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        todo!()
    }
}
impl Render for CssPseudoClassFunctionSelectorList {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let name = match self.name() {
            Ok(n) => format!("{}", n),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        let selectors = self
            .selectors()
            .into_iter()
            .map(|s| match s {
                Ok(s) => s.render_html(&RenderOptions::default()),
                Err(er) => RenderResult {
                    html: "".to_owned(),
                    errors: vec![CharismaError::ParseError(er.to_string())],
                },
            })
            .reduce(|acc, RenderResult { html, errors }| RenderResult {
                html: acc.html + &html,
                errors: [acc.errors, errors].concat(),
            })
            .unwrap_or(RenderResult {
                html: String::from(""),
                errors: vec![],
            });

        RenderResult {
            html: format!(
                "
        <div data-kind=\"pseudo-class-function-selector\" {}>
            <div data-attr=\"name\">{}</div>
            <div data-attr=\"selectors\">{}</div>
        </div>
            ",
                data_string_value(&self.to_string()),
                render_value(name.trim()),
                selectors.html
            ),
            errors: [errors, selectors.errors].concat(),
        }
    }
}
impl Render for CssPseudoClassFunctionValueList {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        todo!()
    }
}

impl Render for CssPseudoClassSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let class = match self.class() {
            Ok(c) => c,
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                unsafe {
                    AnyCssPseudoClass::CssBogusPseudoClass(CssBogusPseudoClass::new_unchecked(
                        CssSyntaxNode::new_detached(CssSyntaxKind::CSS_BOGUS, []),
                    ))
                }
            }
        };

        let result = match class {
            AnyCssPseudoClass::CssBogusPseudoClass(s) => RenderResult {
                html: String::from(""),
                errors: vec![CharismaError::ParseError(format!(
                    "bogus class selector = {:?}",
                    s
                ))],
            },
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
        };

        RenderResult {
            html: result.html,
            errors: [errors, result.errors].concat(),
        }
    }
}

impl Render for CssAttributeSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        match self.matcher() {
            Some(matcher) => {
                let name = match self.name() {
                    Ok(name) => name.to_string(),
                    Err(e) => {
                        errors.push(CharismaError::ParseError(e.to_string()));
                        String::from("")
                    }
                };
                assert!(matcher.modifier().is_none());
                let operator = match matcher.operator() {
                    Ok(op) => op.text_trimmed().to_string(),
                    Err(e) => {
                        errors.push(CharismaError::ParseError(e.to_string()));
                        String::from("")
                    }
                };
                let value = match matcher.value() {
                    Ok(v) => match v.name() {
                        Ok(n) => n.to_string(),
                        Err(e) => {
                            errors.push(CharismaError::ParseError(e.to_string()));
                            String::from("")
                        }
                    },
                    Err(e) => {
                        errors.push(CharismaError::ParseError(e.to_string()));
                        String::from("")
                    }
                };

                RenderResult {
                    html: format!(
                        "
                <div data-kind=\"attribute-selector\" {}>
                    <div data-attr=\"name\">{}</div>
                    <div data-attr=\"operator\">{}</div>
                    <div data-attr=\"value\">{}</div>
                </div>",
                        options,
                        render_value(name.trim()),
                        render_value(operator.trim()),
                        render_value(value.trim())
                    ),
                    errors,
                }
            }
            None => {
                let name = match self.name() {
                    Ok(n) => n.to_string(),
                    Err(e) => {
                        errors.push(CharismaError::ParseError(e.to_string()));
                        String::from("")
                    }
                };
                RenderResult {
                    html: format!(
                        "
                <div data-kind=\"attribute-selector\" {}>
                    <div data-attr=\"name\">{}</div>
                </div>",
                        options,
                        render_value(name.trim())
                    ),
                    errors,
                }
            }
        }
    }
}

impl Render for CssPseudoElementSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        // TODO:
        let element = self.element().unwrap();
        let element = element.as_css_pseudo_element_identifier().unwrap();

        let name = match element.name() {
            Ok(n) => n.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"pseudo-element-selector\" {}>{}</div>",
                options,
                render_value(name.trim())
            ),
            errors,
        }
    }
}

impl Render for CssIdSelector {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let name = match self.name() {
            Ok(n) => n.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"id-selector\" {}>
                <div data-attr=\"name\">{}</div>
            </div>",
                data_string_value(&self.to_string()),
                render_value(name.trim())
            ),
            errors,
        }
    }
}

impl Render for AnyCssSubSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
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
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        assert!(self.namespace().is_none());
        match self.ident() {
            Ok(n) => n.render_html(options),
            Err(e) => RenderResult {
                html: String::from(""),
                errors: vec![CharismaError::ParseError(e.to_string())],
            },
        }
    }
}

impl Render for CssUniversalSelector {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        RenderResult {
            html: "<div data-kind=\"universal-selector\"></div>".to_string(),
            errors: vec![],
        }
    }
}

impl Render for AnyCssSimpleSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        match self {
            AnyCssSimpleSelector::CssTypeSelector(node) => node.render_html(options),
            AnyCssSimpleSelector::CssUniversalSelector(s) => s.render_html(options),
        }
    }
}

impl Render for CssCompoundSelector {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        assert!(self.nesting_selector_token().is_none());
        // simple selector is either an element/type selector eg. `body`, or `*` the universal selector
        // I don't understand at the moment why it is separate from compound selector thing, but we're
        // just going to prepend it onto the list of sub_selectors
        let simple_selector_html = self
            .simple_selector()
            .map(|s| s.render_html(options))
            .unwrap_or(RenderResult {
                html: String::from(""),
                errors: vec![],
            });

        let sub_selectors = self.sub_selectors().into_iter().collect::<Vec<_>>();

        if sub_selectors.len() == 1 && self.simple_selector().is_none() {
            sub_selectors[0].render_html(options)
        } else if sub_selectors.is_empty() && self.simple_selector().is_some() {
            simple_selector_html
        } else {
            let selectors = sub_selectors
                .iter()
                .map(|selector| selector.render_html(options))
                .reduce(|acc, RenderResult { html, errors }| RenderResult {
                    errors: [acc.errors, errors].concat(),
                    html: acc.html + &html,
                })
                .unwrap_or(RenderResult {
                    html: String::from(""),
                    errors: vec![],
                });

            RenderResult {
                html: format!(
                    "<div data-kind=\"compound-selector\" {}>{}{}</div>",
                    options, simple_selector_html.html, selectors.html
                ),
                errors: [simple_selector_html.errors, selectors.errors].concat(),
            }
        }
    }
}

impl Render for CssRegularDimension {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let unit_type = match self.unit_token() {
            Ok(u) => u.to_string(),
            Err(err) => {
                errors.push(CharismaError::ParseError(err.to_string()));
                String::from("")
            }
        };
        let value = match self.value_token() {
            Ok(u) => u.to_string(),
            Err(err) => {
                errors.push(CharismaError::ParseError(err.to_string()));
                String::from("")
            }
        };
        RenderResult {
            html: format!(
                "<div data-kind=\"unit\" data-unit-type=\"{}\" {}>{}</div>",
                unit_type.trim(),
                options,
                render_value(value.trim())
            ),
            errors,
        }
    }
}

impl Render for CssPercentage {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];

        let value = match self.value_token() {
            Ok(u) => u.to_string(),
            Err(err) => {
                errors.push(CharismaError::ParseError(err.to_string()));
                String::from("")
            }
        };
        RenderResult {
            html: format!(
                "<div data-kind=\"unit\" data-unit-type=\"percentage\" {}>{}</div>",
                options,
                render_value(value.trim())
            ),
            errors,
        }
    }
}

impl Render for AnyCssDimension {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        match self {
            AnyCssDimension::CssPercentage(node) => node.render_html(options),
            AnyCssDimension::CssRegularDimension(node) => node.render_html(options),
            AnyCssDimension::CssUnknownDimension(d) => RenderResult {
                html: render_error_node(&d.to_string()),
                errors: vec![CharismaError::ParseError(
                    "unknown css dimension".to_string(),
                )],
            },
        }
    }
}

impl Render for CssIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];

        let value = match self.value_token() {
            Ok(u) => u.to_string(),
            Err(err) => {
                errors.push(CharismaError::ParseError(err.to_string()));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"identifier\" {}>{}</div>",
                data_string_value(value.trim()),
                render_value(value.trim())
            ),
            errors,
        }
    }
}

impl Render for CssComponentValueList {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        self.into_iter()
            .map(|item| item.render_html(options))
            .reduce(|acc, RenderResult { html, errors }| RenderResult {
                errors: [acc.errors, errors].concat(),
                html: acc.html + &html,
            })
            .unwrap_or(RenderResult {
                html: String::from(""),
                errors: vec![],
            })
    }
}

impl Render for CssBinaryExpression {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];

        let left = match self.left() {
            Ok(l) => {
                let r = l.render_html(options);
                errors.extend(r.errors);
                r.html
            }
            Err(er) => {
                errors.push(CharismaError::ParseError(er.to_string()));
                String::from("")
            }
        };
        let right = match self.right() {
            Ok(l) => {
                let r = l.render_html(options);
                errors.extend(r.errors);
                r.html
            }
            Err(er) => {
                errors.push(CharismaError::ParseError(er.to_string()));
                String::from("")
            }
        };
        let operator = match self.operator_token() {
            Ok(r) => r.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"binary-expression\">
                <div data-attr=\"left\">{}</div>
                <div data-attr=\"operator\">{}</div>
                <div data-attr=\"right\">{}</div>
            </div>",
                left,
                render_value(&operator),
                right,
            ),
            errors,
        }
    }
}

impl Render for CssParenthesizedExpression {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        todo!()
    }
}

impl Render for AnyCssExpression {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
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
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        self.any_css_expression().unwrap().render_html(options)
    }
}

impl Render for CssFunction {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let function_name = match self.name() {
            Ok(n) => {
                let r = n.render_html(options);
                errors.extend(r.errors);
                r.html
            }
            Err(err) => {
                errors.push(CharismaError::ParseError(err.to_string()));
                String::from("")
            }
        };

        let args = self
            .items()
            .into_iter()
            .map(|item| match item {
                Ok(r) => r.render_html(options),
                Err(e) => RenderResult {
                    html: String::from(""),
                    errors: vec![CharismaError::ParseError(e.to_string())],
                },
            })
            .reduce(|acc, RenderResult { html, errors }| RenderResult {
                html: acc.html + &html,
                errors: [acc.errors, errors].concat(),
            })
            .unwrap_or(RenderResult {
                html: String::from(""),
                errors: vec![],
            });

        RenderResult {
            html: format!(
                "
        <div data-kind=\"function\" {}>
            <div data-attr=\"name\">{}</div>
            <div data-attr=\"args\">{}</div>
        </div>
        ",
                options, function_name, args.html
            ),
            errors: [errors, args.errors].concat(),
        }
    }
}

impl Render for CssUrlFunction {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        assert!(self.modifiers().into_iter().len() == 0);
        let name = match self.name() {
            Ok(name) => name.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        let url = self.value().unwrap().to_string();
        RenderResult {
            html: format!(
                "<div data-kind=\"url-function\" {}>
                <div data-attr=\"name\">{}</div>
                <div data-attr=\"value\">{}</div>
            </div>",
                data_string_value(&self.to_string()),
                render_value(name.trim()),
                render_value(&url),
            ),
            errors,
        }
    }
}

impl Render for AnyCssFunction {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        match self {
            AnyCssFunction::CssFunction(f) => f.render_html(options),
            AnyCssFunction::CssUrlFunction(s) => s.render_html(options),
        }
    }
}

impl Render for CssNumber {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let value = match self.value_token() {
            Ok(v) => v.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"number\" {}>{}</div>",
                options,
                render_value(value.trim())
            ),
            errors,
        }
    }
}

impl Render for CssDashedIdentifier {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];
        let value = match self.value_token() {
            Ok(v) => v.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"dashed-id\" {}>{}</div>",
                options,
                render_value(value.trim())
            ),
            errors,
        }
    }
}

impl Render for CssColor {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];

        let hash = match self.hash_token() {
            Ok(h) => h.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        let value = match self.value_token() {
            Ok(h) => h.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        // todo: wtf is this hash?
        RenderResult {
            html: format!(
                "<div data-kind=\"color\" data-hash=\"{}\" {}>{}</div>",
                hash.trim(),
                options,
                render_value(value.trim())
            ),
            errors,
        }
    }
}

impl Render for CssString {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let mut errors: Vec<CharismaError> = vec![];

        let value = match self.value_token() {
            Ok(h) => h.to_string(),
            Err(e) => {
                errors.push(CharismaError::ParseError(e.to_string()));
                String::from("")
            }
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"string\" {}>{}</div>",
                options,
                render_value(&value)
            ),
            errors,
        }
    }
}

impl Render for CssRatio {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let numerator = match self.numerator() {
            Ok(n) => n.render_html(options),
            Err(err) => RenderResult {
                html: "".to_string(),
                errors: vec![CharismaError::ParseError(err.to_string())],
            },
        };
        let denominator = match self.denominator() {
            Ok(d) => d.render_html(options),
            Err(err) => RenderResult {
                html: "".to_string(),
                errors: vec![CharismaError::ParseError(err.to_string())],
            },
        };

        RenderResult {
            html: format!(
                "<div data-kind=\"ratio\">
                    <div data-attr=\"numerator\">{}</div>
                    <div data-attr=\"denominator\">{}</div>
                </div>",
                numerator.html, denominator.html
            ),
            errors: [numerator.errors, denominator.errors].concat(),
        }
    }
}

impl Render for CssCustomIdentifier {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
        todo!()
    }
}

impl Render for AnyCssValue {
    fn render_html(&self, _options: &RenderOptions) -> RenderResult {
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
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        match self {
            AnyCssGenericComponentValue::AnyCssValue(node) => node.render_html(options),
            // these are just commas I believe
            AnyCssGenericComponentValue::CssGenericDelimiter(_) => RenderResult {
                html: String::from(""),
                errors: vec![],
            },
        }
    }
}

impl Render for CssSelectorList {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let string_value = self
            .into_iter()
            .map(|s| s.unwrap().to_string())
            .reduce(|acc, cur| acc + ", " + &cur)
            .unwrap();

        let result = self
            .into_iter()
            .map(|s| match s {
                Ok(s) => s.render_html(options),
                Err(e) => RenderResult {
                    html: "".to_string(),
                    errors: vec![CharismaError::ParseError(e.to_string())],
                },
            })
            .reduce(|acc, RenderResult { html, errors }| RenderResult {
                html: acc.html + &html,
                errors: [acc.errors, errors].concat(),
            })
            .unwrap_or(RenderResult {
                html: String::from(""),
                errors: vec![],
            });

        RenderResult {
            html: format!(
                "<div data-kind=\"selector-list\" {}>
                <div data-attr=\"selectors\">{}</div>
            </div>",
                data_string_value(&string_value),
                result.html
            ),
            errors: result.errors,
        }
    }
}

impl Render for Property {
    fn render_html(&self, options: &RenderOptions) -> RenderResult {
        let property = parse_property(&format!("{}: {};", self.name, self.value)).unwrap();
        match property.declaration().and_then(|d| d.property()) {
            Ok(p) => {
                let property = p.as_css_generic_property().unwrap();

                let name = self.name.clone();
                let value = if property.value().into_iter().len() == 1 {
                    let value = property.value().into_iter().next().unwrap();
                    let value = value.as_any_css_value().unwrap();
                    value.render_html(options)
                } else {
                    let values = property
                        .value()
                        .into_iter()
                        .map(|value| value.render_html(options))
                        .reduce(|acc, RenderResult { html, errors }| RenderResult {
                            html: acc.html + &html,
                            errors: [acc.errors, errors].concat(),
                        })
                        .unwrap_or(RenderResult {
                            html: String::from(""),
                            errors: vec![],
                        });
                    RenderResult {
                        html: format!(
                        "<div data-kind=\"multi-part-value\" data-len=\"{}\" data-string-value='{}'>
                            <div data-attr=\"args\">{}</div>
                        </div>",
                        property.value().into_iter().count(),
                        property
                            .value()
                            .into_iter()
                            .map(|value| value.to_string())
                            .collect::<String>(),
                        values.html
                    ),
                        errors: values.errors,
                    }
                };
                let property_kind = if name.starts_with("--") {
                    "variable"
                } else {
                    "property"
                };

                RenderResult {
                    html: format!(
                    "<div data-kind=\"property\" data-property-kind=\"{}\" data-commented=\"{}\">
                        <div data-attr=\"name\">{}</div>
                        <div data-attr=\"value\">{}</div>
                    </div>",
                    property_kind,
                    self.state == State::Commented,
                    render_value(&name),
                    value.html
                ),
                    errors: value.errors,
                }
            }
            Err(e) => RenderResult {
                html: render_error_node(&property.to_string()),
                errors: vec![CharismaError::ParseError(e.to_string())],
            },
        }
    }
}
