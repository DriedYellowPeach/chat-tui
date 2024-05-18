use ratatui::text::{Line, Text};

const COLON_WIDTH: u16 = 1;
const COLOUMN_WIDTH: u16 = 1;
const PROMPT_UP_CORNER: char = '╔';
const PROMPT_DOWN_CORNER: char = '╚';
const PROMPT_HORIZON: char = '═';
const COLON: char = ':';
const FRAME_LEFTUP: char = '╭';
const FRAME_RIGTHUP: char = '╮';
const FRAME_LEFTDOWN: char = '╰';
const FRAME_RIGTHDOWN: char = '╯';
const FRAME_HORIZON: char = '─';
const FRAME_VERT: char = '│';

pub struct MessageBubble {
    content: String,
    author: String,
    bubble_width: u16,
    bubble_height: u16,
    message_width: u16,
    max_width: u16,
    shift_width: u16,
    margin_width: u16, // default to 1
    painting: Vec<Vec<char>>,
}

impl MessageBubble {
    pub fn new(max_width: u16, message: &str, author: &str) -> Self {
        let mut bbl = MessageBubble {
            bubble_width: 0,
            bubble_height: 0,
            content: message.to_owned(),
            author: author.to_owned(),
            message_width: 0,
            max_width,
            shift_width: 4,
            margin_width: 1,
            painting: Vec::new(),
        };
        bbl.init_width();
        bbl.init_height();
        bbl.init_painting();
        bbl
    }

    pub fn finish_painting(&mut self) {
        self.draw_prompt();
        self.draw_bubble_frame();
        self.draw_message_content();
    }

    fn init_height(&mut self) {
        let content_len = self.content.len() as u16; // truncate long messages?
        let message_height = content_len / self.message_width
            + if content_len % self.message_width == 0 {
                0
            } else {
                1
            };
        self.bubble_height = 2 + message_height + 1;
    }

    fn init_width(&mut self) {
        self.message_width = self.content.len() as u16;
        self.bubble_width =
            COLOUMN_WIDTH * 2 + self.message_width + self.margin_width * 2 + self.shift_width;
        self.bubble_width = std::cmp::min(self.bubble_width, self.max_width);
        self.message_width =
            self.bubble_width - self.shift_width - self.margin_width * 2 - COLOUMN_WIDTH * 2;
    }

    fn init_painting(&mut self) {
        let height = self.bubble_height as usize;
        let width = self.bubble_width as usize;
        self.painting = vec![vec![' '; width]; height];
    }

    fn draw_prompt(&mut self) {
        self.painting[0][0] = PROMPT_UP_CORNER;
        for i in 1..self.shift_width {
            self.painting[0][i as usize] = PROMPT_HORIZON;
        }

        for i in 0..self.author.len() + 2 {
            if self.shift_width as usize + i >= self.painting[0].len() {
                break;
            }

            let ch = if i == 0 {
                '<'
            } else if i == self.author.len() + 1 {
                '>'
            } else {
                self.author.chars().nth(i - 1).unwrap()
            };
            self.painting[0][self.shift_width as usize + i] = ch;
        }

        for i in 0..self.shift_width {
            let ch = if i == 0 {
                PROMPT_DOWN_CORNER
            } else if i == self.shift_width - 1 {
                COLON
            } else {
                PROMPT_HORIZON
            };
            self.painting[1][i as usize] = ch;
        }
    }

    fn draw_bubble_frame(&mut self) {
        let row_offset = 1;
        let col_offset = self.shift_width;
        let frame_width = self.bubble_width - self.shift_width;
        let frame_height = self.bubble_height - 1;

        for col in 0..frame_width {
            let ch = if col == 0 {
                FRAME_LEFTUP
            } else if col == frame_width - 1 {
                FRAME_RIGTHUP
            } else {
                FRAME_HORIZON
            };
            self.painting[row_offset as usize][(col + col_offset) as usize] = ch;
        }

        for col in 0..frame_width {
            let ch = if col == 0 {
                FRAME_LEFTDOWN
            } else if col == frame_width - 1 {
                FRAME_RIGTHDOWN
            } else {
                FRAME_HORIZON
            };
            self.painting[(row_offset + frame_height - 1) as usize][(col + col_offset) as usize] =
                ch;
        }

        for row in 1..frame_height - 1 {
            self.painting[(row + row_offset) as usize][col_offset as usize] = FRAME_VERT;
            self.painting[(row + row_offset) as usize][(frame_width - 1 + col_offset) as usize] =
                FRAME_VERT;
        }
    }

    fn draw_message_content(&mut self) {
        let row_offset = 2;
        let col_offset = self.shift_width + 1 + 1;
        for (ith, ch) in self.content.chars().enumerate() {
            let row = ith as u16 / self.message_width;
            let col = ith as u16 % self.message_width;
            self.painting[(row + row_offset) as usize][(col + col_offset) as usize] = ch;
        }
    }
}

// impl<'a> Into<Text<'a>> for MessageBubble {
//     fn into(self) -> Text<'a> {
//         Text::from(
//             self.painting
//                 .iter()
//                 .map(|row| Line::from(row.iter().collect::<String>()))
//                 .collect::<Vec<Line>>(),
//         )
//     }
// }

impl<'a> From<MessageBubble> for Text<'a> {
    fn from(value: MessageBubble) -> Self {
        Text::from(
            value
                .painting
                .iter()
                .map(|row| Line::from(row.iter().collect::<String>()))
                .collect::<Vec<Line>>(),
        )
    }
}

#[test]
fn test_draw_prompt() {
    let mut bbl = MessageBubble::new(
        20, // "hello world hello world hellow world hello world,",
        "hello", "kevin",
    );
    bbl.draw_prompt();
    bbl.draw_bubble_frame();
    bbl.draw_message_content();
    for i in 0..bbl.painting.len() {
        for j in 0..bbl.painting[i].len() {
            print!("{}", bbl.painting[i][j]);
        }
        println!();
    }
}
