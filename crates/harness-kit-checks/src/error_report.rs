pub fn print_error_chain(error: &anyhow::Error) {
    eprintln!("{}", format_error_chain(error));
}

fn format_error_chain(error: &anyhow::Error) -> String {
    error
        .chain()
        .enumerate()
        .map(|(index, cause)| {
            if index == 0 {
                cause.to_string()
            } else {
                format!("caused by: {cause}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, anyhow};

    use super::*;

    #[test]
    fn formats_error_chain_with_causes() {
        let error = Err::<(), _>(anyhow!("inner detail"))
            .context("outer gate failed")
            .unwrap_err();

        assert_eq!(
            format_error_chain(&error),
            "outer gate failed\ncaused by: inner detail"
        );
    }
}
