#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetStateResponse, InstantiateMsg, QueryMsg};
use crate::state::{GridCell, State, STATE, Turn};

/**
 * Tic Tac Toe contract
 * A game can only contains 2 players. The first player to reach 3 in a row, or 3 in a column, or 3 in a diagonal, wins.
 *
 * STATE:
 * 1. A board is represented by a 3x3 matrix. The board is initialized with a empty matrix. Item<Vec<Vec<STATE>>>
 * 2. STATE contains player address and the player's move.
 *
 * INSTANTIATE:
 * 1. Create a new game with the owner as the first player.
 * 2. The second player can join the game. Only two players can join the game.
 *
 * EXECUTE:
 * 1. Check if the player is allowed to play.
 * 2. A player takes turn to put the token on the board.
 * 3. Set the player's move to the board.
 *
 * QUERY:
 * 1. Get the current board.
 * 2. Get the current player.
 * 3. Get the winner.
 */

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tic-tac-toe";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        players: [info.sender.clone(), msg.opponent.clone()],
        board: [[GridCell::Empty; 3]; 3],
        next_turn: Turn::Player0,
        winner: None,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.to_string())
        .add_attribute("opponent", msg.opponent.to_string())
       .add_attribute("turn", state.next_turn.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Move {row, col} => try_move(deps, info, row, col),
    }
}

pub fn try_move(deps: DepsMut, info: MessageInfo, row: u8, col: u8) -> Result<Response, ContractError> {
    // check if the row and col are valid
    if (row < 0 || row > 2) || (col < 0 || col > 2) {
        return Err(ContractError::InvalidMove {
            msg: "Row and col must be between 1 and 3".to_string(),
        });
    }

    // Check if the player is eligible to play
    let state = STATE.load(deps.storage)?;
    if !state.players.contains(&info.sender) {
        return Err(ContractError::InvalidMove {
            msg: "You are not allowed to play".to_string(),
        });
    }

    // Check the player's turn is valid. Player0 = X, Player1 = O. PlayerO is the first player = contract owner
    match state.next_turn {
        Turn::Player0 => {
            if info.sender != state.players[0] {
                return Err(ContractError::InvalidMove {
                    msg: "It's not your turn".to_string(),
                });
            }
        },
        Turn::Player1 => {
            if info.sender != state.players[1] {
                return Err(ContractError::InvalidMove {
                    msg: "It's not your turn".to_string(),
                });
            }
        },
        Turn::Ended => {
            return Err(ContractError::InvalidMove {
                msg: "The game has already ended".to_string()
            });
        }
    }

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.next_turn = match state.next_turn {
            Turn::Player0 => Turn::Player1,
            Turn::Player1 => Turn::Player0,
            Turn::Ended => return Err(ContractError::InvalidMove {
                msg: "The game has already ended".to_string(),
            }),
        };

        state.board[row as usize][col as usize] = match state.next_turn {
            Turn::Player0 => GridCell::X,
            Turn::Player1 => GridCell::O,
            Turn::Ended => return Err(ContractError::InvalidMove {
                msg: "The game has already ended".to_string(),
            }),
        };

        state.winner = check_winner(&state.board, &state.players);
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_move"))
}

pub fn check_winner(board: &[[GridCell; 3]; 3], players: &[Addr; 2]) -> Option<Addr> {
    None
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
    }
}

fn query_state(deps: Deps) -> StdResult<GetStateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetStateResponse { state })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { opponent: Addr::unchecked("player1") };
        let info = mock_info("player0", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let state: GetStateResponse = from_binary(&res).unwrap();
        assert_eq!(Addr::unchecked("player1"), state.state.players[1]);
    }

    #[test]
    fn test_move() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { opponent: Addr::unchecked("player1") };
        let info = mock_info("player0", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("player0", &coins(2, "token"));
        let msg = ExecuteMsg::Move { row: 0, col: 0 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let state: GetStateResponse = from_binary(&res).unwrap();
        assert_eq!(GridCell::O, state.state.board[0][0]);
    }
}
