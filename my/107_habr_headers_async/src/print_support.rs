use super::article::HabrArticle;
use prettytable::{color, Attr, Cell, Row, Table};
use std::collections::HashSet;

fn text_to_multiline(text: &str, symbols_on_line: usize, intent: Option<&str>) -> String {
    let mut line_symb_count = 0;
    let mut multiline_text: String = text
        .split(' ')
        .map(|word| {
            if let Some(last) = word.chars().last() {
                if last == '\n' {
                    line_symb_count = 0;
                }
            }

            let future_line_size = line_symb_count + word.len() + 1;
            if (future_line_size < symbols_on_line) || (line_symb_count == 0) {
                line_symb_count += word.len() + 1;
                vec![word, " "]
            } else if let Some(intent) = intent {
                line_symb_count = intent.len() + word.len() + 1;
                vec!["\n", intent, word, " "]
            } else {
                line_symb_count = word.len() + 1;
                vec!["\n", word, " "]
            }
        })
        .flatten()
        .collect();

    // Убираем пробел или \n в конце
    while let Some(last) = multiline_text.chars().last() {
        if last.is_whitespace() {
            multiline_text.pop();
        } else {
            break;
        }
    }

    multiline_text
}

pub async fn print_results(selected: &[HabrArticle], previous_results: Option<HashSet<String>>) {
    // Create the table
    let mut table = Table::new();

    // let format = format::FormatBuilder::new()
    //     .indent(0)
    //     .column_separator(' ')
    //     .borders(' ')
    //     .separators(&[format::LinePosition::Intern,
    //                 format::LinePosition::Bottom],
    //                 format::LineSeparator::new(' ', ' ', ' ', ' '))
    //     .padding(1, 1)
    //     .build();
    // table.set_format(format);

    // table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    // Выводим текст, используем into_iter для потребляющего итератора
    for info in selected.iter().rev() {
        let multiline_text = text_to_multiline(&info.title, 30, None);
        // multiline_text.push_str("\n\n");
        // multiline_text.push_str(&info.tags);

        let tags_text = text_to_multiline(&info.tags.join("\n"), 25, Some(" "));

        let text_color = previous_results
            .as_ref()
            .map(|set| {
                if set.contains(info.link.as_str()) {
                    color::YELLOW
                } else {
                    color::GREEN
                }
            })
            .unwrap_or(color::GREEN);

        let row = Row::new(vec![
            Cell::new(&multiline_text).with_style(Attr::ForegroundColor(text_color)),
            Cell::new(&tags_text).with_style(Attr::ForegroundColor(color::WHITE)),
            Cell::new(&info.time).with_style(Attr::ForegroundColor(color::WHITE)),
            Cell::new(&info.link),
        ]);
        table.add_row(row);
    }

    // Print the table to stdout
    table.printstd();
}
