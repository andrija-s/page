use chess::{Piece, Board, ChessMove, BoardStatus, Color, MoveGen, ALL_SQUARES};
use std::{cmp, collections::HashMap};

struct Bounds
{
  low: i32,
  high: i32,
  best: Option<ChessMove>,
  depth: isize,
}
const TOTAL_LIMIT: f64 = 10000.0;
const ITER_LIMIT: f64 = 1000.0;
impl Bounds
{
  fn new(lower: i32, upper: i32, best_move: Option<ChessMove>, remaining_depth: isize) -> Bounds 
  {
    Bounds { low: lower, high: upper, best: best_move, depth: remaining_depth }
  }
}

pub fn ai(board: &Board) -> (i32, Option<ChessMove>, usize)
{
  let mut table:HashMap<u64, Bounds> = HashMap::with_capacity(128000);
  let mut result = (0, None, 0);
  let start_total = crate::now();
  let mut end = start_total;
  let mut start_iter = start_total;
  let mut i = 1;
  while (end - start_iter) < ITER_LIMIT && (end - start_total) < TOTAL_LIMIT
  {
    start_iter = crate::now();
    result = mtdf(board, result.0, i, &mut table);
    end = crate::now();
    crate::log(&("depth: ".to_owned() + &i.to_string() + " time: " + &((end-start_iter)/1000.0).to_string() + " secs"));
    i += 1;
  }
  drop(table);
  return result;
}

fn mtdf(root_board: &Board, f: i32, depth: isize, table: &mut HashMap<u64, Bounds>) -> (i32, Option<ChessMove>, usize)
{
  let mut guess = f;
  let mut best_move = None;
  let mut nodes_traversed = 0;
  let mut upperbound = i32::MAX;
  let mut lowerbound = i32::MIN;

  while lowerbound < upperbound
  {
    let beta = if guess == lowerbound { guess + 1 } else { guess };
    (guess, best_move, nodes_traversed) = ab_with_mem(root_board, beta - 1, beta, depth, table, true);
    
    if guess < beta { upperbound = guess; }
    else            { lowerbound = guess; }
  }
  return (guess, best_move, nodes_traversed);
}

fn get_ai_color(board: &Board, max_player: bool) -> Color 
{
  match board.side_to_move() {
    Color::White => return if max_player { Color::White } else { Color::Black },
    Color::Black => return if max_player { Color::Black } else { Color::White },
  }
}

fn eval_board(board: &Board, player: &Color) -> i32
{
  let mut value = 0;
  for sq in ALL_SQUARES.iter() {
    let col = board.color_on(*sq);
    if board.piece_on(*sq).is_none() || col.is_none() { continue; };
    let c = if col==Some(*player) { 1 } else { -1 };
    match board.piece_on(*sq) {
      Some(Piece::Bishop) => { value += (330) * c; },
      Some(Piece::Knight) => { value += (320) * c; },
      Some(Piece::Queen)  => { value += (900) * c; },
      Some(Piece::Rook)   => { value += (500) * c; },
      Some(Piece::Pawn)   => { value += (100) * c; },
      _ => (),
    }
  }
  return value;
}

fn ab_with_mem(board: &Board, mut alpha: i32, mut beta: i32, depth: isize, table: &mut HashMap<u64, Bounds>, max_player: bool) -> (i32, Option<ChessMove>, usize) 
{
  let entry = table.get(&board.get_hash());
  let mut sum = 1;
  let mut best_value = if max_player { i32::MIN } else { i32::MAX };
  let mut best_move = None;
  match entry {
    Some(t) => 
    {
      if t.depth >= depth 
      {
        if t.low >= beta { return (t.low, t.best, sum) }
        if t.high <= alpha { return (t.high, t.best, sum) }
        alpha = cmp::max(alpha, t.low);
        beta = cmp::min(beta, t.high);
        best_move = t.best;
      }
    },
    None => ()
  }

  if depth < 1
  {
    match board.status() {
      BoardStatus::Stalemate => best_value = 0,
      BoardStatus::Ongoing   => best_value = eval_board(board, &get_ai_color(board, max_player)),
      BoardStatus::Checkmate => ()
    }
  }
  else if max_player
  {
    let mut a = alpha;
    if best_move.is_some() 
    {
      let temp_result =  ab_with_mem(&board.make_move_new(best_move.unwrap()), a, beta, depth - 1, table, !max_player);
      best_value = temp_result.0;
      sum += temp_result.2;
      a = cmp::max(a, best_value);
    }
    if best_value < beta
    {
      let move_it = MoveGen::new_legal(board);
      for m in move_it
      {
        let temp_result =  ab_with_mem(&board.make_move_new(m), a, beta, depth - 1, table, !max_player);
        if temp_result.0 > best_value
        {
          best_value = temp_result.0;
          best_move = Some(m);
        } 
        sum += temp_result.2;
        a = cmp::max(a, best_value);
        if best_value >= beta { break; }
      }
    }
  }
  else
  {
    let mut b = beta;
    if best_move.is_some()
    {
      let temp_result =  ab_with_mem(&board.make_move_new(best_move.unwrap()), alpha, b, depth - 1, table, !max_player);
      best_value = temp_result.0;
      sum += temp_result.2;
      b = cmp::min(b, best_value);
    }
    if best_value > alpha
    {
      let move_it = MoveGen::new_legal(board);
      for m in move_it
      {
        let temp_result =  ab_with_mem(&board.make_move_new(m), alpha, b, depth - 1, table, !max_player);
        if temp_result.0 < best_value
        {
          best_value = temp_result.0;
          best_move = Some(m);
        } 
        sum += temp_result.2;
        b = cmp::min(b, best_value);
        if best_value <= alpha { break; }
      }
    }
  }
  let mut temp_lower = i32::MIN;
  let mut temp_upper = i32::MAX;
  if best_value <= alpha
  {
    temp_upper = best_value;
    table.insert((*board).get_hash(), Bounds::new(temp_lower, temp_upper, best_move, depth));
  }
  else if best_value > alpha && best_value < beta
  {
    temp_lower = best_value;
    temp_upper = best_value;
    table.insert((*board).get_hash(), Bounds::new(temp_lower, temp_upper, best_move, depth));
  }
  else if best_value >= beta {
    temp_lower = best_value;
    table.insert((*board).get_hash(), Bounds::new(temp_lower, temp_upper, best_move, depth));
  }

  return (best_value, best_move, sum)
}

/*
/* Traditional transposition table storing of bounds */
/* Fail low result implies an upper bound */
if g <= alpha then n.upperbound := g; store n.upperbound;
/* Found an accurate minimax value - will not occur if called with zero window */
if g >  alpha and g < beta then
n.lowerbound := g; n.upperbound := g; store n.lowerbound, n.upperbound;
/* Fail high result implies a lower bound */
if g >= beta then n.lowerbound := g; store n.lowerbound;
return g;
 */