use crate::checklist::Checklist;
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use regex::Regex;

// A preprocessor for collecting the `{{#check <name> | <description>}}` marks
// and generating a 'checklist' chapter.
pub struct ChecklistPre;

const NAME: &str = "checklist-preprocessor";

impl Preprocessor for ChecklistPre {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let mut checklist = Checklist::new();
        if let Some(cfg) = ctx.config.get_preprocessor(NAME) {
            checklist.update_config(cfg);
        }

        book.for_each_mut(|section: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *section {
                let content = collect_and_replace(&chapter, &mut checklist);
                chapter.content = content;
            }
        });

        let checklist_chapter = checklist.generate_chapter();
        book.sections.push(BookItem::Chapter(checklist_chapter));

        Ok(book)
    }
}

fn collect_and_replace(chapter: &Chapter, checklist: &mut Checklist) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)
            \{\{\s*                     # opening parens and whitespace
            \#check                     # macro tag
            \s+                         # separating whitespace
            (?P<name>[a-zA-Z\-0-9]+)    # ident
            \s*\|\s*                    # separator with whitespaces
            (?P<desc>[^\}]+)            # description
            \}\}                        # whitespace and closing parens"
        )
        .unwrap();
    }

    let s = &chapter.content;
    let mut replaced = String::new();
    let mut previous_end_index = 0;

    for cap in RE.captures_iter(&chapter.content) {
        let name = cap["name"].to_string();
        let desc = cap["desc"].to_string();
        let start_index = cap.get(0).unwrap().start();
        let end_index = cap.get(0).unwrap().end();

        replaced.push_str(&s[previous_end_index..start_index]);
        replaced.push_str(&format!("<a id=\"{}\"></a>", name));
        replaced.push_str(&name);
        previous_end_index = end_index;

        checklist.insert(&chapter.name, &chapter.path.as_ref().unwrap(), name, desc);
    }

    replaced.push_str(&s[previous_end_index..]);
    replaced
}
