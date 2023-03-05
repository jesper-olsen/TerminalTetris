use rand::prelude::*;
use std::{thread, time};

// Tetrominos - packed into 7 64 bit numbers.
// Each tetromino is 4 squares - needs 4*(2+2)=16 bits to describe.
// Hence 448 bits in total: 7 tetrominos * 4 orientations * 16 bits.
static BLOCK: [u64; 7] = [
    0b0010_0001_0101_0100_1001_0101_0100_0000_0010_0001_0101_0100_1001_0101_0100_0000,
    0b0110_0101_0001_0000_1000_0100_0101_0001_0110_0101_0001_0000_1000_0100_0101_0001,
    0b0101_0001_0100_0000_0101_0001_0100_0000_0101_0001_0100_0000_0101_0001_0100_0000,
    0b1001_1000_0100_0000_0010_0001_0100_0000_1001_0101_0001_0000_0010_0110_0101_0100,
    0b0001_0110_0101_0100_0101_1000_0100_0000_0101_0010_0001_0000_0100_1001_0101_0001,
    0b0011_0010_0001_0000_1100_1000_0100_0000_0011_0010_0001_0000_1100_1000_0100_0000,
    0b1000_1001_0101_0001_0110_0101_0100_0000_0001_1000_0100_0000_0110_0010_0001_0000,
];

struct Game {
    x: u8, // coor
    y: u8,
    r: u8,  // orientation
    px: u8, // old coor
    py: u8,
    pr: u8,
    p: u8, // tetromino
    tick: u8,
    score: u32,
    board: [[u8; 10]; 20], // 20 rows x 10 cols
}

// extract a bit packed number from a block
fn num(p: u8, r: u8, i: u8) -> u8 {
    (3 & BLOCK[p as usize] >> (r * 16 + i)) as u8
}

// calculate width-1 for tetromino
fn width(p: u8, r: u8) -> u8 {
    let p = (0..4).map(|i| num(p, r, i * 4 + 2)).fold((0, 9), |m, v| {
        (std::cmp::max(m.0, v), std::cmp::min(m.1, v))
    });
    p.0 - p.1
}

// calculate height-1 for tetromino
fn height(p: u8, r: u8) -> u8 {
    let p = (0..4).map(|i| num(p, r, i * 4)).fold((0, 9), |m, v| {
        (std::cmp::max(m.0, v), std::cmp::min(m.1, v))
    });
    p.0 - p.1
}

fn new_tetramino(g: &mut Game) {
    g.p = random::<u8>() % 7; // tetromino
    g.r = random::<u8>() % 4; // orientation
    g.x = random::<u8>() % (10 - width(g.p, g.r));
    g.y = 0;
    g.py = 0;
    g.pr = g.r;
    g.px = g.x;
}

fn draw_screen(g: &Game) {
    for (i, row) in g.board.iter().enumerate() {
        let i: i32 = (i.try_into()).unwrap();
        ncurses::mv(i + 1, 1);
        row.iter()
            .map(|v| {
                let v = *v as u32;
                if v != 0 {
                    ncurses::attron(0x40020 | v << 8);
                }
                ncurses::addstr("  ");
                ncurses::attroff(0x40020 | v << 8);
            })
            .for_each(drop);
    }
    let n: i32 = g.board.len().try_into().unwrap();
    ncurses::mv(n + 1, 1);
    ncurses::addstr(format!("Score: {}; Shape: {}", g.score, g.p).as_str());
    ncurses::refresh();
}

// place a tetramino on the board
fn set_piece(g: &mut Game, x: u8, y: u8, r: u8, v: u8) {
    for i in 0..4 {
        g.board[(num(g.p, r, i * 4) + y) as usize][(num(g.p, r, i * 4 + 2) + x) as usize] = v;
    }
}

// move a piece from old (p*) coords to new
fn update_piece(g: &mut Game) {
    set_piece(g, g.px, g.py, g.pr, 0);
    g.px = g.x;
    g.py = g.y;
    g.pr = g.r;
    set_piece(g, g.x, g.y, g.r, g.p + 1);
}

fn wipe_filled_rows(g: &mut Game) {
    for row in g.y..=g.y + height(g.p, g.r) {
        if g.board[row as usize]
            .iter()
            .map(|v| *v as u32)
            //.fold(1, |p, v| p * v)
            .product::<u32>()
            > 0
        {
            for i in (1..row).rev() {
                let i = i as usize;
                for j in 0..g.board[i + 1].len() {
                    g.board[i + 1][j] = g.board[i][j];
                }
                for j in 0..g.board[0].len() {
                    g.board[0][j] = 0;
                }
                g.score += 1;
            }
        }
    }
}

// check if placing p at (x,y,r) will hit something
fn check_hit(g: &mut Game, x: u8, y: u8, r: u8) -> bool {
    let bottom: u8 = (g.board.len() - 1).try_into().unwrap();
    if y + height(g.p, r) > bottom {
        return true;
    }
    set_piece(g, g.px, g.py, g.pr, 0);

    let hits = (0..4)
        .filter(|i| {
            g.board[(y + num(g.p, r, i * 4)) as usize][(x + num(g.p, r, i * 4 + 2)) as usize] != 0
        })
        .count();
    set_piece(g, g.px, g.py, g.pr, g.p + 1);
    hits > 0
}

fn do_tick(g: &mut Game) -> bool {
    g.tick += 1;
    if g.tick > 30 {
        // only update 1/30 of the time...
        g.tick = 0;
        if check_hit(g, g.x, g.y + 1, g.r) {
            if g.y == 0 {
                // overflow - game over
                return false;
            }
            wipe_filled_rows(g);
            new_tetramino(g);
        } else {
            g.y += 1;
            update_piece(g);
        }
    }
    true
}

fn runloop(g: &mut Game) {
    const Q: i32 = 'q' as i32;

    while do_tick(g) {
        thread::sleep(time::Duration::from_millis(10));
        let c: i32 = ncurses::getch();

        match c {
            ncurses::KEY_LEFT => {
                if g.x > 0 && !check_hit(g, g.x - 1, g.y, g.r) {
                    g.x -= 1;
                }
            }
            ncurses::KEY_RIGHT => {
                if g.x + width(g.p, g.r) < 9 && !check_hit(g, g.x + 1, g.y, g.r) {
                    g.x += 1;
                }
            }
            ncurses::KEY_DOWN => {
                while !check_hit(g, g.x, g.y + 1, g.r) {
                    g.y += 1;
                    update_piece(g);
                }
                wipe_filled_rows(g);
                new_tetramino(g);
            }
            ncurses::KEY_UP => {
                if c == ncurses::KEY_UP {
                    g.r = (g.r + 1) % 4;
                    while g.x + width(g.p, g.r) > 9 {
                        g.x -= 1;
                    }
                    if check_hit(g, g.x, g.y, g.r) {
                        g.x = g.px;
                        g.r = g.pr;
                    }
                }
            }
            Q => return,
            _ => (),
        }
        update_piece(g);
        draw_screen(g);
    }
}

fn main() {
    let mut game = Game {
        x: 0,
        y: 0,
        r: 0,
        pr: 0,
        px: 0,
        py: 0,
        p: 0,
        tick: 0,
        score: 0,
        board: [[0; 10]; 20],
    };
    new_tetramino(&mut game);

    ncurses::initscr();
    ncurses::start_color();
    for i in 1..8 {
        ncurses::init_pair(i, i, 0); // colours indexed by their position in the block
    }
    ncurses::resizeterm(22, 22);
    ncurses::noecho();
    ncurses::keypad(ncurses::stdscr(), true); // allow arrow keys
    ncurses::timeout(0);
    ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    ncurses::box_(ncurses::stdscr(), 0, 0);
    runloop(&mut game);
    ncurses::endwin();
    println!("Score: {}", game.score);
}
