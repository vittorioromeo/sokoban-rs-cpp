#![deny(unused)]

use std::io::{self, Read};

#[derive(PartialEq, Copy, Clone)]
enum Tile {
    None,
    Wall,
    Goal,
}

#[derive(PartialEq, Copy, Clone)]
enum Obj {
    None,
    Player,
    Box,
}

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

#[derive(Clone)]
struct Layer<T>([T; BOARD_WIDTH * BOARD_HEIGHT]);

impl<T> std::ops::Index<Vec2D> for Layer<T> {
    type Output = T;

    fn index(&self, coord: Vec2D) -> &T {
        &self.0[Self::index(coord)]
    }
}

impl<T> std::ops::IndexMut<Vec2D> for Layer<T> {
    fn index_mut(&mut self, coord: Vec2D) -> &mut T {
        &mut self.0[Self::index(coord)]
    }
}

impl<T> Layer<T> {
    #[must_use]
    fn index((x, y): Vec2D) -> usize {
        x + y * BOARD_WIDTH
    }

    fn swap(&mut self, v1: Vec2D, v2: Vec2D) {
        self.0.swap(Self::index(v1), Self::index(v2));
    }
}

type Coord = usize;
type Vec2D = (Coord, Coord);
type Off2D = (isize, isize);

#[must_use]
fn tile_char(tile: Tile) -> char {
    // ANNOYANCE: Can't be `const fn`
    match tile {
        Tile::None => ' ',
        Tile::Wall => '▒',
        Tile::Goal => '○',
    }
}

#[must_use]
fn obj_char(obj: Obj, tile: Tile) -> char {
    // ANNOYANCE: Can't be `const fn`
    match obj {
        Obj::None => tile_char(tile),
        Obj::Player => '☻',
        Obj::Box => {
            if tile == Tile::Goal {
                '◙'
            } else {
                '■'
            }
        } // ANNOYANCE: verbose, ugly
    }
}

struct Board {
    tiles: Layer<Tile>,
    objects: Layer<Obj>,
}

impl Board {
    fn print(&self) {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let obj = self.objects[(x, y)];
                let tile = self.tiles[(x, y)];
                print!("{}", obj_char(obj, tile));
            }
            println!();
        }
    }

    #[must_use]
    fn find_player(&self) -> Vec2D {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                if self.objects[(x, y)] == Obj::Player {
                    return (x, y);
                }
            }
        }
        panic!("missing player");
    }

    #[must_use]
    fn count_goals(&self) -> usize {
        self.tiles.0.iter().filter(|&x| *x == Tile::Goal).count()
    }
}

#[must_use]
fn offset((px, py): Vec2D, (ox, oy): Off2D) -> Vec2D {
    ((px as isize + ox) as usize, (py as isize + oy) as usize)
}

struct Game {
    board: Board,
    player_index: Vec2D,
    goals_left: usize,
}

impl Game {
    #[must_use]
    fn new(board: Board) -> Game {
        Game {
            player_index: board.find_player(),
            goals_left: board.count_goals(),
            board,
        }
    }

    #[must_use]
    fn move_box(&mut self, source: Vec2D, off: Off2D) -> bool {
        let target = offset(source, off);

        if self.board.tiles[target] == Tile::Wall
            || self.board.objects[target] != Obj::None
        {
            return false;
        }

        if self.board.tiles[source] == Tile::Goal {
            self.goals_left += 1;
        }

        if self.board.tiles[target] == Tile::Goal {
            self.goals_left -= 1;
        }

        self.board.objects.swap(target, source);
        true
    }

    fn move_player(&mut self, off: (isize, isize)) {
        let target = offset(self.player_index, off);

        let couldnt_push_box = self.board.objects[target] == Obj::Box
            && !self.move_box(target, off);

        if self.board.tiles[target] == Tile::Wall || couldnt_push_box {
            return;
        }

        self.board.objects.swap(target, self.player_index);
        self.player_index = target;
    }

    fn print(&self) {
        self.board.print();
        println!("\nGoals left: {}\n", self.goals_left);
    }
}

static TILE_LAYER: Layer<Tile> = Layer({
    #[allow(non_snake_case)]
    let (o, H, X) = (Tile::None, Tile::Wall, Tile::Goal);

    #[rustfmt::skip]
    let layer =
        [H,H,H,H,H,H,H,H,
         H,H,o,o,o,o,o,H,
         H,o,o,o,o,o,o,H,
         H,o,o,o,o,o,o,H,
         H,o,o,o,H,o,X,H,
         H,o,o,o,o,o,X,H,
         H,o,o,o,X,X,X,H,
         H,H,H,H,H,H,H,H];
    layer
});

static OBJECT_LAYER: Layer<Obj> = Layer({
    #[allow(non_snake_case)]
    let (o, P, B) = (Obj::None, Obj::Player, Obj::Box);

    #[rustfmt::skip]
    let layer =
        [o,o,o,o,o,o,o,o,
         o,o,o,o,o,o,o,o,
         o,o,B,B,o,o,o,o,
         o,o,B,o,B,o,o,o,
         o,o,o,o,o,o,o,o,
         o,o,o,o,B,o,o,o,
         o,P,o,o,o,o,o,o,
         o,o,o,o,o,o,o,o];

    layer
});

#[must_use]
fn restart() -> bool {
    let mut game = Game::new(Board {
        tiles: TILE_LAYER.clone(),
        objects: OBJECT_LAYER.clone(),
    });

    loop {
        let _ = std::process::Command::new("clear").status();
        game.print();

        let input =
            io::stdin().lock().bytes().nth(0).unwrap().unwrap() as char;

        #[rustfmt::skip]
        match input as char {
            'w' => game.move_player(( 0, -1)),
            's' => game.move_player(( 0,  1)),
            'a' => game.move_player((-1,  0)),
            'd' => game.move_player(( 1,  0)),
            _   => ()
        };

        if input == 'r' {
            break true;
        }
        if input == 'q' {
            break false;
        }
    }
}

fn main() {
    while restart() {}
}
