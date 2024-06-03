use biome_css_syntax::{AnyCssSelector::*, AnyCssSubSelector::*};
use sha1::Digest;

fn hash(string: String) -> String {
    let mut hasher = sha1::Sha1::new();
    hasher.update(string.as_bytes());
    let h = hasher.finalize();
    format!("{:X}", h)
}

pub trait Storage {
    fn to_path(&self) -> Vec<String>;
}

impl Storage for biome_css_syntax::AnyCssSelector {
    fn to_path(&self) -> Vec<String> {
        match self {
            CssBogusSelector(_) => todo!(),
            CssComplexSelector(_) => todo!(),
            CssCompoundSelector(selector) => selector.to_path(),
        }
    }
}

impl Storage for biome_css_syntax::CssCompoundSelector {
    fn to_path(&self) -> Vec<String> {
        self.sub_selectors()
            .into_iter()
            .flat_map(|selector| selector.to_path())
            .collect()
    }
}

impl Storage for biome_css_syntax::AnyCssSubSelector {
    fn to_path(&self) -> Vec<String> {
        match self {
            CssAttributeSelector(_) => todo!(),
            CssBogusSubSelector(_) => todo!(),
            CssClassSelector(class) => {
                let n = class.name().unwrap().to_string();
                // TODO: remove trim
                let name = n.trim();
                assert!(name == "btn".to_string());
                return vec![hash(".".to_string() + &name)];
            }
            CssIdSelector(_) => todo!(),
            CssPseudoClassSelector(_) => todo!(),
            CssPseudoElementSelector(_) => todo!(),
        }
    }
}
