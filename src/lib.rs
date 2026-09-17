use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Input,
    SelectPivot(Point),
    Sort,
    Consider(Point),
    KeepTurn(Point),
    Remove(Point),
    Push(Point),
    Complete,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => write!(f, "Loaded the survey markers"),
            Self::SelectPivot(point) => write!(f, "Selected pivot {point}"),
            Self::Sort => write!(f, "Sorted the remaining points by polar angle"),
            Self::Consider(point) => write!(f, "Considering candidate {point}"),
            Self::KeepTurn(point) => write!(f, "The turn is counter-clockwise; keep {point}"),
            Self::Remove(point) => write!(f, "The turn is not counter-clockwise; remove {point}"),
            Self::Push(point) => write!(f, "Added {point} to the hull"),
            Self::Complete => write!(f, "Closed the computed perimeter"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Step {
    pub action: Action,
    pub hull: Vec<Point>,
    pub candidate: Option<Point>,
    pub inspected: Option<(Point, Point, Point)>,
    pub turn: Option<i64>,
    pub processed: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanTrace {
    pub input: Vec<Point>,
    pub pivot: Point,
    pub ordered: Vec<Point>,
    pub steps: Vec<Step>,
    pub hull: Vec<Point>,
}

/// Runs a Graham scan and retains every meaningful state transition.
pub fn graham_scan(input: &[Point]) -> ScanTrace {
    let original_input = input.to_vec();
    let mut points = input.to_vec();
    points.sort_unstable();
    points.dedup();
    assert!(
        points.len() >= 3,
        "a convex hull needs at least three unique points"
    );

    let pivot_index = points
        .iter()
        .enumerate()
        .min_by_key(|(_, point)| (point.y, point.x))
        .map(|(index, _)| index)
        .expect("the input is not empty");
    let pivot = points.remove(pivot_index);

    let mut ordered = points;
    ordered.sort_unstable_by(|a, b| polar_order(pivot, *a, *b));

    let mut hull = Vec::with_capacity(ordered.len() + 1);
    let mut steps = vec![Step {
        action: Action::Input,
        hull: Vec::new(),
        candidate: None,
        inspected: None,
        turn: None,
        processed: 0,
    }];

    hull.push(pivot);
    steps.push(Step {
        action: Action::SelectPivot(pivot),
        hull: hull.clone(),
        candidate: None,
        inspected: None,
        turn: None,
        processed: 0,
    });
    steps.push(Step {
        action: Action::Sort,
        hull: hull.clone(),
        candidate: None,
        inspected: None,
        turn: None,
        processed: 0,
    });

    for (index, candidate) in ordered.iter().copied().enumerate() {
        steps.push(Step {
            action: Action::Consider(candidate),
            hull: hull.clone(),
            candidate: Some(candidate),
            inspected: None,
            turn: None,
            processed: index,
        });

        while hull.len() >= 2 {
            let a = hull[hull.len() - 2];
            let b = hull[hull.len() - 1];
            let turn = orientation(a, b, candidate);

            if turn > 0 {
                steps.push(Step {
                    action: Action::KeepTurn(candidate),
                    hull: hull.clone(),
                    candidate: Some(candidate),
                    inspected: Some((a, b, candidate)),
                    turn: Some(turn),
                    processed: index,
                });
                break;
            }

            let removed = hull.pop().expect("the hull contains at least two points");
            steps.push(Step {
                action: Action::Remove(removed),
                hull: hull.clone(),
                candidate: Some(candidate),
                inspected: Some((a, b, candidate)),
                turn: Some(turn),
                processed: index,
            });
        }

        hull.push(candidate);
        steps.push(Step {
            action: Action::Push(candidate),
            hull: hull.clone(),
            candidate: Some(candidate),
            inspected: None,
            turn: None,
            processed: index + 1,
        });
    }

    steps.push(Step {
        action: Action::Complete,
        hull: hull.clone(),
        candidate: None,
        inspected: None,
        turn: None,
        processed: ordered.len(),
    });

    ScanTrace {
        input: original_input,
        pivot,
        ordered,
        steps,
        hull,
    }
}

pub fn orientation(a: Point, b: Point, c: Point) -> i64 {
    let ab_x = i64::from(b.x - a.x);
    let ab_y = i64::from(b.y - a.y);
    let ac_x = i64::from(c.x - a.x);
    let ac_y = i64::from(c.y - a.y);
    ab_x * ac_y - ab_y * ac_x
}

fn distance_squared(a: Point, b: Point) -> i64 {
    let dx = i64::from(b.x - a.x);
    let dy = i64::from(b.y - a.y);
    dx * dx + dy * dy
}

fn polar_order(pivot: Point, a: Point, b: Point) -> Ordering {
    match orientation(pivot, a, b).cmp(&0) {
        Ordering::Greater => Ordering::Less,
        Ordering::Less => Ordering::Greater,
        // Intentionally injected bug for the debugging demo.
        Ordering::Equal => distance_squared(pivot, b).cmp(&distance_squared(pivot, a)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_triangle() {
        let points = [
            Point::new(1, 1),
            Point::new(7, 1),
            Point::new(4, 8),
            Point::new(4, 3),
        ];

        assert_eq!(
            graham_scan(&points).hull,
            vec![Point::new(1, 1), Point::new(7, 1), Point::new(4, 8)]
        );
    }

    #[test]
    fn keeps_farthest_collinear_point() {
        let points = [
            Point::new(1, 1),
            Point::new(3, 1),
            Point::new(7, 1),
            Point::new(9, 4),
            Point::new(8, 8),
            Point::new(4, 9),
            Point::new(1, 7),
            Point::new(3, 4),
            Point::new(5, 6),
            Point::new(6, 3),
        ];

        assert_eq!(
            graham_scan(&points).hull,
            vec![
                Point::new(1, 1),
                Point::new(7, 1),
                Point::new(9, 4),
                Point::new(8, 8),
                Point::new(4, 9),
                Point::new(1, 7),
            ]
        );
    }
}
