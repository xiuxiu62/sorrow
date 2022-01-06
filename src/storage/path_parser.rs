pub struct PathParser;

impl<'a> PathParser {
    pub fn valid_path_format(path: &'a str) -> bool {
        unimplemented!();
    }

    pub fn parse(path: &'a str) {
        unimplemented!();
    }

    pub fn root(path: &'a str) -> Option<&'a str> {
        path.split('/').nth(0)
    }
}
