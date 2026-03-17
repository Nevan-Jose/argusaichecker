use crate::schemas::layer1::{Layer1Anchor, Layer1Token};
use uuid::Uuid;

pub fn normalize_tokens(tokens: Vec<Layer1Token>) -> Vec<Layer1Anchor> {
    tokens.into_iter().map(normalize_one).collect()
}

fn normalize_one(t: Layer1Token) -> Layer1Anchor {
    Layer1Anchor {
        id: Uuid::new_v4().to_string(),
        file: normalize_path(&t.file),
        language: t.language.to_lowercase(),
        line: t.line,
        column: t.column,
        token: t.token,
        token_kind: t.token_kind,
        normalized_kind: t.normalized_kind.to_lowercase(),
        ast_metadata: t.ast_metadata,
        layer1_rule_id: t.layer1_rule_id,
        layer1_confidence: t.layer1_confidence,
    }
}

fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}
