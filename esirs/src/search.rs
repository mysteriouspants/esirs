use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SearchResult {
  pub character: Vec<u64>
}

#[cfg(test)]
mod tests {
  use crate::search::SearchResult;
  use std::fs::File;

  #[test]
  fn single_character_search_result_de() {
    let file = File::open("../tst-data/search/single-character.json").unwrap();
    let search_result: SearchResult = serde_json::from_reader(file).unwrap();
    assert!(search_result.character.len() == 1);
    assert!(*search_result.character.first().unwrap() == 404345999);
  }
}