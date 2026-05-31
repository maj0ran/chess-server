use chess_core::ChessColor;
use smol::lock::Mutex;
use smol::Timer;
use std::sync::Arc;
use std::time::Instant;

/**
 * Struct that represents a chess clock. A `ChessClock` has a time block (`ChessClockTime`), that
 * contains the time and increment for each player, as well as the active player.
 * The `clock_task` is a Task that _observes_ the time by polling it every second. This way,
 * the clock detects when a player runs out of time.
**/
pub struct ChessClock {
    pub time: Arc<Mutex<ChessClockTime>>,
    clock_task: Option<smol::Task<()>>,
}

/**
 * The time block inside a `ChessCock`.
**/
pub struct ChessClockTime {
    pub white: u32,
    pub black: u32,

    white_inc: u32,
    black_inc: u32,

    active_player: ChessColor,

    last_time: Instant,
}

impl ChessClock {
    /**
     * Creates a new chess clock. Does not start the task to observe the time yet.
     **/
    pub fn new(white_time: u32, black_time: u32, white_inc: u32, black_inc: u32) -> Self {
        let time = Arc::new(Mutex::new(ChessClockTime {
            white: white_time,
            black: black_time,
            white_inc,
            black_inc,

            active_player: ChessColor::White,

            last_time: Instant::now(),
        }));

        ChessClock {
            time,
            clock_task: None,
        }
    }

    /**
     * Press the clock. This will switch the active player and subtracts the time used since the last press.
     **/
    pub async fn press(&mut self) -> u32 {
        let mut time = self.time.lock().await;

        let ts = Instant::now();
        let time_used = ts.duration_since(time.last_time).as_millis() as u32;

        if time.active_player == ChessColor::White {
            time.white = time.white.saturating_sub(time_used);
            time.white += time.white_inc;
        } else {
            time.black = time.black.saturating_sub(time_used);
            time.black += time.black_inc;
        }

        time.last_time = ts;
        time.active_player = !time.active_player;

        log::info!("White time: {}s, Black time: {}s", time.white, time.black);
        time_used
    }

    /**
     * Starts the clock task. This will observe the time and detect when a player runs out of time.
     **/
    pub fn start(&mut self) {
        let time_poll = self.time.clone();

        self.clock_task = Some(smol::spawn(async move {
            log::info!("Clock started");

            time_poll.lock().await.last_time = Instant::now();

            loop {
                Timer::after(std::time::Duration::from_secs(1)).await;
                let mut t = time_poll.lock().await;
                let ts = Instant::now();
                let time_used = ts.duration_since(t.last_time).as_millis() as u32;

                if t.active_player == ChessColor::White {
                    t.white = t.white.saturating_sub(time_used);
                    if t.white == 0 {
                        log::info!("White time ran out!");
                        break;
                    }
                } else {
                    t.black = t.black.saturating_sub(time_used);
                    if t.black == 0 {
                        log::info!("Black time ran out!");
                        break;
                    }
                }
                log::trace!("Time remaining: White: {}s, Black: {}s", t.white, t.black);
            }
            log::info!("Clock stopped");
        }));
    }
}
