use std::io::{stdout};
use std::time::Duration;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, poll, KeyEvent},
    execute,
    terminal::{Clear, ClearType},
    Result, queue, style::Print
};  
use rand::{thread_rng, Rng};
use std::fmt::{Formatter, Display, self};

#[derive(Debug)]
struct Food {
    display_char : String, 
    x : usize,
    y : usize,
}

impl Food {
    fn new(max:usize, min:usize) -> Food {
        let mut rng = thread_rng();
        let x:usize = rng.gen_range(min..max);
        let y:usize = rng.gen_range(min..max);
        Food {display_char:"*".to_string(), x, y}
    }

}

#[derive(Debug)]
struct SnakePiece {
    display_char : String, 
    x : isize,
    y : isize,
    dx : isize,
    dy : isize,
}

#[derive(Debug)]
struct Snake {
    snake_body : Vec<SnakePiece>
}

impl Snake {
    fn new(x: isize, y : isize) -> Snake {
        Snake { snake_body : vec![SnakePiece{display_char : "#".to_string(), x, y, dx : 0, dy : 0}]}
    }

    fn update(&mut self, board : &mut Matrix) {
        let head = &mut self.snake_body[0];
        let (mut x, mut y, mut dx, mut dy) = (head.x, head.y, head.dx, head.dy);
        head.x += head.dx;
        head.y += head.dy;
        board.convert_back(x as usize, y as usize);
        for next in &mut self.snake_body[1..] {
            let (prev_x, prev_y, prev_dx, prev_dy) = (next.x, next.y, next.dx, next.dy);
            (next.x, next.y, next.dx, next.dy) = (x, y, dx, dy);
            board.convert_back(prev_x as usize, prev_y as usize);
            (x, y, dx, dy) = (prev_x, prev_y, prev_dx, prev_dy);

        }
        
    }

    fn move_around(&mut self, dir : char){
        let head = &mut self.snake_body[0];
        head.dx = 0;
        head.dy = 0;
        match dir {
            'w' => {head.dy = -1},
            'a' => {head.dx = -1},
            's' => {head.dy = 1},
            'd' => {head.dx = 1},
            _ => {}
        }
    }

    fn collision(&self, food : &mut Food) -> bool {
        let head = &self.snake_body[0];
        let (x, y) = (head.x as usize, head.y as usize);
        if x == food.x && y == food.y {
            // println!("Collided");
            return true;
        }
        false
    }

    fn extend(&mut self){
        let end: &SnakePiece = self.snake_body.last().unwrap();
        let new_snake : SnakePiece = SnakePiece {display_char : "#".to_string(), x : end.x - end.dx, y : end.y - end.dy, dx : end.dx, dy : end.dy };
        self.snake_body.push(new_snake);
    }
}
    


struct Point {
    display_char : String,
}

impl Point {
    fn new(display_char : String) -> Point {
        Point { display_char}
    }
}

struct Matrix {
    food : Food,
    size : usize,
    matrix : Vec<Vec<Point>>
}

impl Matrix {
    fn new(row : usize, col : usize) -> Matrix {
        let mut matrix = vec![];
        for _ in 0..row {
            let mut row = vec![];
            for _ in 0..col {
                let new_point = Point::new(".".to_string());
                row.push(new_point);
            }
            matrix.push(row);
        }
        Matrix { matrix, food: Food::new(row, 0) , size : row}
    }

    fn get_point(&mut self, x :usize, y: usize ) -> &mut Point {
        let matrix = self.matrix.as_mut_slice();
        &mut matrix[y][x]
    }

    fn convert_back(&mut self, x : usize, y:usize) {
        let point = self.get_point(x, y);
        point.display_char = ".".to_string();
    }

    fn draw_snake(&mut self, player : &Snake){
        for piece in player.snake_body.as_slice(){
            let (x, y) = (piece.x, piece.y);
            self.get_point(x as usize, y as usize).display_char = piece.display_char.to_string();
        }
    }
    
    fn draw_food(&mut self){
        let food = &self.food;
        let (x, y) = (food.x, food.y);
        self.get_point(x, y).display_char = food.display_char.to_string();

    }

    fn edge_collision(&mut self, player : &Snake) -> bool{
        let head = &player.snake_body[0];
        let size = self.size as isize;
        if head.x < 0 || head.x >=size || head.y < 0 || head.y >= size {
            return true;
        }
        false
    }

    fn game_loop(&mut self, player : &mut Snake) -> Result<()> {
        queue!(stdout(), cursor::Hide)?;
        loop {
            if poll(Duration::from_millis(1000/5))? {
                let event = read()?;

                if let Event::Key(KeyEvent { code, .. }) = event{
                    match code {
                        KeyCode::Char('w') => player.move_around('w'),
                        KeyCode::Char('a') => player.move_around('a'),
                        KeyCode::Char('s') => player.move_around('s'),
                        KeyCode::Char('d') => player.move_around('d'),
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }                
            }
            
            if player.collision(&mut self.food) {
                self.food = Food::new(self.size, 0);
                player.extend();
            }

            player.update(self);

            if self.edge_collision(&player) {
                queue!(stdout(), Print("Snake went out of bounds... Better luck next time.. "))?;
                break;
            }

            self.draw_food();
            self.draw_snake(player);
            
            queue!(stdout(),cursor::MoveTo(0, 0)  ,Clear(ClearType::FromCursorDown))?;
            execute!(stdout(), Print(&self))?;
        }
    
        Ok(())
    }
}

impl Display for Matrix {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result{
        let mut matrix = vec![];
        for row in self.matrix.as_slice() {
            let mut row_list = vec![];
            for point in row {
                row_list.push(point.display_char.as_str());
            }
            matrix.push(row_list.join(" "));
        }
        write!(f, "{}\n", matrix.join("\n"))
    }
}


fn main(){
    let mut board = Matrix::new(15, 15);
    let mut player = Snake::new(2, 2);  
    println!("{}", board);
    let _ = board.game_loop(&mut player);
}
