

use std::collections::HashMap;

use base64::{prelude::BASE64_STANDARD, Engine};

enum PageState {
    File,
    Missing,
}

pub struct PageStorage {
    page_states: HashMap<String, PageState>,
}

fn load_page_net(name: &str) -> Option<String> {
    let path = format!("https://neolurk.org/wiki/{}", name);

    ureq::get(&path)
        .call()
        .ok()
        ?
        .into_string()
        .ok()
}

fn get_local_path(name: &str) -> String {
    let mut name_base64 = String::new();

    BASE64_STANDARD.encode_string(name, &mut name_base64);

    format!("db/pages/page{}.html", name_base64)
}

fn load_page_local(name: &str) -> Option<String> {
    std::fs::read_to_string(get_local_path(name)).ok()
}

fn save_page_local(name: &str, contents: &str) -> Option<()> {
    std::fs::write(get_local_path(name), contents).ok()
}

impl PageStorage {
    pub fn new() -> Self {
        Self {
            page_states: HashMap::new(),
        }
    }

    pub fn apply_config(&mut self, config: &str) {
        let state_iter = config
            .lines()
            .filter_map(|line| -> Option<(String, PageState)> {
                let line_arr = line.split(' ').collect::<Vec<_>>();

                Some((
                    line_arr.get(0)?.to_string(),
                    match line_arr.get(1)?.chars().next()? {
                        'f' => PageState::File,
                        'm' => PageState::Missing,
                        _ => return None
                    }
                ))
            });

        self.page_states.extend(state_iter);
    }

    pub fn generate_config(&self) -> String {
        self
            .page_states
            .iter()
            .map(|(name, state)| (name, match state { PageState::File => "f", _ => "m" }))
            .fold(String::new(), |collector, (name, state_str)| {
                collector + name + " " + state_str + "\n"
            })
    }

    pub fn load_page(&mut self, name: &str) -> Option<String> {
        if let Some(state) = self.page_states.get(name) {
            match *state {
                PageState::Missing => return None,

                PageState::File => {
                    if let Some(contents) = load_page_local(name) {
                        return Some(contents);
                    }

                    // fallback to web loading if state lies
                }
            }
        }

        if let Some(contents) = load_page_net(name) {
            if save_page_local(name, &contents).is_some() {
                self.page_states.insert(name.to_string(), PageState::File);
            }

            Some(contents)
        } else {
            self.page_states.insert(name.to_string(), PageState::Missing);
            None
        }
    }
}
