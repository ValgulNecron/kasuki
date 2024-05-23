#[derive(Debug, Clone)]
pub struct CharacterAPIBuilder {
    id_filter: Option<u32>,
    is_birthday: Option<bool>,
    search: Option<String>,
    id_not: Option<u32>,
    id_in: Option<Vec<u32>>,
    id_not_in: Option<Vec<u32>>,
    sort: Option<CharacterAPISort>,
    include_media: Option<bool>,
    query: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CharacterAPISort {
    Id,
    IdDesc,
    Role,
    RoleDesc,
    SearchMatch,
    Favourites,
    FavouritesDesc,
    Relevance,
}

impl From<CharacterAPISort> for String {
    fn from(val: CharacterAPISort) -> Self {
        match val {
            CharacterAPISort::Id => "ID".to_string(),
            CharacterAPISort::IdDesc => "ID_DESC".to_string(),
            CharacterAPISort::Role => "ROLE".to_string(),
            CharacterAPISort::RoleDesc => "ROLE_DESC".to_string(),
            CharacterAPISort::SearchMatch => "SEARCH_MATCH".to_string(),
            CharacterAPISort::Favourites => "FAVOURITES".to_string(),
            CharacterAPISort::FavouritesDesc => "FAVOURITES_DESC".to_string(),
            CharacterAPISort::Relevance => "RELEVANCE".to_string(),
        }
    }
}

impl From<String> for CharacterAPISort {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ID" => Self::Id,
            "ID_DESC" => Self::IdDesc,
            "ROLE" => Self::Role,
            "ROLE_DESC" => Self::RoleDesc,
            "SEARCH_MATCH" => Self::SearchMatch,
            "FAVOURITES" => Self::Favourites,
            "FAVOURITES_DESC" => Self::FavouritesDesc,
            "RELEVANCE" => Self::Relevance,
            _ => Self::Id,
        }
    }
}

const QUERY: &str = r#"
	  id
    name {
      first
      middle
      last
      full
      native
      alternative
      alternativeSpoiler
      userPreferred
    }
    image {
      large
      medium
    }
  	description
    gender
    dateOfBirth {
      year
      month
      day
    }
    age
    bloodType
    siteUrl
    favourites
    modNotes
	}
"#;

impl CharacterAPIBuilder {
    pub fn new() -> Self {
        Self {
            id_filter: None,
            is_birthday: None,
            search: None,
            id_not: None,
            id_in: None,
            id_not_in: None,
            sort: None,
            include_media: None,
            query: None,
        }
    }

    pub fn id_filter(mut self, id: u32) -> Self {
        self.id_filter = Some(id);
        self
    }

    pub fn is_birthday(mut self, is_birthday: bool) -> Self {
        self.is_birthday = Some(is_birthday);
        self
    }

    pub fn search(mut self, search: String) -> Self {
        self.search = Some(search);
        self
    }

    pub fn id_not(mut self, id_not: u32) -> Self {
        self.id_not = Some(id_not);
        self
    }

    pub fn id_in(mut self, id_in: Vec<u32>) -> Self {
        self.id_in = Some(id_in);
        self
    }

    pub fn id_not_in(mut self, id_not_in: Vec<u32>) -> Self {
        self.id_not_in = Some(id_not_in);
        self
    }

    pub fn sort(mut self, sort: CharacterAPISort) -> Self {
        self.sort = Some(sort);
        self
    }

    pub fn include_media(mut self, include_media: bool) -> Self {
        self.include_media = Some(include_media);
        self
    }

    pub fn build(mut self, limit: Option<u32>, actual: Option<u32>) -> Self {
        let limit = limit.unwrap_or(1);
        let actual = actual.unwrap_or(0);
        let starting_query = r"query(".to_string();
        let mut characters = r"Character(".to_string();
        let mut filter = String::new();
        if let Some(id_filter) = &self.id_filter {
            filter.push_str(format!("$id: Int = {},", id_filter).as_str());
            characters.push_str("id: $id,");
        }
        if let Some(is_birthday) = self.is_birthday {
            let is_birthday = if is_birthday { "true" } else { "false" };
            filter.push_str(format!("$isBirthday: Boolean = {}", is_birthday).as_str());
            characters.push_str("isBirthday: $isBirthday,");
        }
        if let Some(search) = &self.search {
            filter.push_str(format!("$search: String = {}", search).as_str());
            characters.push_str("search: $search,");
        }
        if let Some(id_not) = &self.id_not {
            filter.push_str(format!("$idNot: Int = {}", id_not).as_str());
            characters.push_str("id_not: $idNot,");
        }
        if let Some(id_in) = &self.id_in {
            let id_in = id_in
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let id_in = format!("[{}]", id_in);
            filter.push_str(format!("$idIn: [Int] = {}", id_in).as_str());
            characters.push_str("id_in: $idIn,");
        }
        if let Some(id_not_in) = &self.id_not_in {
            let id_not_in = id_not_in
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let id_not_in = format!("[{}]", id_not_in);
            filter.push_str(format!("$idNotIn: [Int] = {}", id_not_in).as_str());
            characters.push_str("id_not_in: $idNotIn,");
        }
        if let Some(sort) = &self.sort {
            let sort: String = sort.clone().into();
            filter.push_str(format!("$sort: CharacterSort = {}", sort).as_str());
            characters.push_str("sort: $sort,");
        }

        let end_query = r"}";
        let start_query = r"{";
        let query = format!(
            "{}{}){}{}){}{}{}",
            starting_query, filter, start_query, characters, start_query, QUERY, end_query
        );
        self.query = Some(query);
        self
    }

    pub fn get_query(self) -> Option<String> {
        self.query
    }
}
