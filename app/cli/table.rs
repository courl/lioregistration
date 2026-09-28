pub struct Table {
    headers: Vec<String>,
    widths: Option<Vec<usize>>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub const fn new(headers: Vec<String>, widths: Option<Vec<usize>>) -> Self {
        Self {
            headers,
            widths,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) -> &Self {
        self.rows.push(row);
        self
    }

    fn calculate_widths(&self) -> Vec<usize> {
        if let Some(widths) = &self.widths {
            return widths.clone();
        }
        let mut widths = vec![0; self.headers.len()];
        for row in &self.rows {
            for (i, el) in row.iter().enumerate() {
                if i >= widths.len() {
                    break;
                }
                widths[i] = std::cmp::max(widths[i], el.len());
            }
        }
        widths
    }

    pub fn draw(&self) {
        let widths = self.calculate_widths();
        let draw_row = |row: &Vec<String>| {
            print!("\u{2502}");
            for (i, el) in row.iter().enumerate() {
                print!(
                    " {} \u{2502}",
                    if el.len() < widths[i] {
                        el.clone() + " ".repeat(widths[i] - el.len()).as_str()
                    } else {
                        el[..widths[i]].to_string()
                    }
                );
            }
            println!();
        };

        println!(
            "\u{256d}{}\u{256e}",
            widths
                .iter()
                .map(|w| { "\u{2500}".repeat(w + 2) })
                .reduce(|a, b| { a + "\u{252c}" + b.as_str() })
                .unwrap_or(String::new())
        );
        draw_row(&self.headers);
        println!(
            "\u{251c}{}\u{2524}",
            widths
                .iter()
                .map(|w| { "\u{2500}".repeat(w + 2) })
                .reduce(|a, b| { a + "\u{253c}" + b.as_str() })
                .unwrap_or(String::new())
        );
        self.rows.iter().for_each(draw_row);
        println!(
            "\u{2570}{}\u{256f}",
            widths
                .iter()
                .map(|w| { "\u{2500}".repeat(w + 2) })
                .reduce(|a, b| { a + "\u{2534}" + b.as_str() })
                .unwrap_or(String::new())
        );
    }
}
