use std::path::Path;

pub(crate) trait AsStr<'a> {
    fn as_str(&'a self) -> &'a str;
}

pub(crate) trait AsString {
    fn as_string(&self) -> String;
}

impl AsString for Path {
    fn as_string(&self) -> String {
        self.to_str().unwrap().to_owned()
    }
}

impl<'a> AsStr<'a> for Path {
    fn as_str(&'a self) -> &'a str {
        self.to_str().unwrap()
    }
}