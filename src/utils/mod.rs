pub trait ChessField {
    fn up(&self) -> Self;
    fn down(&self) -> Self;
    fn left(&self) -> Self;
    fn right(&self) -> Self;
    fn file(&self) -> char;
    fn rank(&self) -> char;
}

impl ChessField for String {
    fn up(&self) -> Self {
        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        let rank = (rank as u8 + 1) as char;

        let mut result = file.to_string();
        result.push(rank);
        result
    }

    fn down(&self) -> Self {
        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        let rank = (rank as u8 - 1) as char;

        let mut result = file.to_string();
        result.push(rank);
        result
    }

    fn left(&self) -> Self {
        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        let file = (file as u8 - 1) as char;

        let mut result = file.to_string();
        result.push(rank);
        result
    }

    fn right(&self) -> Self {
        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        let file = (file as u8 + 1) as char;

        let mut result = file.to_string();
        result.push(rank);
        result
    }

    fn file(&self) -> char {
        self.chars().nth(0).unwrap()
    }

    fn rank(&self) -> char {
        self.chars().nth(1).unwrap()
    }
}
