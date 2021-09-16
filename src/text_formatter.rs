use crate::enums::{Delimiters, NewLineState};

///For formatting the output file
pub(crate) struct StringFormatter {
    pub string_container: String,
    current_number_of_tabs: u16,
    delimiters: Vec<Delimiters>,
}

impl StringFormatter {
    ///For a new struct the current number of tabs added should be 0
    pub fn new(string_container: String, current_number_of_tabs: u16) -> StringFormatter {
        StringFormatter {
            string_container,
            current_number_of_tabs,
            delimiters: Vec::with_capacity(2),
        }
    }

    ///Function to properly add a new line to the output file\
    /// It shouldn't be used directly\
    /// How it works: We add a new line and then add the number of tabs
    /// in anticipation of new text being added
    fn add_newline(&mut self, state: NewLineState) {
        match state {
            NewLineState::Current => {}
            NewLineState::ShiftRight => self.current_number_of_tabs += 1,
            NewLineState::ShiftLeft => self.current_number_of_tabs -= 1,
        };
        //take effect
        let new_tabs = "\t".repeat(self.current_number_of_tabs as usize);
        self.string_container.push_str("\n");
        self.string_container.push_str(&new_tabs);
    }

    #[inline]
    fn add_text_from_vec(&mut self, vec: Vec<&str>) {
        vec.into_iter()
            .for_each(|it| self.string_container.push_str(it));
    }

    #[inline]
    pub fn add_text_and_then_line(&mut self, vec: Vec<&str>, state: NewLineState) {
        assert!(!vec.is_empty());
        self.add_text_from_vec(vec);
        self.add_newline(state)
    }

    pub fn add_text_delimiter_then_line(
        &mut self,
        vec: Vec<&str>,
        delimiter: Delimiters,
        state: NewLineState,
    ) {
        self.add_text_from_vec(vec);
        match delimiter {
            Delimiters::Bracket => self.add_start_bracket(),
            Delimiters::Parenthesis => {
                self.add_start_parenthesis();
            }
        }
        self.add_newline(state);
    }

    fn add_start_parenthesis(&mut self) {
        self.delimiters.push(Delimiters::Parenthesis);
        self.string_container.push_str("(")
    }

    pub fn add_start_bracket(&mut self) {
        self.delimiters.push(Delimiters::Bracket);
        self.string_container.push_str(" {")
    }

    ///Ideally after a colon, we move to the same line with no additional tabs
    fn add_colon(&mut self) {
        self.string_container.push_str(";");
        self.add_newline(NewLineState::Current)
    }

    ///Ideally after a comma (in rust), we move to the same line with no additional tabs
    fn add_comma(&mut self) {
        self.string_container.push_str(",");
        self.add_newline(NewLineState::Current)
    }

    pub fn add_text_and_colon(&mut self, vec: Vec<&str>) {
        self.add_text_from_vec(vec);
        self.add_colon()
    }

    pub fn add_text_and_comma(&mut self, vec: Vec<&str>) {
        self.add_text_from_vec(vec);
        self.add_comma()
    }

    pub fn close_all_delimiters(&mut self) {
        while let Some(delimiter) = self.delimiters.pop() {
            let tab = self.string_container.pop().unwrap(); //remove the tab
            assert_eq!(tab, '\t');
            self.add_text_and_then_line(
                vec![match delimiter {
                    Delimiters::Parenthesis => ")",
                    Delimiters::Bracket => "}",
                }],
                NewLineState::ShiftLeft,
            )
        }
        assert_eq!(
            self.string_container.trim_end().len(),
            self.string_container.len() - 1
        ); //Since the last character is just a new line
        self.string_container.pop().unwrap();
        //add ";" to the end
        self.string_container.push_str(";\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::{Delimiters, NewLineState};
    use crate::text_formatter::StringFormatter;

    #[test]
    fn testing_various_states() {
        let mut format = StringFormatter::new(String::new(), 0);
        format.add_text_and_then_line(vec!["fn ", "main", "{"], NewLineState::ShiftRight);
        format.add_text_and_then_line(vec!["let", "x", "= 15 {"], NewLineState::ShiftRight);
        format.add_text_and_then_line(vec!["32"], NewLineState::Current);
        format.add_text_and_then_line(vec!["i get", " it"], NewLineState::Current);
        format.add_text_and_then_line(vec!["let", "x", "= 15 {"], NewLineState::ShiftRight);
        format.add_text_and_then_line(vec!["let", "y", "= 15"], NewLineState::ShiftLeft);
        println!("{}", format.string_container);
    }

    #[test]
    fn testing_delimiters() {
        let mut format = StringFormatter::new(String::new(), 0);
        format.add_text_delimiter_then_line(
            vec!["fn ", "main"],
            Delimiters::Bracket,
            NewLineState::ShiftRight,
        );
        format.add_text_and_colon(vec!["let", "x", "= 15"]);
        format.add_text_and_colon(vec!["let", "y", "= 34"]);
        format.add_text_delimiter_then_line(
            vec!["foreign_enum!"],
            Delimiters::Parenthesis,
            NewLineState::ShiftRight,
        );
        format.add_text_and_then_line(vec!["just a doc"], NewLineState::Current);
        format.add_text_delimiter_then_line(
            vec!["enum Trial"],
            Delimiters::Bracket,
            NewLineState::ShiftRight,
        );
        format.add_text_and_comma(vec!["Trial::Here"]);
        println!(
            "{} {}",
            format.string_container, format.current_number_of_tabs
        );
        format.close_all_delimiters();
        println!("{}", format.string_container);
    }
}
