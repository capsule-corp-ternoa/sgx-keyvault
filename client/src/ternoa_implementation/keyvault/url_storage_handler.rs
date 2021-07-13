/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/
use std::fs;
use std::io::Result;
use std::io::Write;

const KEYVAULT_DEFAULT_PATH: &str = "my_keyvaults";
const KEYVAULT_DEFAULT_URLLIST_FILENAME: &str = "keyvault_pool.txt";

pub struct UrlStorageHandler {
    path: String,
    filename: String,
}

impl UrlStorageHandler {
    pub fn new() -> Self {
        UrlStorageHandler {
            path: KEYVAULT_DEFAULT_PATH.to_string(),
            filename: KEYVAULT_DEFAULT_URLLIST_FILENAME.to_string(),
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
    pub fn write_urls_to_file(&self, urls: Vec<String>) -> Result<()> {
        self.ensure_dir_exists()?;
        let mut text: String = urls.iter().map(|url| format!("{}\n", url)).collect();
        text.pop(); // remove last line break
        let mut file = fs::File::create(&self.filepath())?;
        file.write_all(text.as_bytes())
    }

    // write/overwrite string to file:
    pub fn read_urls_from_file(&self) -> Result<Vec<String>> {
        Ok(fs::read_to_string(&self.filepath())?
            .split('\n')
            .map(|str| str.to_owned())
            .collect())
    }
}

impl Default for UrlStorageHandler {
    fn default() -> Self {
        UrlStorageHandler::new()
    }
}

#[cfg(test)]
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
