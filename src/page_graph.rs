use std::collections::{btree_map, BTreeMap};

#[derive(Copy, Clone, Hash)]
pub struct PageId(u32);

pub struct Page {
    name: String,
    referenced: Vec<PageId>,
}

pub struct PageGraph {
    // name table
    name_table: BTreeMap<String, PageId>,

    /// set of all known pages
    page_array: Vec<Option<Page>>,
}

pub struct UnknownPageIter<'t> {
    map_iterator: btree_map::Iter::<'t, String, PageId>,
    pages: &'t [Option<Page>],
}

impl<'t> Iterator for UnknownPageIter<'t> {
    type Item = (&'t str, PageId);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (name, id) = self.map_iterator.next()?;

            if self.pages[id.0 as usize].is_none() {
                return Some((name.as_str(), *id));
            }
        }
    }
}

impl PageGraph {
    pub fn new() -> Self {
        Self {
            name_table: BTreeMap::new(),
            page_array: Vec::new(),
        }
    }

    /// Iterator by unknown page names getting function.
    pub fn iter_unknown_pages<'t>(&'t self) -> UnknownPageIter<'t> {
        UnknownPageIter {
            map_iterator: self.name_table.iter(),
            pages: &self.page_array,
        }
    }

    fn get_page_indices(&mut self, names: &[&str]) -> Vec<PageId> {
        names
            .iter()
            .copied()
            .map(|name| self.get_page_id_or_insert_empty(name))
            .collect::<Vec<_>>()
    }

    pub fn get_page_id_or_insert_empty(&mut self, name: &str) -> PageId {
        if let Some(key) = self.name_table.get(name) {
            return *key;
        }

        let id = PageId(self.page_array.len() as u32);
        self.name_table.insert(name.to_string(), id);
        self.page_array.push(None);
        id
    }

    /// Page insertion function.
    /// Returns `None` if page is already inserted, `Some` if not.
    /// In `Some` value, `PageId` is id by that you can get inserted page, `bool` is true if page is already known
    pub fn insert_page(&mut self, name: &str, reference_names: Vec<&str>) -> Option<(PageId, bool)> {
        let page_indices = self.get_page_indices(&reference_names);

        let new_page = Page {
            name: name.to_string(),
            referenced: page_indices,
        };

        if let Some(known_page_id) = self.name_table.get(name) {
            // page is already known

            let known_page_ref = &mut self.page_array[known_page_id.0 as usize];

            // return None if page is already processed.
            if known_page_ref.is_some() {
                return None;
            }

            self.page_array[known_page_id.0 as usize] = Some(new_page);
            Some((*known_page_id, true))
        } else {
            let page_id = PageId(self.page_array.len() as u32);
            self.name_table.insert(name.to_string(), page_id);
            self.page_array.push(Some(new_page));
            Some((page_id, false))
        }
    }

    pub fn to_json(&self) -> json::Value {
        json::json!({
            "name_table": self.name_table
                .iter()
                .map(|(name, id)| {
                    let string = name.clone();
                    let number = json::Number::from_u128(id.0 as u128).unwrap();

                    (string, json::Value::Number(number))
                })
                .collect::<json::Map::<String, json::Value>>(),
            "pages": self.page_array
                .iter()
                .map(|page_opt| {
                    if let Some(page) = page_opt {
                        json::Value::Array(page.referenced.iter()
                            .map(|v| json::Number::from_u128(v.0 as u128).unwrap())
                            .map(|n| json::Value::Number(n))
                            .collect::<Vec<_>>()
                        )
                    } else {
                        json::Value::Null
                    }
                })
                .collect::<Vec<_>>(),
        })
    }

    pub fn from_json(json: &json::Value) -> Option<Self> {
        let object = json.as_object()?;

        let json_page_array = object.get("pages")?.as_array()?;
        let json_name_table = object.get("name_table")?.as_object()?;

        let mut page_array = json_page_array
            .iter()
            .map(|_| Option::<Page>::None)
            .collect::<Vec<_>>()
            ;

        let name_table = json_name_table
            .iter()
            .map(|(name, page_id)| -> Option<(String, PageId)> {
                Some((name.to_string(), PageId(page_id.as_u64()? as u32)))
            })
            .collect::<Option<BTreeMap<_, _>>>()
            ?
            ;

        for (name, page_id) in &name_table {
            let jpa_element = &json_page_array[page_id.0 as usize];
            let pa_element = &mut page_array[page_id.0 as usize];

            match jpa_element {
                json::Value::Null => {}
                json::Value::Array(json_array) => {
                    *pa_element = Some(Page {
                        name: name.to_string(),
                        referenced: json_array
                            .iter()
                            .map(|num| num
                                .as_u64()
                                .map(|v| PageId(v as u32))
                            )
                            .collect::<Option<Vec<_>>>()
                            ?,
                    });
                }
                _ => return None
            }
            jpa_element.as_array();
        }

        Some(PageGraph { name_table, page_array })
    }
}
