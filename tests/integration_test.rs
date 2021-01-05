#[cfg(all(
    not(feature = "feature-cookie"),
    not(feature = "cookie"),
    not(feature = "time")
))]
#[cfg(test)]
mod tests {
    use std::fs;

    use curl::easy::{Easy2, Handler};
    use netscape_cookie::parse;
    use tempfile::tempdir;

    struct EmptyHandler();
    impl Handler for EmptyHandler {}

    #[test]
    fn test_github_com() -> Result<(), String> {
        // curl -b /tmp/github.com_cookies.txt -c /tmp/github.com_cookies.txt https://github.com/ -o /dev/null

        let dir = tempdir().map_err(|err| err.to_string())?;
        let file_path = dir.path().join("github.com_cookies.txt");

        let mut easy = Easy2::new(EmptyHandler {});
        easy.cookie_jar(&file_path).map_err(|err| err.to_string())?;

        easy.get(true).unwrap();
        easy.url("https://github.com/").unwrap();
        easy.perform().unwrap();

        assert_eq!(easy.response_code().unwrap(), 200);

        drop(easy);

        let txt_content = fs::read_to_string(&file_path).map_err(|err| err.to_string())?;

        let cookies = parse(txt_content.as_bytes()).map_err(|err| err.to_string())?;

        println!("{:?}", cookies);

        assert!(cookies.len() > 0);

        Ok(())
    }
}
