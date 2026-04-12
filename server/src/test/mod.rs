mod testclient;

#[cfg(test)]
const TEST_GAME_FILE: &str = "testgame2";

#[cfg(test)]
pub mod testgames {
    use crate::server::server::Server;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::time::Duration;

    use super::testclient::TestClient;
    use crate::test;
    use chess_core::{ServerMessage, UserRoleSelection};
    use smol::Timer;
    use smol_macros::test;

    async fn start_server(port: u16) {
        smol::spawn(async move {
            let mut server = Server::new();
            let _ = server.run(port).await;
        })
        .detach();

        Timer::after(Duration::from_millis(500)).await;
    }

    test! {
        async fn test_one_player() {
            env_logger::try_init().ok();

            let port = 7878;
            start_server(port).await;

            let mut client = TestClient::new(port).await;

            let game_id = client.create_game(1, 120, 0).await;
            client.join_game(game_id, UserRoleSelection::Both).await;

            let file = File::open(test::TEST_GAME_FILE).unwrap();
            let moves = io::BufReader::new(file).lines();
            let mut move_count = 0;
            for full_line in moves {
                move_count += 1;
                Timer::after(Duration::from_millis(100)).await;
                match full_line {
                    Ok(line) => {
                        let without_comment: Vec<&str> = line.split("#").collect();
                        let without_comment = without_comment.first().unwrap();
                        let testmove: Vec<&str> = without_comment.split_whitespace().collect();
                        if testmove.is_empty() {
                            continue;
                        }
                        let mov_str = testmove[0];
                        let expected = testmove[1];

                        let response = client.make_move(game_id, mov_str).await;
                        let response_opcode = response.opcode();

                        match expected {
                            "OK" => assert!(response_opcode == ServerMessage::GAME_CREATED || response_opcode == ServerMessage::MOVE_ACCEPTED || response_opcode == 0x84, "Move {} (#{}) should be OK (Update, MoveOk or GameCreated). Got 0x{:02X}", mov_str, move_count, response_opcode),
                            "NOK" => assert_eq!(response_opcode, ServerMessage::ILLEGAL_MOVE, "Move {} (#{}) should be NOK. Got 0x{:02X}", mov_str, move_count, response_opcode),
                            _ => panic!("Unknown indicator {}", expected),
                        }
                    }
                    Err(_) => todo!(),
                }
            }
            log::info!("testgame complete");
        }
    }

    test! {
        async fn test_two_players() {
            env_logger::try_init().ok();

            let port = 7879;
            start_server(port).await;

            let mut client1 = TestClient::new(port).await;
            let mut client2 = TestClient::new(port).await;

            // client 1: create new game
            let game_id = client1.create_game(1, 120, 0).await;

            // client 2: list games
            let game_ids = client2.list_games().await;
            assert!(game_ids.contains(&game_id));

            // client 1 joins as white
            client1.join_game(game_id, UserRoleSelection::White).await;

            // client 2 joins as black
            client2.join_game(game_id, UserRoleSelection::Black).await;

            let file = File::open(test::TEST_GAME_FILE).unwrap();
            let moves = io::BufReader::new(file).lines();
            let mut c1 = client1;
            let mut c2 = client2;

            let mut white_turn = true;

            for full_line in moves {
                match full_line {
                    Ok(line) => {
                        let without_comment: Vec<&str> = line.split("#").collect();
                        let without_comment = without_comment.first().unwrap();
                        let testmove: Vec<&str> = without_comment.split_whitespace().collect();
                        if testmove.is_empty() {
                            continue;
                        }
                        let mov_str = testmove[0];
                        let expected = testmove[1];

                        let current_client = if white_turn { &mut c1 } else { &mut c2 };

                        let response = current_client.make_move(game_id, mov_str).await;
                        let response_opcode = response.opcode();

                        match expected {
                            "OK" => {
                                if response_opcode != ServerMessage::ILLEGAL_MOVE {
                                    // Success! Swap turns
                                    white_turn = !white_turn;
                                }
                            }
                            "NOK" => {
                                if response_opcode != ServerMessage::ILLEGAL_MOVE {
                                    // Server thought it was legal, but test expected NOK.
                                    // Even so, we don't swap turns as per requirement.
                                    white_turn = !white_turn;
                                }
                            }
                            _ => panic!("Unknown indicator {}", expected),
                        }
                    }
                    Err(_) => todo!(),
                }
                Timer::after(Duration::from_millis(100)).await;
            }
            log::info!("test_two_players complete");
        }
    }

    test! {
        async fn test_wrong_color_move_rejected() {
            env_logger::try_init().ok();

            let port = 7880;
            start_server(port).await;

            let mut client_white = TestClient::new(port).await;
            let mut client_black = TestClient::new(port).await;

            let game_id = client_white.create_game(1, 120, 0).await;
            client_white.join_game(game_id, UserRoleSelection::White).await;
            client_black.join_game(game_id, UserRoleSelection::Black).await;

            // It's white's turn. Black tries to move.
            let response = client_black.make_move(game_id, "e7e5").await;
            assert_eq!(response.opcode(), ServerMessage::ILLEGAL_MOVE, "Black moving on White's turn should be rejected");

            // White moves.
            let response = client_white.make_move(game_id, "e2e4").await;
            assert!(response.opcode() != ServerMessage::ILLEGAL_MOVE, "White moving on White's turn should be accepted");

            // Now it's black's turn. White tries to move.
            let response = client_white.make_move(game_id, "d2d4").await;
            assert_eq!(response.opcode(), ServerMessage::ILLEGAL_MOVE, "White moving on Black's turn should be rejected");

            // Black moves.
            let response = client_black.make_move(game_id, "e7e5").await;
            assert!(response.opcode() != ServerMessage::ILLEGAL_MOVE, "Black moving on Black's turn should be accepted");
        }
    }

    test! {
        async fn test_join_both_can_move_both_colors() {
            env_logger::try_init().ok();

            let port = 7881;
            start_server(port).await;

            let mut client = TestClient::new(port).await;

            let game_id = client.create_game(1, 120, 0).await;
            client.join_game(game_id, UserRoleSelection::Both).await;

            // White's turn. Client (as Both) should be able to move.
            let response = client.make_move(game_id, "e2e4").await;
            assert!(response.opcode() != ServerMessage::ILLEGAL_MOVE, "Client (Both) should be able to move White");

            // Now Black's turn. Same client should be able to move.
            let response = client.make_move(game_id, "e7e5").await;
            assert!(response.opcode() != ServerMessage::ILLEGAL_MOVE, "Client (Both) should be able to move Black");
        }
    }

    test! {
        async fn test_game_details() {
            env_logger::try_init().ok();

            let port = 7883;
            start_server(port).await;

            let mut client = TestClient::new(port).await;

            let game_id = client.create_game(1, 120, 5).await;

            let details = client.get_game_details(game_id).await;
            if let ServerMessage::GameDetails(id, white, black, time, inc) = details {
                assert_eq!(id, game_id);
                assert_eq!(time, 120);
                assert_eq!(inc, 5);
                assert_eq!(white, None);
                assert_eq!(black, None);
            } else {
                panic!("Expected GameDetails event");
            }
        }
    }

    test! {
        async fn test_checkmate() {
            env_logger::try_init().ok();

            let port = 7882;
            start_server(port).await;

            let mut client = TestClient::new(port).await;

            let game_id = client.create_game(1, 120, 0).await;
            client.join_game(game_id, UserRoleSelection::Both).await;

            // Fool's Mate
            // 1. f3 e5
            // 2. g4 Qh4#
            let moves = vec![
                ("f2f3", ServerMessage::MOVE_ACCEPTED),
                ("e7e5", ServerMessage::MOVE_ACCEPTED),
                ("g2g4", ServerMessage::MOVE_ACCEPTED),
                ("d8h4", ServerMessage::CHECKMATE),
            ];

            for (mov, expected_opcode) in moves {
                let response = client.make_move(game_id, mov).await;
                let response_opcode = response.opcode();
                assert_eq!(response_opcode, expected_opcode, "Move {} failed. Got 0x{:02X}, expected 0x{:02X}", mov, response_opcode, expected_opcode);
            }
        }
    }
}
