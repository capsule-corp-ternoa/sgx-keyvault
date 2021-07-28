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

use sharks::Share;
use std::convert::TryFrom;
use std::fs;
use std::fs::OpenOptions;
use std::io::Result;
use std::io::Write;
use std::path::PathBuf;

pub trait VecToLinesConverter<T> {
    fn write_lines(&self, lines: Vec<T>) -> Result<()>;
    fn read_lines(&self) -> Result<Vec<T>>;
    fn write_line(&self, line: T) -> Result<()>;
    fn read(&self) -> Result<T>;
}

pub struct LocalFileStorage {
    path: PathBuf,
    filename: PathBuf,
}

impl LocalFileStorage {
    pub fn new(path: PathBuf, filename: PathBuf) -> Self {
        LocalFileStorage { path, filename }
    }
    pub fn set_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    pub fn set_filename(mut self, filename: PathBuf) -> Self {
        self.filename = filename;
        self
    }

    pub fn filepath(&self) -> PathBuf {
        self.path.join(self.filename.to_owned())
    }

    /// checks if the dir exists, and if not, creates a new one
    fn ensure_dir_exists(&self) -> Result<()> {
        if !&self.path.is_dir() {
            fs::create_dir_all(&self.path)?
        }
        Ok(())
    }
}

impl VecToLinesConverter<String> for LocalFileStorage {
    fn write_line(&self, line: String) -> Result<()> {
        self.ensure_dir_exists()?;
        let mut file = match OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.filepath())
        {
            Err(_) => fs::File::create(&self.filepath())?,
            Ok(file) => file,
        };
        writeln!(file, "{}", line)
    }

    fn write_lines(&self, lines: Vec<String>) -> Result<()> {
        self.ensure_dir_exists()?;

        let mut text: String = lines.iter().map(|url| format!("{}\n", url)).collect();
        text.pop(); // remove last line break
        let mut file = fs::File::create(&self.filepath())?;
        file.write_all(text.as_bytes())
    }

    fn read_lines(&self) -> Result<Vec<String>> {
        Ok(fs::read_to_string(&self.filepath())?
            .split('\n')
            .map(|str| str.to_owned())
            .collect())
    }

    fn read(&self) -> Result<String> {
        fs::read_to_string(&self.filepath())
    }
}

impl VecToLinesConverter<Share> for LocalFileStorage {
    fn write_line(&self, line: Share) -> Result<()> {
        self.ensure_dir_exists()?;
        // open file or create if it does not exist
        let mut file = match OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.filepath())
        {
            Err(_) => fs::File::create(&self.filepath())?,
            Ok(file) => file,
        };

        let text: String = hex::encode(Vec::from(&line));
        writeln!(file, "{}", text)
    }

    fn write_lines(&self, shares: Vec<Share>) -> Result<()> {
        self.ensure_dir_exists()?;
        let mut file = fs::File::create(&self.filepath())?;
        let mut text: String = shares
            .iter()
            .map(|share| hex::encode(Vec::from(share)) + "\n")
            .collect();
        text.pop(); // remove last line break

        file.write_all(text.as_bytes())
    }

    fn read_lines(&self) -> Result<Vec<Share>> {
        Ok(fs::read_to_string(&self.filepath())?
            .split('\n')
            .take_while(|str| str.len() > 2)
            .map(|str| Share::try_from(&*hex::decode(str).unwrap()).unwrap())
            .collect())
    }

    fn read(&self) -> Result<Share> {
        let str = fs::read_to_string(&self.filepath())?;
        Ok(Share::try_from(&*hex::decode(str).unwrap()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_file_storage_works() {
        // given
        let path = PathBuf::from("hello");
        let filename = PathBuf::from("name.txt");

        // when
        let handler = LocalFileStorage::new(path.clone(), filename.clone());

        // then
        assert_eq!(handler.filename, filename);
        assert_eq!(handler.path, path);
    }

    #[test]
    fn set_filename_and_path_works() {
        // when
        let path = "hello";
        let filename = "name.txt";
        let file_path = PathBuf::from(path).join(filename);

        let handler = LocalFileStorage::new(PathBuf::from(""), PathBuf::from(""))
            .set_path(PathBuf::from(path))
            .set_filename(PathBuf::from(filename));

        // then
        assert_eq!(handler.filepath(), file_path); // then
        assert_eq!(handler.filename, PathBuf::from(filename));
        assert_eq!(handler.path, PathBuf::from(path));
    }

    #[test]
    fn filepath_concat_works() {
        // when
        let handler = LocalFileStorage::new(PathBuf::from("hello"), PathBuf::from("name.txt"));

        // then
        assert_eq!(handler.filepath(), PathBuf::from("hello/name.txt"));
    }

    #[test]
    fn ensure_dir_exists_creates_new_if_not_existing() {
        // given
        let path = PathBuf::from("hello_create");
        let filename = PathBuf::from("hello_world.txt");
        let handler = LocalFileStorage::new(path.clone(), filename);
        assert!(!path.is_dir());

        // when
        handler.ensure_dir_exists().unwrap();

        // then
        assert!(fs::read_dir(path.as_path()).is_ok());

        //clean up
        fs::remove_dir_all(path.as_path()).unwrap();
    }

    #[test]
    fn ensure_dir_exists_does_not_fail_due_to_existing_path() {
        // given
        let path = PathBuf::from("hello_already_there");
        let filename = PathBuf::from("hello_world.txt");
        let handler = LocalFileStorage::new(path.clone(), filename.clone());
        let handler_two = LocalFileStorage::new(path.clone(), filename);
        // when
        handler.ensure_dir_exists().unwrap();
        handler_two.ensure_dir_exists().unwrap();

        // then
        assert!(fs::read_dir(path.as_path()).is_ok());

        //clean up
        fs::remove_dir_all(path.as_path()).unwrap();
    }

    #[test]
    fn read_from_file_works_with_proper_line_ending() {
        // given
        let path = "read_file";
        let filename = "empty_file.txt";
        let line1 = "lfaljaklaf a";
        let line2 = "kfjak.a-lasa";
        let line3 = "hellolee";
        let text = format! {"{}\n{}\n{}\n", line1, line2, line3};
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // create file
        fs::create_dir_all(path).unwrap();
        let mut file = fs::File::create(PathBuf::from(path).join(filename)).unwrap();
        file.write_all(text.as_bytes()).unwrap();

        // when
        let read_lines: Vec<String> = handler.read_lines().unwrap();

        // then
        assert_eq!(read_lines[0], line1);
        assert_eq!(read_lines[1], line2);
        assert_eq!(read_lines[2], line3);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn read_from_file_works_without_proper_line_ending() {
        // given
        let path = "read_file_not_proper";
        let filename = "empty_file.txt";
        let line1 = "lfaljaklaf a";
        let line2 = "kfjak.a-lasa";
        let line3 = "hellolee";
        let text = format! {"{}\n{}\n{}", line1, line2, line3};
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));
        // create file
        fs::create_dir_all(path).unwrap();
        let mut file = fs::File::create(PathBuf::from(path).join(filename)).unwrap();
        file.write_all(text.as_bytes()).unwrap();

        // when
        let read_lines: Vec<String> = handler.read_lines().unwrap();

        // then
        assert_eq!(read_lines[0], line1);
        assert_eq!(read_lines[1], line2);
        assert_eq!(read_lines[2], line3);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_empty_file_works_for_string() {
        // given
        let path = "test_empty";
        let filename = "empty_file.txt";
        let url: Vec<String> = vec![];
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_lines(url).unwrap();
        // then
        let lines: Vec<String> = handler.read_lines().unwrap();
        assert_eq!(lines, vec![""]);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_a_vector_of_size_one_works_for_string() {
        // given
        let path = "test_one_line";
        let filename = "one_line_file.txt";
        let url = vec!["hello_there".to_owned()];
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_lines(url.clone()).unwrap();

        // then
        let lines: Vec<String> = handler.read_lines().unwrap();
        assert_eq!(lines, url);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_multi_lines_works_for_string() {
        // given
        let path = "test_multi_line";
        let filename = "multi_line_file.txt";
        let url1 = "hello_there".to_owned();
        let url2 = "ohhh_hi".to_owned();
        let url3 = "who are you?".to_owned();
        let urls = vec![url1, url2, url3];
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_lines(urls.clone()).unwrap();

        // then
        let lines: Vec<String> = handler.read_lines().unwrap();
        assert_eq!(lines, urls);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }
    //Shamir Share Storage
    #[test]
    fn read_from_file_works_with_proper_line_ending_for_shamir_shares() {
        // given
        let path = "read_shamir_file";
        let filename = "empty_file.txt";

        let line1 ="016247cc9f4c161c7d8bb4ba34a66fab80b87353233a636dd1f08b13f70aa13b71168e9c265e5d41af2238065d6336a8e2";
        let line2 = "0352095a797aa3181fb035022f0c5d404e72b4c520fc42c546698d3c73c21f8aa1c0503abc3e8e4f2bb1820ece8ecd0fd2";
        let line3 ="0548ec12a8a643012b780022da1c4a8c1c4bf36f74942ba2c4c3b11b3df412920be5ec1fc07d72f836dd4916fefabd4434";
        let share1 = Share::try_from(&*hex::decode(line1).unwrap()).unwrap();
        let share2 = Share::try_from(&*hex::decode(line2).unwrap()).unwrap();
        let share3 = Share::try_from(&*hex::decode(line3).unwrap()).unwrap();

        let text = format! {"{}\n{}\n{}\n", line1, line2, line3};
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // create file
        fs::create_dir_all(path).unwrap();
        let mut file = fs::File::create(PathBuf::from(path).join(filename)).unwrap();
        file.write_all(text.as_bytes()).unwrap();

        // when
        let read_lines: Vec<Share> = handler.read_lines().unwrap();

        // then
        assert_eq!(read_lines[0].x, share1.x);
        assert_eq!(read_lines[0].y, share1.y);
        assert_eq!(read_lines[1].x, share2.x);
        assert_eq!(read_lines[1].y, share2.y);
        assert_eq!(read_lines[2].x, share3.x);
        assert_eq!(read_lines[2].y, share3.y);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn read_from_file_works_without_proper_line_ending_for_shamir_shares() {
        // given
        let path = "read_shamir_file_not_proper";
        let filename = "empty_file.txt";
        let line1 ="016247cc9f4c161c7d8bb4ba34a66fab80b87353233a636dd1f08b13f70aa13b71168e9c265e5d41af2238065d6336a8e2";
        let line2 = "0352095a797aa3181fb035022f0c5d404e72b4c520fc42c546698d3c73c21f8aa1c0503abc3e8e4f2bb1820ece8ecd0fd2";
        let line3 ="0548ec12a8a643012b780022da1c4a8c1c4bf36f74942ba2c4c3b11b3df412920be5ec1fc07d72f836dd4916fefabd4434";
        let share1 = Share::try_from(&*hex::decode(line1).unwrap()).unwrap();
        let share2 = Share::try_from(&*hex::decode(line2).unwrap()).unwrap();
        let share3 = Share::try_from(&*hex::decode(line3).unwrap()).unwrap();

        let text = format! {"{}\n{}\n{}", line1, line2, line3};
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));
        // create file
        fs::create_dir_all(path).unwrap();
        let mut file = fs::File::create(PathBuf::from(path).join(filename)).unwrap();
        file.write_all(text.as_bytes()).unwrap();

        // when
        let read_lines: Vec<Share> = handler.read_lines().unwrap();

        // then
        assert_eq!(read_lines[0].x, share1.x);
        assert_eq!(read_lines[0].y, share1.y);
        assert_eq!(read_lines[1].x, share2.x);
        assert_eq!(read_lines[1].y, share2.y);
        assert_eq!(read_lines[2].x, share3.x);
        assert_eq!(read_lines[2].y, share3.y);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_empty_file_works_for_shamir_shares() {
        // given
        let path = "test_shamir_empty";
        let filename = "empty_file.txt";
        let shares: Vec<Share> = vec![];
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_lines(shares).unwrap();

        // then
        let lines: Vec<Share> = handler.read_lines().unwrap();
        assert_eq!(lines.len(), 0);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_a_vector_of_size_one_works_for_shamir_shares() {
        // given
        let path = "test_shamir_one_line";
        let filename = "one_line_file.txt";
        let share_hex_text = "016247cc9f4c161c7d8bb4ba34a66fab80b87353233a636dd1f08b13f70aa13b71168e9c265e5d41af2238065d6336a8e2";
        let share = Share::try_from(&*hex::decode(share_hex_text).unwrap()).unwrap();
        let shares: Vec<Share> = vec![share];

        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_lines(shares.clone()).unwrap();

        // then
        let lines: Vec<Share> = handler.read_lines().unwrap();
        assert_eq!(shares.len(), 1);
        assert_eq!(lines.len(), shares.len());
        assert_eq!(lines[0].x, shares[0].x);
        assert_eq!(lines[0].y, shares[0].y);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_and_read_multi_lines_works_for_shamir_shares() {
        // given
        let path = "test_shamir_multi_line";
        let filename = "multi_line_file.txt";
        let share1 = Share::try_from(&*hex::decode("016247cc9f4c161c7d8bb4ba34a66fab80b87353233a636dd1f08b13f70aa13b71168e9c265e5d41af2238065d6336a8e2").unwrap()).unwrap();
        let share2 = Share::try_from(&*hex::decode("0352095a797aa3181fb035022f0c5d404e72b4c520fc42c546698d3c73c21f8aa1c0503abc3e8e4f2bb1820ece8ecd0fd2").unwrap()).unwrap();
        let share3 = Share::try_from(&*hex::decode("0548ec12a8a643012b780022da1c4a8c1c4bf36f74942ba2c4c3b11b3df412920be5ec1fc07d72f836dd4916fefabd4434").unwrap()).unwrap();
        let shares: Vec<Share> = vec![share1, share2, share3];

        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_lines(shares.clone()).unwrap();

        // then
        let lines: Vec<Share> = handler.read_lines().unwrap();
        assert_eq!(lines.len(), shares.len());
        assert_eq!(lines[0].x, shares[0].x);
        assert_eq!(lines[0].y, shares[0].y);
        assert_eq!(lines[1].x, shares[1].x);
        assert_eq!(lines[1].y, shares[1].y);
        assert_eq!(lines[2].x, shares[2].x);
        assert_eq!(lines[2].y, shares[2].y);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_line_and_read_works_for_shamir_shares() {
        // given
        let path = "test_shamir_writeline";
        let filename = "one_line_file.txt";
        let share_hex_text = "016247cc9f4c161c7d8bb4ba34a66fab80b87353233a636dd1f08b13f70aa13b71168e9c265e5d41af2238065d6336a8e2";
        let share = Share::try_from(&*hex::decode(share_hex_text).unwrap()).unwrap();

        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_line(share.clone()).unwrap();

        // then
        let new_share: Vec<Share> = handler.read_lines().unwrap();
        assert_eq!(Vec::from(&new_share[0]), Vec::from(&share));

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_line_and_read_works_for_string() {
        // given
        let path = "test_string_writeline";
        let filename = "one_line_file.txt";
        let str = "hello_there".to_owned();
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_line(str.clone()).unwrap();

        // then
        let line: Vec<String> = handler.read_lines().unwrap();
        assert_eq!(line[0], str);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_line_string_does_not_overwrite_existing() {
        // given
        let path = "write_does_not_overwrite";
        let filename = "not_overwritten.txt";
        let str = "hello_there".to_owned();
        let str_two = "hello_second".to_owned();
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_line(str.clone()).unwrap();
        handler.write_line(str_two.clone()).unwrap();

        // then
        let lines: Vec<String> = handler.read_lines().unwrap();
        assert_eq!(lines[0], str);
        assert_eq!(lines[1], str_two);

        //clean up
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn write_line_shard_does_not_overwrite_existing() {
        // given
        let path = "write_shard_does_not_overwrite";
        let filename = "not_overwritten.txt";
        let share_hex_text = "016247cc9f4c161c7d8bb4ba34a66fab80b87353233a636dd1f08b13f70aa13b71168e9c265e5d41af2238065d6336a8e2";
        let share = Share::try_from(&*hex::decode(share_hex_text).unwrap()).unwrap();
        let share_hex_text_two = "016247cc9f4c161c7d8bb4ba34a66fab80b87353233a636dd1f08b13f70bb13b71168e9c265e5d41af2238065d6336a8e2";
        let share_two = Share::try_from(&*hex::decode(share_hex_text_two).unwrap()).unwrap();
        let handler = LocalFileStorage::new(PathBuf::from(path), PathBuf::from(filename));

        // when
        handler.write_line(share.clone()).unwrap();
        handler.write_line(share_two.clone()).unwrap();

        // then
        let lines: Vec<Share> = handler.read_lines().unwrap();
        assert_eq!(Vec::from(&lines[0]), Vec::from(&share));
        assert_eq!(Vec::from(&lines[1]), Vec::from(&share_two));

        //clean up
        fs::remove_dir_all(path).unwrap();
    }
}
