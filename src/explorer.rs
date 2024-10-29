use crate::{log, page_graph, page_storage};

fn get_reference_names<'t>(page: &'t str) -> Vec<&'t str> {
    let Some((begin, end)) = Option::zip(
        page.find("mw-parser-output"),
        page.find("margin-top: 1.8em"),
    ) else {
        return Vec::new();
    };

    let mut result = Vec::new();

    let mut text = &page[begin..end];
    const WIKI_REF_HEADER: &'static str = "\"/wiki/";

    while let Some(name_start_index) = text.find(WIKI_REF_HEADER) {
        text = &text[name_start_index + WIKI_REF_HEADER.len()..];
        
        let Some(name_end) = text.find('\"') else {
            break;
        };
        let mut name = &text[..name_end];

        if let Some(pos) = name.find(':') {
            name = &name[0..pos];
        }

        if let Some(pos) = name.find('#') {
            name = &name[0..pos];
        }

        result.push(name);
    }

    result
}

/// Site explorer representation structure
struct Explorer {
    /// Logger
    logger: log::Logger,

    /// Page storage
    page_storage: page_storage::PageStorage,

    /// Page graph
    page_graph: page_graph::PageGraph,

    /// Current reading generation index
    global_generation_index: usize,

    /// Current page index
    global_page_index: usize,
}

impl Explorer {
    pub fn new() -> Explorer {
        let graph = std::fs::read_to_string("db/db.json")
            .ok()
            .map(|v| json::from_str(&v).ok())
            .flatten()
            .map(|v| page_graph::PageGraph::from_json(&v))
            .flatten()
            .unwrap_or(page_graph::PageGraph::new())
        ;

        let mut storage = page_storage::PageStorage::new();

        storage.apply_config(
            &std::fs::read_to_string("db/page_storage_config").unwrap_or(String::new())
        );

        Explorer {
            global_generation_index: 0,
            global_page_index: 0,
            logger: log::Logger::new("db/log.txt"),

            page_graph: graph,
            page_storage: storage,
        }
    }

    /// Single page exploration function
    fn explore_page(&mut self, page_name: &str) {
        if let Some(contents) = self.page_storage.load_page(&page_name) {
            self.page_graph.insert_page(page_name, get_reference_names(&contents));
            log!(self.logger, "ok");
        } else {
            log!(self.logger, "error");
        }

        self.global_page_index += 1;
        // save all kal every 8 pages readed
        if self.global_page_index % 8 == 0 {
            self.log_configs();
        }
    } // explore_page

    /// Page generation exploration index
    fn explore_generation(&mut self, generation_pages: &[String]) {
        log!(self.logger, "GENERATION #{} STARTED. {} PAGES TO EXPLORE.\n", self.global_generation_index, generation_pages.len());
        for (page_index, page_name) in generation_pages.iter().enumerate() {
            log!(self.logger, "    {:5}/{}: ", page_index + 1, generation_pages.len());

            self.explore_page(page_name);

            log!(self.logger, ". path: {}\n", page_name);
        }
        log!(self.logger, "GENERATION #{} FINISHED\n", self.global_generation_index);

        self.global_generation_index += 1;
        self.log_configs();
    } // explore pages

    fn log_configs(&self) {
        // rewrite configuration and database

        // write page storage config
        _ = std::fs::write(
            "db/page_storage_config",
            self.page_storage.generate_config()
        );

        // write page graph to db.json file

        let json = self.page_graph.to_json();
        let text = json::to_string_pretty(&json).unwrap();

        if std::fs::write("db/db.json", text).is_err() {
            log!(self.logger, "Error writing database to file.\n");
        }

        self.logger.flush();
    }

    pub fn explore(&mut self, root: &str) {
        log!(self.logger, "EXPLORATION STARTED\n\n");

        let mut generation_pages = vec![root.to_string()];

        loop {
            if self.global_generation_index >= 4 {
                break;
            }

            self.explore_generation(&generation_pages);

            generation_pages = self.page_graph
                .iter_unknown_pages()
                .map(|(name, _)| name.to_string())
                .collect();
        }

        log!(self.logger, "EXPLORATION FINISHED\n\n");
    }
}

pub fn explore(root: &str) {
    Explorer::new().explore(root);
}
