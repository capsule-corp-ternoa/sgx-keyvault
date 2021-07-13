use sharks::Share;
use std::convert::TryFrom;
use std::fmt;
use std::fs;
use std::io::Result;
use std::io::Write;

pub struct LinesStorageHandler {
    path: String,
    filename: String,
}

impl LinesStorageHandler {
    pub fn new(path: &str, filename: &str) -> Self {
        LinesStorageHandler {
            path: path.to_owned(),
            filename: filename.to_owned(),
        }
    }

    pub fn set_path(mut self, path: &str) -> Self {
        self.path = path.to_owned();
        self
    }

    pub fn set_filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_owned();
        self
    }

    pub fn filepath(&self) -> String {
        format!("{}/{}", self.path, self.filename)
    }

    /// checks if the dir exists, and if not, creates a new one
    fn ensure_dir_exists(&self) -> Result<()> {
        if fs::read_dir(&self.path).is_err() {
            fs::create_dir_all(&self.path)?
        }
        Ok(())
    }

    // write/overwrite string to file:
    pub fn write_lines_to_file<T: fmt::Display>(&self, lines: Vec<T>) -> Result<()> {
        self.ensure_dir_exists()?;

        let mut text: String = lines.iter().map(|url| format!("{}\n", url)).collect();
        text.pop(); // remove last line break
        let mut file = fs::File::create(&self.filepath())?;
        file.write_all(text.as_bytes())
    }

    // write/overwrite string to file:
    pub fn read_strings_from_file(&self) -> Result<Vec<String>> {
        Ok(fs::read_to_string(&self.filepath())?
            .split('\n')
            .map(|str| str.to_owned())
            .collect())
    }

    pub fn read_shares_from_file(&self) -> Result<Vec<Share>> {
        Ok(fs::read_to_string(&self.filepath())?
            .split('\n')
            .map(|str| Share::try_from(str.as_bytes()).unwrap())
            .collect())
    }
}

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_urlstoragehandler_works() {
        // given
        let path = "hello";
        let filename = "name.txt";

        // when
        let handler = UrlStorageHandler::new()
            .set_path(path)
            .set_filename(filename);

        // then
        assert_eq!(handler.filename, filename);
        assert_eq!(handler.path, path);
    }

    #[test]
    fn filepath_concat_works() {
        // when
        let handler = UrlStorageHandler::new()
            .set_path("hello")
            .set_filename("name.txt");

        // then
        assert_eq!(handler.filepath(), "hello/name.txt");
    }

    #[test]
    fn create_file_works() {
        // given
        let path = "hello_two";
        let filename = "hello_world.txt";
        let url = vec![];

        // when
        let url_handler = UrlStorageHandler::new()
            .set_path(path)
            .set_filename(filename);
        url_handler.write_urls_to_file(url).unwrap();

        // then
        fs::read_dir(path).unwrap();
        fs::read(&url_handler.filepath()).unwrap();

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn does_not_fail_due_to_existing_path() {
        // given
        let path = "hello_three";
        let filename = "hello_world.txt";
        let url = vec![];

        // when
        let url_handler = UrlStorageHandler::new()
            .set_path(path)
            .set_filename(filename);
        url_handler.write_urls_to_file(url.clone()).unwrap();
        let url_handler_two = UrlStorageHandler::new()
            .set_path(path)
            .set_filename(filename);
        url_handler_two.write_urls_to_file(url).unwrap();

        // then
        fs::read_dir(path).unwrap();
        fs::read(&url_handler.filepath()).unwrap();

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_empty_url_works() {
        // given
        let path = "test_empty";
        let filename = "empty_file.txt";
        let url = vec![];
        let url_handler = UrlStorageHandler::new()
            .set_path(path)
            .set_filename(filename);

        // when
        url_handler.write_urls_to_file(url).unwrap();

        // then
        assert_eq!(url_handler.read_urls_from_file().unwrap(), vec![""]);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_one_line_works() {
        // given
        let path = "test_one_line";
        let filename = "one_line_file.txt";
        let url = vec!["hello_there".to_owned()];
        let url_handler = UrlStorageHandler::new()
            .set_path(path)
            .set_filename(filename);

        // when
        url_handler.write_urls_to_file(url.clone()).unwrap();

        // then
        assert_eq!(url_handler.read_urls_from_file().unwrap(), url);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_multi_lines_works() {
        // given
        let path = "test_multi_line";
        let filename = "multi_line_file.txt";
        let url1 = "hello_there".to_owned();
        let url2 = "ohhh_hi".to_owned();
        let url3 = "who are you?".to_owned();
        let urls = vec![url1, url2, url3];
        let url_handler = UrlStorageHandler::new()
            .set_path(path)
            .set_filename(filename);

        // when
        url_handler.write_urls_to_file(urls.clone()).unwrap();

        // then
        assert_eq!(url_handler.read_urls_from_file().unwrap(), urls);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }
}
*/
